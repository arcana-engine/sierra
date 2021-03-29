use {
    super::{
        access::supported_access,
        convert::{oom_error_from_erupt, ToErupt},
        device::WeakDevice,
    },
    crate::{
        accel::{AccelerationStructureGeometry, AccelerationStructureLevel, IndexData},
        buffer::{BufferUsage, StridedBufferRegion},
        encode::*,
        format::{FormatDescription, FormatRepr, FormatType},
        queue::QueueId,
        render_pass::{AttachmentLoadOp, ClearValue, RENDERPASS_SMALLVEC_ATTACHMENTS},
        IndexType, OutOfMemory,
    },
    erupt::{
        extensions::{khr_acceleration_structure as vkacc, khr_ray_tracing_pipeline as vkrt},
        vk1_0,
    },
    smallvec::SmallVec,
    std::{
        convert::TryFrom as _,
        fmt::{self, Debug},
    },
};

pub struct CommandBuffer {
    handle: vk1_0::CommandBuffer,
    queue: QueueId,
    owner: WeakDevice,
    recording: bool,
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

impl CommandBuffer {
    pub(super) fn new(handle: vk1_0::CommandBuffer, queue: QueueId, owner: WeakDevice) -> Self {
        CommandBuffer {
            handle,
            queue,
            owner,
            recording: false,
        }
    }

    pub(super) fn handle(&self) -> vk1_0::CommandBuffer {
        self.handle
    }

    pub(super) fn is_owned_by(&self, owner: &impl PartialEq<WeakDevice>) -> bool {
        *owner == self.owner
    }

    pub fn queue(&self) -> QueueId {
        self.queue
    }

    pub fn write(&mut self, commands: &[Command<'_>]) -> Result<(), OutOfMemory> {
        let device = match self.owner.upgrade() {
            Some(device) => device,
            None => return Ok(()),
        };

        if !self.recording {
            unsafe {
                device.logical().begin_command_buffer(
                    self.handle,
                    &vk1_0::CommandBufferBeginInfoBuilder::new()
                        .flags(vk1_0::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
            }
            .result()
            .map_err(oom_error_from_erupt)?;

            self.recording = true;
        }

        let logical = &device.logical();

        for command in commands {
            match *command {
                Command::BeginRenderPass {
                    pass,
                    framebuffer,
                    clears,
                } => {
                    assert_owner!(pass, device);
                    assert_owner!(framebuffer, device);

                    let mut clears = clears.into_iter();

                    let clear_values = pass
                            .info()
                            .attachments
                            .iter()
                            .map(|attachment| {
                                use FormatDescription::*;

                                if attachment.load_op == AttachmentLoadOp::Clear {
                                    let clear = clears.next().expect("Not enough clear values");
                                    match clear {
                                        &ClearValue::Color(r, g, b, a) => vk1_0::ClearValue {
                                        color: match attachment.format.description() {
                                            R(repr)|RG(repr)|RGB(repr)|RGBA(repr)|BGR(repr)|BGRA(repr) => colors_f32_to_value(r, g, b, a, repr),
                                            _ => panic!("Attempt to clear depth-stencil attachment with color value"),
                                        }
                                    },
                                    &ClearValue::DepthStencil(depth, stencil) => {
                                        assert!(
                                            attachment.format.is_depth()
                                                || attachment.format.is_stencil()
                                        );
                                        vk1_0::ClearValue {
                                            depth_stencil: vk1_0::ClearDepthStencilValue {
                                                depth,
                                                stencil,
                                            },
                                        }
                                    }}
                                } else {
                                    vk1_0::ClearValue {
                                        color: vk1_0::ClearColorValue {
                                            uint32: [0; 4],
                                        }
                                    }
                                }
                            })
                            .collect::<SmallVec<[_; RENDERPASS_SMALLVEC_ATTACHMENTS]>>();

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
                                .clear_values(&clear_values),
                            vk1_0::SubpassContents::INLINE,
                        )
                    }
                }
                Command::EndRenderPass => unsafe { logical.cmd_end_render_pass(self.handle) },
                Command::BindGraphicsPipeline { pipeline } => unsafe {
                    assert_owner!(pipeline, device);

                    logical.cmd_bind_pipeline(
                        self.handle,
                        vk1_0::PipelineBindPoint::GRAPHICS,
                        pipeline.handle(),
                    )
                },
                Command::BindComputePipeline { pipeline } => unsafe {
                    assert_owner!(pipeline, device);

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
                    assert_eq!(offset % 4, 0);
                    assert!(data.len() < 65_536);
                    assert_owner!(buffer, device);

                    logical.cmd_update_buffer(
                        self.handle,
                        buffer.handle(),
                        offset,
                        data.len() as _,
                        data.as_ptr() as _,
                    );
                },
                Command::BindVertexBuffers { first, buffers } => unsafe {
                    for (buffer, _) in buffers {
                        assert_owner!(buffer, device);
                    }

                    let offsets: SmallVec<[_; 8]> =
                        buffers.iter().map(|&(_, offset)| offset).collect();

                    let buffers: SmallVec<[_; 8]> =
                        buffers.iter().map(|(buffer, _)| buffer.handle()).collect();

                    logical.cmd_bind_vertex_buffers(self.handle, first, &buffers, &offsets);
                },
                Command::BuildAccelerationStructure { infos } => {
                    assert!(
                        device.logical().enabled().khr_acceleration_structure,
                        "`AccelerationStructure` feature is not enabled"
                    );

                    // Vulkan specific checks.
                    assert!(u32::try_from(infos.len()).is_ok(), "Too many infos");

                    for (i, info) in infos.iter().enumerate() {
                        if let Some(src) = &info.src {
                            assert!(
                                src.is_owned_by(&device),
                                "`infos[{}].src` belongs to wrong device",
                                i
                            );
                        }

                        let dst = &info.dst;

                        assert!(
                            dst.is_owned_by(&device),
                            "`infos[{}].dst` belongs to wrong device",
                            i,
                        );
                    }

                    // Collect geometries.
                    let mut geometries = SmallVec::<[_; 32]>::new();

                    let mut offsets = SmallVec::<[_; 32]>::new();

                    let ranges: SmallVec<[_; 32]> = infos.iter().map(|info| {
                            if let Some(src) = &info.src {
                                assert_owner!(src, device);
                            }
                            assert_owner!(info.dst, device);

                            let mut total_primitive_count = 0u64;
                            let offset = geometries.len();

                            for geometry in info.geometries {
                                match geometry {
                                    AccelerationStructureGeometry::Triangles {
                                        flags,
                                        vertex_format,
                                        vertex_data,
                                        vertex_stride,
                                        vertex_count,
                                        first_vertex,
                                        primitive_count,
                                        index_data,
                                        transform_data,
                                    } => {
                                        total_primitive_count += (*primitive_count) as u64;
                                        geometries.push(vkacc::AccelerationStructureGeometryKHRBuilder::new()
                                            .flags(flags.to_erupt())
                                            .geometry_type(vkacc::GeometryTypeKHR::TRIANGLES_KHR)
                                            .geometry(vkacc::AccelerationStructureGeometryDataKHR {
                                                triangles: vkacc::AccelerationStructureGeometryTrianglesDataKHRBuilder::new()
                                                .vertex_format(vertex_format.to_erupt())
                                                .vertex_data(vertex_data.to_erupt())
                                                .vertex_stride(*vertex_stride)
                                                .max_vertex(*vertex_count)
                                                .index_type(match index_data {
                                                    None => vk1_0::IndexType::NONE_KHR,
                                                    Some(IndexData::U16(_)) => vk1_0::IndexType::UINT16,
                                                    Some(IndexData::U32(_)) => vk1_0::IndexType::UINT32,
                                                })
                                                .index_data(match index_data {
                                                    None => Default::default(),
                                                    Some(IndexData::U16(device_address)) => device_address.to_erupt(),
                                                    Some(IndexData::U32(device_address)) => device_address.to_erupt(),
                                                })
                                                .transform_data(transform_data.as_ref().map(|da| da.to_erupt()).unwrap_or_default())
                                                .build()
                                            }));

                                        offsets.push(vkacc::AccelerationStructureBuildRangeInfoKHRBuilder::new()
                                            .primitive_count(*primitive_count)
                                            .first_vertex(*first_vertex)
                                        );
                                    }
                                    AccelerationStructureGeometry::AABBs { flags, data, stride, primitive_count } => {
                                        total_primitive_count += (*primitive_count) as u64;
                                        geometries.push(vkacc::AccelerationStructureGeometryKHRBuilder::new()
                                            .flags(flags.to_erupt())
                                            .geometry_type(vkacc::GeometryTypeKHR::AABBS_KHR)
                                            .geometry(vkacc::AccelerationStructureGeometryDataKHR {
                                                aabbs: vkacc::AccelerationStructureGeometryAabbsDataKHRBuilder::new()
                                                    .data(data.to_erupt())
                                                    .stride(*stride)
                                                    .build()
                                            }));

                                        offsets.push(vkacc::AccelerationStructureBuildRangeInfoKHRBuilder::new()
                                            .primitive_count(*primitive_count)
                                        );
                                    }
                                    AccelerationStructureGeometry::Instances { flags, data, primitive_count } => {
                                        geometries.push(vkacc::AccelerationStructureGeometryKHRBuilder::new()
                                            .flags(flags.to_erupt())
                                            .geometry_type(vkacc::GeometryTypeKHR::INSTANCES_KHR)
                                            .geometry(vkacc::AccelerationStructureGeometryDataKHR {
                                                instances: vkacc::AccelerationStructureGeometryInstancesDataKHRBuilder::new()
                                                    .data(data.to_erupt())
                                                    .build()
                                            }));

                                        offsets.push(vkacc::AccelerationStructureBuildRangeInfoKHRBuilder::new()
                                            .primitive_count(*primitive_count)
                                        );
                                    }
                                }
                            }

                            if let AccelerationStructureLevel::Bottom = info.dst.info().level {
                                assert!(total_primitive_count <= device.properties().acc.max_primitive_count);
                            }

                            offset .. geometries.len()
                        }).collect();

                    let build_infos: SmallVec<[_; 32]> = infos
                        .iter()
                        .zip(&ranges)
                        .map(|(info, range)| {
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
                                .geometries(&geometries[range.clone()])
                        })
                        .collect();

                    let build_offsets: SmallVec<[_; 32]> = ranges
                        .into_iter()
                        .map(|range| &*offsets[range][0] as *const _)
                        .collect();

                    unsafe {
                        device.logical().cmd_build_acceleration_structures_khr(
                            self.handle,
                            &build_infos,
                            &build_offsets,
                        )
                    }
                }
                Command::BindIndexBuffer {
                    buffer,
                    offset,
                    index_type,
                } => unsafe {
                    assert_owner!(buffer, device);

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

                    for set in sets {
                        assert_owner!(set, device);
                    }

                    logical.cmd_bind_descriptor_sets(
                        self.handle,
                        vk1_0::PipelineBindPoint::GRAPHICS,
                        layout.handle(),
                        first_set,
                        &sets
                            .iter()
                            .map(|set| set.handle())
                            .collect::<SmallVec<[_; 8]>>(),
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

                    for set in sets {
                        assert_owner!(set, device);
                    }

                    logical.cmd_bind_descriptor_sets(
                        self.handle,
                        vk1_0::PipelineBindPoint::COMPUTE,
                        layout.handle(),
                        first_set,
                        &sets
                            .iter()
                            .map(|set| set.handle())
                            .collect::<SmallVec<[_; 8]>>(),
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

                    for set in sets {
                        assert_owner!(set, device);
                    }

                    logical.cmd_bind_descriptor_sets(
                        self.handle,
                        vk1_0::PipelineBindPoint::RAY_TRACING_KHR,
                        layout.handle(),
                        first_set,
                        &sets
                            .iter()
                            .map(|set| set.handle())
                            .collect::<SmallVec<[_; 8]>>(),
                        dynamic_offsets,
                    )
                },

                Command::TraceRays {
                    shader_binding_table,
                    extent,
                } => {
                    assert!(device.logical().enabled().khr_ray_tracing_pipeline);
                    if let Some(raygen) = &shader_binding_table.raygen {
                        assert_owner!(raygen.region.buffer, device);
                    }
                    if let Some(miss) = &shader_binding_table.miss {
                        assert_owner!(miss.region.buffer, device);
                    }
                    if let Some(hit) = &shader_binding_table.hit {
                        assert_owner!(hit.region.buffer, device);
                    }
                    if let Some(callable) = &shader_binding_table.callable {
                        assert_owner!(callable.region.buffer, device);
                    }

                    let sbr = vkrt::StridedDeviceAddressRegionKHR::default();

                    let to_erupt = |sbr: &StridedBufferRegion| {
                        assert!(sbr
                            .region
                            .buffer
                            .info()
                            .usage
                            .contains(BufferUsage::SHADER_BINDING_TABLE), "Buffers used to store shader binding table must be created with `SHADER_BINDING_TABLE` usage");

                        let device_address =
                            sbr.region.buffer.address().expect("Buffers used to store shader binding table must be created with `DEVICE_ADDRESS` usage").0.get();

                        vkrt::StridedDeviceAddressRegionKHRBuilder::new()
                            .device_address(device_address + sbr.region.offset)
                            .stride(sbr.stride)
                            .size(sbr.region.size)
                            .build()
                    };

                    unsafe {
                        device.logical().cmd_trace_rays_khr(
                            self.handle,
                            &shader_binding_table.raygen.as_ref().map_or(sbr, to_erupt),
                            &shader_binding_table.miss.as_ref().map_or(sbr, to_erupt),
                            &shader_binding_table.hit.as_ref().map_or(sbr, to_erupt),
                            &shader_binding_table.callable.as_ref().map_or(sbr, to_erupt),
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

                    logical.cmd_copy_image(
                        self.handle,
                        src_image.handle(),
                        src_layout.to_erupt(),
                        dst_image.handle(),
                        dst_layout.to_erupt(),
                        &regions
                            .iter()
                            .map(|region| region.to_erupt().into_builder())
                            .collect::<SmallVec<[_; 4]>>(),
                    );
                },

                Command::CopyBuffer {
                    src_buffer,
                    dst_buffer,
                    regions,
                } => unsafe {
                    assert_owner!(src_buffer, device);
                    assert_owner!(dst_buffer, device);

                    logical.cmd_copy_buffer(
                        self.handle,
                        src_buffer.handle(),
                        dst_buffer.handle(),
                        &regions
                            .iter()
                            .map(|region| region.to_erupt().into_builder())
                            .collect::<SmallVec<[_; 4]>>(),
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

                    logical.cmd_copy_buffer_to_image(
                        self.handle,
                        src_buffer.handle(),
                        dst_image.handle(),
                        dst_layout.to_erupt(),
                        &regions
                            .iter()
                            .map(|region| region.to_erupt().into_builder())
                            .collect::<SmallVec<[_; 4]>>(),
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

                    logical.cmd_blit_image(
                        self.handle,
                        src_image.handle(),
                        src_layout.to_erupt(),
                        dst_image.handle(),
                        dst_layout.to_erupt(),
                        &regions
                            .iter()
                            .map(|region| region.to_erupt().into_builder())
                            .collect::<SmallVec<[_; 4]>>(),
                        filter.to_erupt(),
                    );
                },

                Command::PipelineBarrier {
                    src,
                    dst,
                    images,
                    memory,
                } => unsafe {
                    for barrier in images {
                        assert_owner!(barrier.image, device);
                    }

                    logical.cmd_pipeline_barrier(
                        self.handle,
                        src.to_erupt(),
                        dst.to_erupt(),
                        None,
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
                        &[],
                        &images
                            .iter()
                            .map(|image| {
                                vk1_0::ImageMemoryBarrierBuilder::new()
                                    .image(image.image.handle())
                                    .src_access_mask(
                                        image
                                            .old
                                            .map(|(a, __)| a.to_erupt())
                                            .unwrap_or(vk1_0::AccessFlags::empty()),
                                    )
                                    .dst_access_mask(image.new.0.to_erupt())
                                    .old_layout(image.old.map(|(_, l)| l).to_erupt())
                                    .new_layout(image.new.1.to_erupt())
                                    .src_queue_family_index(
                                        image
                                            .family_transfer
                                            .as_ref()
                                            .map(|r| r.start)
                                            .unwrap_or(vk1_0::QUEUE_FAMILY_IGNORED),
                                    )
                                    .dst_queue_family_index(
                                        image
                                            .family_transfer
                                            .as_ref()
                                            .map(|r| r.end)
                                            .unwrap_or(vk1_0::QUEUE_FAMILY_IGNORED),
                                    )
                                    .subresource_range(image.subresource.to_erupt())
                            })
                            .collect::<SmallVec<[_; 8]>>(),
                    )
                },
                Command::PushConstants {
                    layout,
                    stages,
                    offset,
                    data,
                } => unsafe {
                    assert_owner!(layout, device);

                    logical.cmd_push_constants(
                        self.handle,
                        layout.handle(),
                        stages.to_erupt(),
                        offset,
                        data.len() as u32,
                        data.as_ptr() as *const _,
                    )
                },
                Command::Dispatch { x, y, z } => unsafe {
                    logical.cmd_dispatch(self.handle, x, y, z)
                },
            }
        }

        unsafe { logical.end_command_buffer(self.handle) }
            .result()
            .map_err(oom_error_from_erupt)?;

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

fn colors_f32_to_value(r: f32, g: f32, b: f32, a: f32, repr: FormatRepr) -> vk1_0::ClearColorValue {
    match repr {
        FormatRepr {
            bits: 8,
            ty: FormatType::Uint,
        } => vk1_0::ClearColorValue {
            uint32: [
                color_f32_to_uint8(r) as _,
                color_f32_to_uint8(g) as _,
                color_f32_to_uint8(b) as _,
                color_f32_to_uint8(a) as _,
            ],
        },
        FormatRepr {
            bits: 8,
            ty: FormatType::Sint,
        } => vk1_0::ClearColorValue {
            int32: [
                color_f32_to_sint8(r) as _,
                color_f32_to_sint8(g) as _,
                color_f32_to_sint8(b) as _,
                color_f32_to_sint8(a) as _,
            ],
        },
        FormatRepr {
            bits: 16,
            ty: FormatType::Uint,
        } => vk1_0::ClearColorValue {
            uint32: [
                color_f32_to_uint16(r) as _,
                color_f32_to_uint16(g) as _,
                color_f32_to_uint16(b) as _,
                color_f32_to_uint16(a) as _,
            ],
        },
        FormatRepr {
            bits: 16,
            ty: FormatType::Sint,
        } => vk1_0::ClearColorValue {
            int32: [
                color_f32_to_sint16(r) as _,
                color_f32_to_sint16(g) as _,
                color_f32_to_sint16(b) as _,
                color_f32_to_sint16(a) as _,
            ],
        },
        FormatRepr {
            bits: 32,
            ty: FormatType::Uint,
        } => vk1_0::ClearColorValue {
            uint32: [
                color_f32_to_uint32(r) as _,
                color_f32_to_uint32(g) as _,
                color_f32_to_uint32(b) as _,
                color_f32_to_uint32(a) as _,
            ],
        },
        FormatRepr {
            bits: 32,
            ty: FormatType::Sint,
        } => vk1_0::ClearColorValue {
            int32: [
                color_f32_to_sint32(r) as _,
                color_f32_to_sint32(g) as _,
                color_f32_to_sint32(b) as _,
                color_f32_to_sint32(a) as _,
            ],
        },
        FormatRepr {
            bits: 64,
            ty: FormatType::Uint,
        } => vk1_0::ClearColorValue {
            uint32: [
                color_f32_to_uint64(r) as _,
                color_f32_to_uint64(g) as _,
                color_f32_to_uint64(b) as _,
                color_f32_to_uint64(a) as _,
            ],
        },
        FormatRepr {
            bits: 64,
            ty: FormatType::Sint,
        } => vk1_0::ClearColorValue {
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
