use crate::{Format, RenderingAttachmentInfo};

use {
    super::{
        access::supported_access,
        convert::{oom_error_from_erupt, ToErupt},
        device::{Device, WeakDevice},
        epochs::References,
    },
    crate::{
        accel::{AccelerationStructureGeometry, AccelerationStructureLevel, IndexData},
        buffer::{BufferRange, BufferUsage, StridedBufferRange},
        encode::*,
        format::{Channels, FormatDescription, Type},
        queue::QueueId,
        render_pass::{ClearValue, LoadOp},
        IndexType, OutOfMemory,
    },
    erupt::{
        extensions::{khr_acceleration_structure as vkacc, khr_ray_tracing_pipeline as vkrt},
        vk1_0, vk1_3,
    },
    scoped_arena::Scope,
    std::{
        convert::TryFrom as _,
        fmt::{self, Debug},
    },
};

#[cfg(feature = "leak-detection")]
static COMMAND_BUFFER_ALLOCATED: AtomicU64 = AtomicU64::new(0);

#[cfg(feature = "leak-detection")]
static COMMAND_BUFFER_FREED: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
enum CommandBufferDevice {
    Strong(Device),
    Weak(WeakDevice),
}

pub struct CommandBuffer {
    handle: vk1_0::CommandBuffer,
    queue: QueueId,
    owner: CommandBufferDevice,
    references: References,
}

impl Debug for CommandBuffer {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            fmt.debug_struct("CommandBuffer ")
                .field("handle", &self.handle)
                .field("owner", &self.owner)
                .field("queue", &self.queue)
                .finish()
        } else {
            Debug::fmt(&self.handle, fmt)
        }
    }
}

#[cfg(feature = "leak-detection")]
impl Drop for CommandBuffer {
    fn drop(&mut self) {
        COMMAND_BUFFER_ALLOCATED.fetch_sub(1, Relaxed);
    }
}

impl CommandBuffer {
    pub(super) fn new(handle: vk1_0::CommandBuffer, queue: QueueId, owner: Device) -> Self {
        #[cfg(feature = "leak-detection")]
        let allocated = 1 + COMMAND_BUFFER_ALLOCATED.fetch_add(1, Relaxed);

        #[cfg(feature = "leak-detection")]
        if allocated - COMMAND_BUFFER_FREED.load(Relaxed) > 1024 {
            error!("Too many cbufs allocated");
        }

        CommandBuffer {
            handle,
            queue,
            owner: CommandBufferDevice::Strong(owner),
            references: References::new(),
        }
    }

    #[inline]
    pub(super) fn handle(&self) -> vk1_0::CommandBuffer {
        self.handle
    }

    #[inline]
    pub(super) fn is_owned_by(&self, owner: &Device) -> bool {
        match &self.owner {
            CommandBufferDevice::Strong(device) => *owner == *device,
            CommandBufferDevice::Weak(device) => *owner == *device,
        }
    }

    #[inline]
    pub(super) fn references(&mut self) -> &mut References {
        &mut self.references
    }

    #[inline]
    pub fn queue(&self) -> QueueId {
        self.queue
    }

    pub fn write(&mut self, scope: &Scope, command: Command<'_>) {
        let device = match &self.owner {
            CommandBufferDevice::Strong(device) => device,
            CommandBufferDevice::Weak(_) => return,
        };

        let references = &mut self.references;

        let logical = device.logical();

        match command {
            Command::BeginRenderPass {
                framebuffer,
                clears,
            } => {
                assert_owner!(framebuffer, device);
                references.add_framebuffer(framebuffer.clone());

                let pass = &framebuffer.info().render_pass;

                let mut clears = clears.iter();
                let clear_values =
                    scope.to_scope_from_iter(pass.info().attachments.iter().map(|attachment| {
                        if attachment.load_op == LoadOp::Clear {
                            let clear = clears.next().expect("Not enough clear values");
                            clear.to_erupt(attachment.format)
                        } else {
                            vk1_0::ClearValue::default()
                        }
                    }));

                assert!(clears.next().is_none(), "Too many clear values");

                unsafe {
                    logical.cmd_begin_render_pass(
                        self.handle,
                        &vk1_0::RenderPassBeginInfoBuilder::new()
                            .render_pass(pass.handle())
                            .framebuffer(framebuffer.handle()) //FIXME: Check `framebuffer` belongs to the
                            // pass.
                            .render_area(vk1_0::Rect2D {
                                offset: vk1_0::Offset2D { x: 0, y: 0 },
                                extent: framebuffer.info().extent.to_erupt(),
                            })
                            .clear_values(clear_values),
                        vk1_0::SubpassContents::INLINE,
                    )
                }
            }
            Command::EndRenderPass => unsafe { logical.cmd_end_render_pass(self.handle) },
            Command::BindGraphicsPipeline { pipeline } => unsafe {
                assert_owner!(pipeline, device);
                references.add_graphics_pipeline(pipeline.clone());

                logical.cmd_bind_pipeline(
                    self.handle,
                    vk1_0::PipelineBindPoint::GRAPHICS,
                    pipeline.handle(),
                )
            },
            Command::BindComputePipeline { pipeline } => unsafe {
                assert_owner!(pipeline, device);
                references.add_compute_pipeline(pipeline.clone());

                logical.cmd_bind_pipeline(
                    self.handle,
                    vk1_0::PipelineBindPoint::COMPUTE,
                    pipeline.handle(),
                )
            },
            Command::Draw {
                ref vertices,
                ref instances,
            } => unsafe {
                logical.cmd_draw(
                    self.handle,
                    vertices.end - vertices.start,
                    instances.end - instances.start,
                    vertices.start,
                    instances.start,
                )
            },
            Command::DrawIndexed {
                ref indices,
                vertex_offset,
                ref instances,
            } => unsafe {
                logical.cmd_draw_indexed(
                    self.handle,
                    indices.end - indices.start,
                    instances.end - instances.start,
                    indices.start,
                    vertex_offset,
                    instances.start,
                )
            },
            Command::SetViewport { viewport } => unsafe {
                // FIXME: Check that bound pipeline has dynamic viewport
                // state.
                logical.cmd_set_viewport(self.handle, 0, &[viewport.to_erupt().into_builder()]);
            },
            Command::SetScissor { scissor } => unsafe {
                // FIXME: Check that bound pipeline has dynamic scissor
                // state.
                logical.cmd_set_scissor(self.handle, 0, &[scissor.to_erupt().into_builder()]);
            },
            Command::UpdateBuffer {
                buffer,
                offset,
                data,
            } => unsafe {
                debug_assert_eq!(offset % 4, 0);
                debug_assert!(data.len() <= 65_536, "Data length greater than 65536 MUST NOT be uploaded with encoder, consider buffer mapping. Actual data is {} bytes", data.len());
                assert_owner!(buffer, device);
                references.add_buffer(buffer.clone());

                logical.cmd_update_buffer(
                    self.handle,
                    buffer.handle(),
                    offset,
                    data.len() as _,
                    data.as_ptr() as _,
                );
            },
            Command::BindVertexBuffers { first, buffers } => unsafe {
                for &(buffer, _) in buffers {
                    assert_owner!(buffer, device);
                    references.add_buffer(buffer.clone());
                }

                let offsets = scope.to_scope_from_iter(buffers.iter().map(|&(_, offset)| offset));

                let buffers =
                    scope.to_scope_from_iter(buffers.iter().map(|(buffer, _)| buffer.handle()));

                logical.cmd_bind_vertex_buffers(self.handle, first, buffers, offsets);
            },
            Command::BuildAccelerationStructure { infos } => {
                assert!(
                    device.logical().enabled().khr_acceleration_structure,
                    "`AccelerationStructure` feature is not enabled"
                );

                // Vulkan specific checks.
                assert!(u32::try_from(infos.len()).is_ok(), "Too many infos");

                for (i, info) in infos.iter().enumerate() {
                    if let Some(src) = info.src {
                        assert!(
                            src.is_owned_by(&device),
                            "`infos[{}].src` belongs to wrong device",
                            i
                        );
                    }

                    assert!(
                        info.dst.is_owned_by(&device),
                        "`infos[{}].dst` belongs to wrong device",
                        i,
                    );
                }

                let geometries_per_info = &*scope.to_scope_from_iter(infos.iter().map(|info| {
                        if let Some(src) = info.src {
                            references.add_acceleration_strucutre(src.clone());
                        }
                        references.add_acceleration_strucutre(info.dst.clone());
                        let mut total_primitive_count = 0u64;

                        let geometries = &*scope.to_scope_from_iter( info.geometries.iter().map(|geometry| {
                            match geometry {
                                AccelerationStructureGeometry::Triangles {
                                    flags,
                                    vertex_format,
                                    vertex_data,
                                    vertex_stride,
                                    vertex_count,
                                    primitive_count,
                                    index_data,
                                    transform_data,
                                    ..
                                } => {
                                    total_primitive_count += (*primitive_count) as u64;
                                    vkacc::AccelerationStructureGeometryKHRBuilder::new()
                                        .flags(flags.to_erupt())
                                        .geometry_type(vkacc::GeometryTypeKHR::TRIANGLES_KHR)
                                        .geometry(vkacc::AccelerationStructureGeometryDataKHR {
                                            triangles: vkacc::AccelerationStructureGeometryTrianglesDataKHRBuilder::new()
                                            .vertex_format(vertex_format.to_erupt())
                                            .vertex_data(buffer_range_to_device_address(vertex_data, references))
                                            .vertex_stride(*vertex_stride)
                                            .max_vertex(*vertex_count)
                                            .index_type(match index_data {
                                                None => vk1_0::IndexType::NONE_KHR,
                                                Some(IndexData::U16(_)) => vk1_0::IndexType::UINT16,
                                                Some(IndexData::U32(_)) => vk1_0::IndexType::UINT32,
                                            })
                                            .index_data(match index_data {
                                                None => Default::default(),
                                                Some(IndexData::U16(range)) => buffer_range_to_device_address(range, references),
                                                Some(IndexData::U32(range)) => buffer_range_to_device_address(range, references),
                                            })
                                            .transform_data(transform_data.as_ref().map(|da| buffer_range_to_device_address(da, references)).unwrap_or_default())
                                            .build_dangling()
                                        })
                                }
                                AccelerationStructureGeometry::AABBs { flags, data, stride, primitive_count } => {
                                    total_primitive_count += (*primitive_count) as u64;
                                    vkacc::AccelerationStructureGeometryKHRBuilder::new()
                                        .flags(flags.to_erupt())
                                        .geometry_type(vkacc::GeometryTypeKHR::AABBS_KHR)
                                        .geometry(vkacc::AccelerationStructureGeometryDataKHR {
                                            aabbs: vkacc::AccelerationStructureGeometryAabbsDataKHRBuilder::new()
                                                .data(buffer_range_to_device_address(data, references))
                                                .stride(*stride)
                                                .build_dangling()
                                        })
                                }
                                AccelerationStructureGeometry::Instances { flags, data, .. } => {
                                    vkacc::AccelerationStructureGeometryKHRBuilder::new()
                                        .flags(flags.to_erupt())
                                        .geometry_type(vkacc::GeometryTypeKHR::INSTANCES_KHR)
                                        .geometry(vkacc::AccelerationStructureGeometryDataKHR {
                                            instances: vkacc::AccelerationStructureGeometryInstancesDataKHRBuilder::new()
                                                .data(buffer_range_to_device_address(data, references))
                                                .build_dangling()
                                        })
                                }
                            }
                        }));

                        if let AccelerationStructureLevel::Bottom = info.dst.info().level {
                            assert!(total_primitive_count <= device.properties().acc.max_primitive_count);
                        }

                        geometries
                    }));

                let offsets_per_info = &*scope.to_scope_from_iter(infos.iter().map(|info| {
                    &*scope.to_scope_from_iter(info.geometries.iter().map(|geometry| {
                        match geometry {
                            AccelerationStructureGeometry::Triangles {
                                first_vertex,
                                primitive_count,
                                ..
                            } => vkacc::AccelerationStructureBuildRangeInfoKHRBuilder::new()
                                .primitive_count(*primitive_count)
                                .first_vertex(*first_vertex)
                                .build(),
                            AccelerationStructureGeometry::AABBs {
                                primitive_count, ..
                            } => vkacc::AccelerationStructureBuildRangeInfoKHRBuilder::new()
                                .primitive_count(*primitive_count)
                                .build(),
                            AccelerationStructureGeometry::Instances {
                                primitive_count, ..
                            } => vkacc::AccelerationStructureBuildRangeInfoKHRBuilder::new()
                                .primitive_count(*primitive_count)
                                .build(),
                        }
                    }))
                }));

                let build_infos =
                    scope.to_scope_from_iter(infos.iter().zip(geometries_per_info).map(
                        |(info, &geometries)| {
                            let src = info
                                .src
                                .as_ref()
                                .map(|src| src.handle())
                                .unwrap_or_default();

                            vkacc::AccelerationStructureBuildGeometryInfoKHRBuilder::new()
                                ._type(info.dst.info().level.to_erupt())
                                .flags(info.flags.to_erupt())
                                .mode(if info.src.is_some() {
                                    vkacc::BuildAccelerationStructureModeKHR::UPDATE_KHR
                                } else {
                                    vkacc::BuildAccelerationStructureModeKHR::BUILD_KHR
                                })
                                .src_acceleration_structure(src)
                                .dst_acceleration_structure(info.dst.handle())
                                .scratch_data(info.scratch.to_erupt()) // TODO: Validate this one.
                                .geometries(geometries)
                        },
                    ));

                let build_offsets = &*scope
                    .to_scope_from_iter(offsets_per_info.iter().map(|&offsets| offsets.as_ptr()));

                unsafe {
                    device.logical().cmd_build_acceleration_structures_khr(
                        self.handle,
                        &*build_infos,
                        build_offsets,
                    )
                }
            }
            Command::BindIndexBuffer {
                buffer,
                offset,
                index_type,
            } => unsafe {
                assert_owner!(buffer, device);
                references.add_buffer(buffer.clone());

                logical.cmd_bind_index_buffer(
                    self.handle,
                    buffer.handle(),
                    offset,
                    match index_type {
                        IndexType::U16 => vk1_0::IndexType::UINT16,
                        IndexType::U32 => vk1_0::IndexType::UINT32,
                    },
                );
            },

            Command::BindRayTracingPipeline { pipeline } => unsafe {
                assert_owner!(pipeline, device);
                references.add_ray_tracing_pipeline(pipeline.clone());

                logical.cmd_bind_pipeline(
                    self.handle,
                    vk1_0::PipelineBindPoint::RAY_TRACING_KHR,
                    pipeline.handle(),
                )
            },

            Command::BindGraphicsDescriptorSets {
                layout,
                first_set,
                sets,
                dynamic_offsets,
            } => unsafe {
                assert_owner!(layout, device);
                references.add_pipeline_layout(layout.clone());

                for &set in sets {
                    assert_owner!(set, device);
                    references.add_descriptor_set(set.clone());
                }

                logical.cmd_bind_descriptor_sets(
                    self.handle,
                    vk1_0::PipelineBindPoint::GRAPHICS,
                    layout.handle(),
                    first_set,
                    scope.to_scope_from_iter(sets.iter().map(|set| set.handle())),
                    dynamic_offsets,
                )
            },

            Command::BindComputeDescriptorSets {
                layout,
                first_set,
                sets,
                dynamic_offsets,
            } => unsafe {
                assert_owner!(layout, device);
                references.add_pipeline_layout(layout.clone());

                for &set in sets {
                    assert_owner!(set, device);
                    references.add_descriptor_set(set.clone());
                }

                logical.cmd_bind_descriptor_sets(
                    self.handle,
                    vk1_0::PipelineBindPoint::COMPUTE,
                    layout.handle(),
                    first_set,
                    scope.to_scope_from_iter(sets.iter().map(|set| set.handle())),
                    dynamic_offsets,
                )
            },

            Command::BindRayTracingDescriptorSets {
                layout,
                first_set,
                sets,
                dynamic_offsets,
            } => unsafe {
                assert_owner!(layout, device);
                references.add_pipeline_layout(layout.clone());

                for &set in sets {
                    assert_owner!(set, device);
                    references.add_descriptor_set(set.clone());
                }

                logical.cmd_bind_descriptor_sets(
                    self.handle,
                    vk1_0::PipelineBindPoint::RAY_TRACING_KHR,
                    layout.handle(),
                    first_set,
                    scope.to_scope_from_iter(sets.iter().map(|set| set.handle())),
                    dynamic_offsets,
                )
            },

            Command::TraceRays {
                shader_binding_table,
                extent,
            } => {
                assert!(device.logical().enabled().khr_ray_tracing_pipeline);
                if let Some(raygen) = &shader_binding_table.raygen {
                    assert_owner!(raygen.range.buffer, device);
                }
                if let Some(miss) = &shader_binding_table.miss {
                    assert_owner!(miss.range.buffer, device);
                }
                if let Some(hit) = &shader_binding_table.hit {
                    assert_owner!(hit.range.buffer, device);
                }
                if let Some(callable) = &shader_binding_table.callable {
                    assert_owner!(callable.range.buffer, device);
                }

                let sbr = vkrt::StridedDeviceAddressRegionKHR::default();

                unsafe {
                    device.logical().cmd_trace_rays_khr(
                        self.handle,
                        &shader_binding_table
                            .raygen
                            .as_ref()
                            .map_or(sbr, |sbr| strided_buffer_range_to_erupt(sbr, references)),
                        &shader_binding_table
                            .miss
                            .as_ref()
                            .map_or(sbr, |sbr| strided_buffer_range_to_erupt(sbr, references)),
                        &shader_binding_table
                            .hit
                            .as_ref()
                            .map_or(sbr, |sbr| strided_buffer_range_to_erupt(sbr, references)),
                        &shader_binding_table
                            .callable
                            .as_ref()
                            .map_or(sbr, |sbr| strided_buffer_range_to_erupt(sbr, references)),
                        extent.width,
                        extent.height,
                        extent.depth,
                    )
                }
            }
            Command::CopyImage {
                src_image,
                src_layout,
                dst_image,
                dst_layout,
                regions,
            } => unsafe {
                assert_owner!(src_image, device);
                assert_owner!(dst_image, device);

                references.add_image(src_image.clone());
                references.add_image(dst_image.clone());

                logical.cmd_copy_image(
                    self.handle,
                    src_image.handle(),
                    src_layout.to_erupt(),
                    dst_image.handle(),
                    dst_layout.to_erupt(),
                    scope.to_scope_from_iter(
                        regions
                            .iter()
                            .map(|region| region.to_erupt().into_builder()),
                    ),
                );
            },

            Command::CopyBuffer {
                src_buffer,
                dst_buffer,
                regions,
            } => unsafe {
                assert_owner!(src_buffer, device);
                assert_owner!(dst_buffer, device);

                references.add_buffer(src_buffer.clone());
                references.add_buffer(dst_buffer.clone());

                logical.cmd_copy_buffer(
                    self.handle,
                    src_buffer.handle(),
                    dst_buffer.handle(),
                    scope.to_scope_from_iter(
                        regions
                            .iter()
                            .map(|region| region.to_erupt().into_builder()),
                    ),
                );
            },
            Command::CopyBufferImage {
                src_buffer,
                dst_image,
                dst_layout,
                regions,
            } => unsafe {
                assert_owner!(src_buffer, device);
                assert_owner!(dst_image, device);

                references.add_buffer(src_buffer.clone());
                references.add_image(dst_image.clone());

                logical.cmd_copy_buffer_to_image(
                    self.handle,
                    src_buffer.handle(),
                    dst_image.handle(),
                    dst_layout.to_erupt(),
                    scope.to_scope_from_iter(
                        regions
                            .iter()
                            .map(|region| region.to_erupt().into_builder()),
                    ),
                );
            },

            Command::BlitImage {
                src_image,
                src_layout,
                dst_image,
                dst_layout,
                regions,
                filter,
            } => unsafe {
                assert_owner!(src_image, device);
                assert_owner!(dst_image, device);

                references.add_image(src_image.clone());
                references.add_image(dst_image.clone());

                logical.cmd_blit_image(
                    self.handle,
                    src_image.handle(),
                    src_layout.to_erupt(),
                    dst_image.handle(),
                    dst_layout.to_erupt(),
                    scope.to_scope_from_iter(
                        regions
                            .iter()
                            .map(|region| region.to_erupt().into_builder()),
                    ),
                    filter.to_erupt(),
                );
            },

            Command::PipelineBarrier {
                src,
                dst,
                images,
                buffers,
                memory,
            } => unsafe {
                for barrier in images {
                    assert_owner!(barrier.image, device);
                    references.add_image(barrier.image.clone());
                }
                for barrier in buffers {
                    assert_owner!(barrier.buffer, device);
                    references.add_buffer(barrier.buffer.clone());
                }

                logical.cmd_pipeline_barrier(
                    self.handle,
                    src.to_erupt(),
                    dst.to_erupt(),
                    vk1_0::DependencyFlags::empty(),
                    &[vk1_0::MemoryBarrierBuilder::new()
                        .src_access_mask(
                            memory
                                .as_ref()
                                .map_or(supported_access(src.to_erupt()), |m| m.src.to_erupt()),
                        )
                        .dst_access_mask(
                            memory
                                .as_ref()
                                .map_or(supported_access(dst.to_erupt()), |m| m.dst.to_erupt()),
                        )],
                    scope.to_scope_from_iter(buffers.iter().map(|buffer| {
                        vk1_0::BufferMemoryBarrierBuilder::new()
                            .buffer(buffer.buffer.handle())
                            .offset(buffer.offset)
                            .size(buffer.size)
                            .src_access_mask(buffer.old_access.to_erupt())
                            .dst_access_mask(buffer.new_access.to_erupt())
                            .src_queue_family_index(
                                buffer
                                    .family_transfer
                                    .as_ref()
                                    .map(|r| r.0)
                                    .unwrap_or(vk1_0::QUEUE_FAMILY_IGNORED),
                            )
                            .dst_queue_family_index(
                                buffer
                                    .family_transfer
                                    .as_ref()
                                    .map(|r| r.1)
                                    .unwrap_or(vk1_0::QUEUE_FAMILY_IGNORED),
                            )
                    })),
                    scope.to_scope_from_iter(images.iter().map(|image| {
                        vk1_0::ImageMemoryBarrierBuilder::new()
                            .image(image.image.handle())
                            .subresource_range(image.range.to_erupt())
                            .src_access_mask(image.old_access.to_erupt())
                            .dst_access_mask(image.new_access.to_erupt())
                            .old_layout(image.old_layout.to_erupt())
                            .new_layout(image.new_layout.to_erupt())
                            .src_queue_family_index(
                                image
                                    .family_transfer
                                    .as_ref()
                                    .map(|r| r.0)
                                    .unwrap_or(vk1_0::QUEUE_FAMILY_IGNORED),
                            )
                            .dst_queue_family_index(
                                image
                                    .family_transfer
                                    .as_ref()
                                    .map(|r| r.1)
                                    .unwrap_or(vk1_0::QUEUE_FAMILY_IGNORED),
                            )
                    })),
                )
            },
            Command::PushConstants {
                layout,
                stages,
                offset,
                data,
            } => unsafe {
                assert_owner!(layout, device);
                references.add_pipeline_layout(layout.clone());

                logical.cmd_push_constants(
                    self.handle,
                    layout.handle(),
                    stages.to_erupt(),
                    offset,
                    data.len() as u32,
                    data.as_ptr() as *const _,
                )
            },
            Command::Dispatch { x, y, z } => unsafe { logical.cmd_dispatch(self.handle, x, y, z) },

            Command::BeginRendering { info } => {
                assert_ne!(
                    device.features().v13.dynamic_rendering,
                    0,
                    "DynamicRendering feature is not enabled"
                );

                let attachment_to_erupt = |attachment: &RenderingAttachmentInfo| {
                    let clear_value = match attachment.load_op {
                        LoadOp::Clear => match attachment.clear_value {
                            None => vk1_0::ClearValue::default(),
                            Some(clear_value) => clear_value
                                .to_erupt(attachment.image_view.info().image.info().format),
                        },
                        _ => vk1_0::ClearValue::default(),
                    };

                    vk1_3::RenderingAttachmentInfoBuilder::new()
                        .image_view(attachment.image_view.handle())
                        .image_layout(attachment.image_layout.to_erupt())
                        .load_op(attachment.load_op.to_erupt())
                        .store_op(attachment.store_op.to_erupt())
                        .clear_value(clear_value)
                };

                let mut builder =
                    vk1_3::RenderingInfoBuilder::new().render_area(info.render_area.to_erupt());

                let colors = scope.to_scope_from_iter(info.colors.iter().map(attachment_to_erupt));

                builder = builder.color_attachments(&*colors);

                let depth_attachment_info;
                if let Some(depth) = &info.depth {
                    depth_attachment_info = attachment_to_erupt(depth);
                    builder = builder.depth_attachment(&depth_attachment_info);
                }

                let stencil_attachment_info;
                if let Some(stencil) = &info.stencil {
                    stencil_attachment_info = attachment_to_erupt(stencil);
                    builder = builder.stencil_attachment(&stencil_attachment_info);
                }

                unsafe { logical.cmd_begin_rendering(self.handle, &builder) }
            }

            Command::EndRendering => {
                assert_ne!(
                    device.features().v13.dynamic_rendering,
                    0,
                    "DynamicRendering feature is not enabled"
                );

                unsafe { logical.cmd_end_rendering(self.handle) }
            }
        }
    }

    pub fn begin(&mut self) -> Result<(), OutOfMemory> {
        let upgraded;
        let device = match &self.owner {
            CommandBufferDevice::Strong(device) => device,
            CommandBufferDevice::Weak(weak) => match weak.upgrade() {
                None => return Ok(()),
                Some(device) => {
                    self.owner = CommandBufferDevice::Strong(device.clone());
                    upgraded = device;
                    &upgraded
                }
            },
        };

        let logical = device.logical();

        unsafe {
            logical.begin_command_buffer(
                self.handle,
                &vk1_0::CommandBufferBeginInfoBuilder::new()
                    .flags(vk1_0::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )
        }
        .result()
        .map_err(oom_error_from_erupt)
    }

    pub fn end(&mut self) -> Result<(), OutOfMemory> {
        let device = match &self.owner {
            CommandBufferDevice::Strong(device) => device,
            CommandBufferDevice::Weak(_) => return Ok(()),
        };

        let logical = device.logical();

        unsafe { logical.end_command_buffer(self.handle) }
            .result()
            .map_err(oom_error_from_erupt)?;

        let weak = device.downgrade();
        self.owner = CommandBufferDevice::Weak(weak);
        Ok(())
    }
}

fn color_f32_to_uint64(color: f32) -> u64 {
    color.min(0f32).max(u64::max_value() as f32) as u64
}

fn color_f32_to_sint64(color: f32) -> i64 {
    color
        .min(i64::min_value() as f32)
        .max(i64::max_value() as f32) as i64
}

fn color_f32_to_uint32(color: f32) -> u32 {
    color.min(0f32).max(u32::max_value() as f32) as u32
}

fn color_f32_to_sint32(color: f32) -> i32 {
    color
        .min(i32::min_value() as f32)
        .max(i32::max_value() as f32) as i32
}

fn color_f32_to_uint16(color: f32) -> u16 {
    color.min(0f32).max(u16::max_value() as f32) as u16
}

fn color_f32_to_sint16(color: f32) -> i16 {
    color
        .min(i16::min_value() as f32)
        .max(i16::max_value() as f32) as i16
}

fn color_f32_to_uint8(color: f32) -> u8 {
    color.min(0f32).max(u8::max_value() as f32) as u8
}

fn color_f32_to_sint8(color: f32) -> i8 {
    color
        .min(i8::min_value() as f32)
        .max(i8::max_value() as f32) as i8
}

fn colors_f32_to_value(
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    bits: u32,
    ty: Type,
) -> vk1_0::ClearColorValue {
    match (bits, ty) {
        (8, Type::Uint) => vk1_0::ClearColorValue {
            uint32: [
                color_f32_to_uint8(r) as _,
                color_f32_to_uint8(g) as _,
                color_f32_to_uint8(b) as _,
                color_f32_to_uint8(a) as _,
            ],
        },
        (8, Type::Sint) => vk1_0::ClearColorValue {
            int32: [
                color_f32_to_sint8(r) as _,
                color_f32_to_sint8(g) as _,
                color_f32_to_sint8(b) as _,
                color_f32_to_sint8(a) as _,
            ],
        },
        (16, Type::Uint) => vk1_0::ClearColorValue {
            uint32: [
                color_f32_to_uint16(r) as _,
                color_f32_to_uint16(g) as _,
                color_f32_to_uint16(b) as _,
                color_f32_to_uint16(a) as _,
            ],
        },
        (16, Type::Sint) => vk1_0::ClearColorValue {
            int32: [
                color_f32_to_sint16(r) as _,
                color_f32_to_sint16(g) as _,
                color_f32_to_sint16(b) as _,
                color_f32_to_sint16(a) as _,
            ],
        },
        (32, Type::Uint) => vk1_0::ClearColorValue {
            uint32: [
                color_f32_to_uint32(r) as _,
                color_f32_to_uint32(g) as _,
                color_f32_to_uint32(b) as _,
                color_f32_to_uint32(a) as _,
            ],
        },
        (32, Type::Sint) => vk1_0::ClearColorValue {
            int32: [
                color_f32_to_sint32(r) as _,
                color_f32_to_sint32(g) as _,
                color_f32_to_sint32(b) as _,
                color_f32_to_sint32(a) as _,
            ],
        },
        (64, Type::Uint) => vk1_0::ClearColorValue {
            uint32: [
                color_f32_to_uint64(r) as _,
                color_f32_to_uint64(g) as _,
                color_f32_to_uint64(b) as _,
                color_f32_to_uint64(a) as _,
            ],
        },
        (64, Type::Sint) => vk1_0::ClearColorValue {
            int32: [
                color_f32_to_sint64(r) as _,
                color_f32_to_sint64(g) as _,
                color_f32_to_sint64(b) as _,
                color_f32_to_sint64(a) as _,
            ],
        },
        _ => vk1_0::ClearColorValue {
            float32: [r, g, b, a],
        },
    }
}

fn buffer_range_to_device_address(
    range: &BufferRange,
    references: &mut References,
) -> vkacc::DeviceOrHostAddressConstKHR {
    assert!(range
        .buffer
        .info()
        .usage
        .contains(BufferUsage::DEVICE_ADDRESS));

    references.add_buffer(range.buffer.clone());

    let device_address = range.buffer.address().unwrap();
    let device_address = device_address.0.get();
    vkacc::DeviceOrHostAddressConstKHR {
        device_address: device_address + range.offset,
    }
}

fn strided_buffer_range_to_erupt(
    sbr: &StridedBufferRange,
    references: &mut References,
) -> vkrt::StridedDeviceAddressRegionKHR {
    assert!(sbr
        .range
        .buffer
        .info()
        .usage
        .contains(BufferUsage::SHADER_BINDING_TABLE), "Buffers used to store shader binding table must be created with `SHADER_BINDING_TABLE` usage");

    references.add_buffer(sbr.range.buffer.clone());

    let device_address =
        sbr.range.buffer.address().expect("Buffers used to store shader binding table must be created with `DEVICE_ADDRESS` usage").0.get();

    vkrt::StridedDeviceAddressRegionKHRBuilder::new()
        .device_address(device_address + sbr.range.offset)
        .stride(sbr.stride)
        .size(sbr.range.size)
        .build()
}

impl ClearValue {
    fn to_erupt(self, format: Format) -> vk1_0::ClearValue {
        use Channels::*;

        match self {
            ClearValue::Color(r, g, b, a) => vk1_0::ClearValue {
                color: match format.description() {
                    FormatDescription {
                        channels: R | RG | RGB | RGBA | BGR | BGRA,
                        bits,
                        ty,
                    } => colors_f32_to_value(r, g, b, a, bits, ty),
                    _ => panic!("Attempt to clear depth-stencil attachment with color value"),
                },
            },
            ClearValue::DepthStencil(depth, stencil) => {
                assert!(format.is_depth() || format.is_stencil());
                vk1_0::ClearValue {
                    depth_stencil: vk1_0::ClearDepthStencilValue { depth, stencil },
                }
            }
        }
    }
}
