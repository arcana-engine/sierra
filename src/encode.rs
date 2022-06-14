use std::{
    fmt,
    mem::{forget, size_of_val},
    ops::Range,
};

use bytemuck::{cast_slice, Pod};
use scoped_arena::Scope;

use crate::{
    accel::AccelerationStructureBuildGeometryInfo,
    access::AccessFlags,
    arith_ge, arith_le,
    buffer::{Buffer, BufferMemoryBarrier},
    descriptor::{DescriptorSet, UpdatedPipelineDescriptors},
    framebuffer::{Framebuffer, FramebufferError},
    image::{Image, ImageBlit, ImageMemoryBarrier, Layout, SubresourceLayers},
    memory::MemoryBarrier,
    pipeline::{
        ComputePipeline, DynamicGraphicsPipeline, GraphicsPipeline, PipelineInputLayout,
        PipelineLayout, RayTracingPipeline, ShaderBindingTable, Viewport,
    },
    queue::QueueCapabilityFlags,
    render_pass::{ClearValue, RenderPass, RenderPassInstance},
    sampler::Filter,
    shader::ShaderStageFlags,
    stage::PipelineStageFlags,
    BufferInfo, BufferUsage, Device, Extent3d, IndexType, Offset3d, OutOfMemory,
    PipelinePushConstants, Rect2d,
};

pub use crate::backend::CommandBuffer;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct BufferCopy {
    pub src_offset: u64,
    pub dst_offset: u64,
    pub size: u64,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct ImageCopy {
    pub src_subresource: SubresourceLayers,
    pub src_offset: Offset3d,
    pub dst_subresource: SubresourceLayers,
    pub dst_offset: Offset3d,
    pub extent: Extent3d,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct BufferImageCopy {
    pub buffer_offset: u64,
    pub buffer_row_length: u32,
    pub buffer_image_height: u32,
    pub image_subresource: SubresourceLayers,
    pub image_offset: Offset3d,
    pub image_extent: Extent3d,
}

#[derive(Debug)]
pub enum Command<'a> {
    BeginRenderPass {
        framebuffer: &'a Framebuffer,
        clears: &'a [ClearValue],
    },
    EndRenderPass,

    BindGraphicsPipeline {
        pipeline: &'a GraphicsPipeline,
    },

    BindComputePipeline {
        pipeline: &'a ComputePipeline,
    },

    BindRayTracingPipeline {
        pipeline: &'a RayTracingPipeline,
    },

    BindGraphicsDescriptorSets {
        layout: &'a PipelineLayout,
        first_set: u32,
        sets: &'a [&'a DescriptorSet],
        dynamic_offsets: &'a [u32],
    },

    BindComputeDescriptorSets {
        layout: &'a PipelineLayout,
        first_set: u32,
        sets: &'a [&'a DescriptorSet],
        dynamic_offsets: &'a [u32],
    },

    BindRayTracingDescriptorSets {
        layout: &'a PipelineLayout,
        first_set: u32,
        sets: &'a [&'a DescriptorSet],
        dynamic_offsets: &'a [u32],
    },

    SetViewport {
        viewport: Viewport,
    },

    SetScissor {
        scissor: Rect2d,
    },

    Draw {
        vertices: Range<u32>,
        instances: Range<u32>,
    },

    DrawIndexed {
        indices: Range<u32>,
        vertex_offset: i32,
        instances: Range<u32>,
    },

    UpdateBuffer {
        buffer: &'a Buffer,
        offset: u64,
        data: &'a [u8],
    },

    BindVertexBuffers {
        first: u32,
        buffers: &'a [(&'a Buffer, u64)],
    },

    BindIndexBuffer {
        buffer: &'a Buffer,
        offset: u64,
        index_type: IndexType,
    },

    BuildAccelerationStructure {
        infos: &'a [AccelerationStructureBuildGeometryInfo<'a>],
    },

    TraceRays {
        shader_binding_table: &'a ShaderBindingTable,
        extent: Extent3d,
    },

    CopyBuffer {
        src_buffer: &'a Buffer,
        dst_buffer: &'a Buffer,
        regions: &'a [BufferCopy],
    },

    CopyImage {
        src_image: &'a Image,
        src_layout: Layout,
        dst_image: &'a Image,
        dst_layout: Layout,
        regions: &'a [ImageCopy],
    },

    CopyBufferImage {
        src_buffer: &'a Buffer,
        dst_image: &'a Image,
        dst_layout: Layout,
        regions: &'a [BufferImageCopy],
    },

    BlitImage {
        src_image: &'a Image,
        src_layout: Layout,
        dst_image: &'a Image,
        dst_layout: Layout,
        regions: &'a [ImageBlit],
        filter: Filter,
    },

    PipelineBarrier {
        src: PipelineStageFlags,
        dst: PipelineStageFlags,
        images: &'a [ImageMemoryBarrier<'a>],
        buffers: &'a [BufferMemoryBarrier<'a>],
        memory: Option<MemoryBarrier>,
    },

    PushConstants {
        layout: &'a PipelineLayout,
        stages: ShaderStageFlags,
        offset: u32,
        data: &'a [u8],
    },

    Dispatch {
        x: u32,
        y: u32,
        z: u32,
    },
}

/// Basis for encoding capabilities.
/// Implements encoding of commands that can be inside and outside of render
/// pass.
#[allow(missing_debug_implementations)]
pub struct EncoderCommon<'a> {
    capabilities: QueueCapabilityFlags,
    scope: &'a Scope<'a>,
    command_buffer: CommandBuffer,
}

impl<'a> EncoderCommon<'a> {
    pub fn scope(&self) -> &'a Scope<'a> {
        self.scope
    }

    pub fn set_viewport(&mut self, viewport: Viewport) {
        assert!(self.capabilities.supports_graphics());

        self.command_buffer
            .write(self.scope, Command::SetViewport { viewport });
    }

    pub fn set_scissor(&mut self, scissor: Rect2d) {
        assert!(self.capabilities.supports_graphics());

        self.command_buffer
            .write(self.scope, Command::SetScissor { scissor })
    }

    pub fn bind_graphics_pipeline(&mut self, pipeline: &GraphicsPipeline) {
        assert!(self.capabilities.supports_graphics());

        self.command_buffer
            .write(self.scope, Command::BindGraphicsPipeline { pipeline })
    }

    pub fn bind_compute_pipeline(&mut self, pipeline: &ComputePipeline) {
        assert!(self.capabilities.supports_compute());
        self.command_buffer
            .write(self.scope, Command::BindComputePipeline { pipeline })
    }

    pub fn bind_ray_tracing_pipeline(&mut self, pipeline: &RayTracingPipeline) {
        assert!(self.capabilities.supports_compute());

        self.command_buffer
            .write(self.scope, Command::BindRayTracingPipeline { pipeline })
    }

    pub fn bind_vertex_buffers(&mut self, first: u32, buffers: &[(&Buffer, u64)]) {
        assert!(self.capabilities.supports_graphics());

        self.command_buffer
            .write(self.scope, Command::BindVertexBuffers { first, buffers })
    }

    pub fn bind_index_buffer(&mut self, buffer: &Buffer, offset: u64, index_type: IndexType) {
        assert!(self.capabilities.supports_graphics());

        self.command_buffer.write(
            self.scope,
            Command::BindIndexBuffer {
                buffer,
                offset,
                index_type,
            },
        )
    }

    pub fn bind_graphics_descriptor_sets(
        &mut self,
        layout: &PipelineLayout,
        first_set: u32,
        sets: &[&DescriptorSet],
        dynamic_offsets: &[u32],
    ) {
        assert!(self.capabilities.supports_graphics());

        self.command_buffer.write(
            self.scope,
            Command::BindGraphicsDescriptorSets {
                layout,
                first_set,
                sets,
                dynamic_offsets,
            },
        );
    }

    pub fn bind_graphics_descriptors<P, const N: u32>(
        &mut self,
        layout: &P,
        descriptors: &impl UpdatedPipelineDescriptors<P, N>,
    ) where
        P: PipelineInputLayout,
    {
        layout.bind_graphics(descriptors, self);
    }

    pub fn bind_compute_descriptor_sets(
        &mut self,
        layout: &PipelineLayout,
        first_set: u32,
        sets: &[&DescriptorSet],
        dynamic_offsets: &[u32],
    ) {
        assert!(self.capabilities.supports_compute());

        self.command_buffer.write(
            self.scope,
            Command::BindComputeDescriptorSets {
                layout,
                first_set,
                sets,
                dynamic_offsets,
            },
        );
    }

    pub fn bind_compute_descriptors<P, const N: u32>(
        &mut self,
        layout: &P,
        descriptors: &impl UpdatedPipelineDescriptors<P, N>,
    ) where
        P: PipelineInputLayout,
    {
        layout.bind_compute(descriptors, self);
    }

    pub fn bind_ray_tracing_descriptor_sets(
        &mut self,
        layout: &PipelineLayout,
        first_set: u32,
        sets: &[&DescriptorSet],
        dynamic_offsets: &[u32],
    ) {
        assert!(self.capabilities.supports_compute());

        self.command_buffer.write(
            self.scope,
            Command::BindRayTracingDescriptorSets {
                layout,
                first_set,
                sets,
                dynamic_offsets,
            },
        );
    }

    pub fn bind_ray_tracing_descriptors<P, const N: u32>(
        &mut self,
        layout: &P,
        descriptors: &impl UpdatedPipelineDescriptors<P, N>,
    ) where
        P: PipelineInputLayout,
    {
        layout.bind_ray_tracing(descriptors, self);
    }

    pub fn push_constants_pod<T>(
        &mut self,
        layout: &PipelineLayout,
        stages: ShaderStageFlags,
        offset: u32,
        data: &[T],
    ) where
        T: Pod,
    {
        assert!(arith_le(size_of_val(data), u32::max_value()));

        self.command_buffer.write(
            self.scope,
            Command::PushConstants {
                layout,
                stages,
                offset,
                data: cast_slice(data),
            },
        );
    }

    pub fn push_constants<P>(&mut self, layout: &P, constants: &impl PipelinePushConstants<P>)
    where
        P: PipelineInputLayout,
    {
        layout.push_constants(constants, self);
    }
}

/// Command encoder that can encode commands outside render pass.
pub struct Encoder<'a> {
    inner: EncoderCommon<'a>,
    drop: EncoderDrop,
}

impl<'a> fmt::Debug for Encoder<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Encoder")
            .field("command_buffer", &self.inner.command_buffer)
            .field("capabilities", &self.inner.capabilities)
            .finish()
    }
}

struct EncoderDrop;

impl Drop for EncoderDrop {
    fn drop(&mut self) {
        #[cfg(feature = "tracing")]
        tracing::warn!(
            "Encoder is dropped. Encoders must be either submitted or explicitly discarded"
        );

        #[cfg(not(feature = "tracing"))]
        eprintln!("Encoder is dropped. Encoders must be either submitted or explicitly discarded")
    }
}

impl<'a> Encoder<'a> {
    pub fn discard(self) {
        forget(self.drop)
    }
}

impl<'a> std::ops::Deref for Encoder<'a> {
    type Target = EncoderCommon<'a>;

    fn deref(&self) -> &EncoderCommon<'a> {
        &self.inner
    }
}

impl<'a> std::ops::DerefMut for Encoder<'a> {
    fn deref_mut(&mut self) -> &mut EncoderCommon<'a> {
        &mut self.inner
    }
}

impl<'a> Encoder<'a> {
    pub(crate) fn new(
        command_buffer: CommandBuffer,
        capabilities: QueueCapabilityFlags,
        scope: &'a Scope<'a>,
    ) -> Self {
        Encoder {
            inner: EncoderCommon {
                capabilities,
                scope,
                command_buffer,
            },
            drop: EncoderDrop,
        }
    }

    /// Begins render pass and returns `RenderPassEncoder` to encode commands of
    /// the render pass. `RenderPassEncoder` borrows `Encoder`.
    /// To continue use this `Encoder` returned `RenderPassEncoder` must be
    /// dropped which implicitly ends render pass.
    ///
    /// `framebuffer` - a framebuffer (set of attachments) for render pass to use.
    /// `clears` - an array of clear values. render pass will clear attachments
    ///            with `load_op == LoadOp::Clear` using those values.
    ///            They will be used in order.
    pub fn with_framebuffer(
        &mut self,
        framebuffer: &'a Framebuffer,
        clears: &[ClearValue],
    ) -> RenderPassEncoder<'_, 'a> {
        assert!(self.inner.capabilities.supports_graphics());

        self.inner.command_buffer.write(
            self.scope,
            Command::BeginRenderPass {
                framebuffer,
                clears,
            },
        );

        RenderPassEncoder {
            framebuffer,
            render_pass: &framebuffer.info().render_pass,
            inner: &mut self.inner,
            subpass: 0,
        }
    }

    /// Begins render pass and returns `RenderPassEncoder` to encode commands of
    /// the render pass. `RenderPassEncoder` borrows `Encoder`.
    /// To continue use this `Encoder` returned `RenderPassEncoder` must be
    /// dropped which implicitly ends render pass.
    ///
    /// `pass` - render pass to encode.
    /// `framebuffer` - a framebuffer (set of attachments) for render pass to use.
    /// `clears` - an array of clear values. render pass will clear attachments
    ///            with `load_op == LoadOp::Clear` using those values.
    ///            They will be used in order.
    pub fn with_render_pass<R, I>(
        &mut self,
        render_pass: &'a mut R,
        input: &I,
        device: &Device,
    ) -> Result<RenderPassEncoder<'_, 'a>, FramebufferError>
    where
        R: RenderPassInstance<Input = I>,
    {
        render_pass.begin_render_pass(input, device, self)
    }

    /// Updates a buffer's contents from host memory
    pub fn update_buffer<T>(&mut self, buffer: &Buffer, offset: u64, data: &[T])
    where
        T: Pod,
    {
        assert_eq!(offset % 4, 0);
        assert!(size_of_val(data) <= 65_536, "Data length greater than 65536 MUST NOT be uploaded with encoder, consider buffer mapping. Actual data is {} bytes", size_of_val(data));

        if data.is_empty() {
            return;
        }

        let data = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data))
        };

        self.inner.command_buffer.write(
            self.scope,
            Command::UpdateBuffer {
                buffer,
                offset,
                data,
            },
        )
    }

    /// Uploads data to the buffer.
    /// May create intermediate staging buffer if necessary.
    pub fn upload_buffer<T>(
        &mut self,
        buffer: &'a Buffer,
        offset: u64,
        data: &'a [T],
        device: &Device,
    ) -> Result<(), OutOfMemory>
    where
        T: Pod,
    {
        const UPDATE_LIMIT: usize = 16384;

        assert_eq!(
            size_of_val(data) & 3,
            0,
            "Buffer uploading data size must be a multiple of 4"
        );

        if data.is_empty() {
            return Ok(());
        }

        if size_of_val(data) <= UPDATE_LIMIT {
            self.update_buffer(buffer, offset, data);
        } else {
            let staging = device.create_buffer_static(
                BufferInfo {
                    align: 15,
                    size: size_of_val(data) as u64,
                    usage: BufferUsage::TRANSFER_SRC,
                },
                data,
            )?;

            self.copy_buffer(
                &staging,
                buffer,
                &[BufferCopy {
                    src_offset: 0,
                    dst_offset: offset,
                    size: size_of_val(data) as u64,
                }],
            );
        }

        Ok(())
    }

    /// Uploads data to the buffer.
    /// Uses cached staging buffers and may create new if necessary.
    pub fn upload_buffer_cached<T, S>(
        &mut self,
        buffer: &'a Buffer,
        offset: u64,
        data: &'a [T],
        device: &Device,
        staging: &mut S,
    ) -> Result<(), OutOfMemory>
    where
        T: Pod,
        S: AsMut<[Buffer]> + Extend<Buffer>,
    {
        const UPDATE_LIMIT: usize = 16384;

        assert_eq!(
            size_of_val(data) & 3,
            0,
            "Buffer uploading data size must be a multiple of 4"
        );

        if data.is_empty() {
            return Ok(());
        }

        if size_of_val(data) <= UPDATE_LIMIT {
            self.update_buffer(buffer, offset, data);
        } else {
            let new_staging;
            let mut iter = staging.as_mut().iter_mut();
            let staging = loop {
                match iter.next() {
                    None => {
                        new_staging = device.create_buffer_static(
                            BufferInfo {
                                align: 15,
                                size: size_of_val(data) as u64,
                                usage: BufferUsage::TRANSFER_SRC,
                            },
                            data,
                        )?;
                        break &new_staging;
                    }
                    Some(buffer) => {
                        if arith_ge(buffer.info().size, size_of_val(data)) {
                            if let Some(mappable_buffer) = buffer.try_as_mappable() {
                                device
                                    .upload_to_memory(mappable_buffer, offset, data)
                                    .expect("Map failed");

                                break &*buffer;
                            }
                        }
                    }
                }
            };

            self.copy_buffer(
                &staging,
                buffer,
                &[BufferCopy {
                    src_offset: 0,
                    dst_offset: offset,
                    size: size_of_val(data) as u64,
                }],
            );
        }

        Ok(())
    }

    /// Builds acceleration structures.
    pub fn build_acceleration_structure(
        &mut self,
        infos: &[AccelerationStructureBuildGeometryInfo],
    ) {
        assert!(self.inner.capabilities.supports_compute());

        if infos.is_empty() {
            return;
        }

        // Checks.
        for (i, info) in infos.iter().enumerate() {
            if let Some(src) = info.src {
                for (j, info) in infos[..i].iter().enumerate() {
                    assert_ne!(
                        info.dst, src,
                        "`infos[{}].src` and `infos[{}].dst` collision",
                        i, j,
                    );
                }
            }

            for (j, info) in infos[..i].iter().enumerate() {
                assert_ne!(
                    info.src,
                    Some(info.dst),
                    "`infos[{}].src` and `infos[{}].dst` collision",
                    j,
                    i,
                );
            }
        }

        self.inner.command_buffer.write(
            self.inner.scope,
            Command::BuildAccelerationStructure { infos },
        )
    }

    pub fn trace_rays(&mut self, shader_binding_table: &'a ShaderBindingTable, extent: Extent3d) {
        assert!(self.inner.capabilities.supports_compute());

        self.inner.command_buffer.write(
            self.inner.scope,
            Command::TraceRays {
                shader_binding_table,
                extent,
            },
        )
    }

    pub fn copy_buffer(
        &mut self,
        src_buffer: &Buffer,
        dst_buffer: &Buffer,
        regions: &[BufferCopy],
    ) {
        #[cfg(debug_assertions)]
        {
            for region in regions {
                assert!(src_buffer.info().size >= region.src_offset + region.size);
                assert!(dst_buffer.info().size >= region.dst_offset + region.size);
            }
        }

        self.inner.command_buffer.write(
            self.inner.scope,
            Command::CopyBuffer {
                src_buffer,
                dst_buffer,
                regions,
            },
        )
    }

    pub fn copy_image(
        &mut self,
        src_image: &Image,
        src_layout: Layout,
        dst_image: &Image,
        dst_layout: Layout,
        regions: &[ImageCopy],
    ) {
        self.inner.command_buffer.write(
            self.inner.scope,
            Command::CopyImage {
                src_image,
                src_layout,
                dst_image,
                dst_layout,
                regions,
            },
        )
    }

    pub fn copy_buffer_to_image(
        &mut self,
        src_buffer: &Buffer,
        dst_image: &Image,
        dst_layout: Layout,
        regions: &[BufferImageCopy],
    ) {
        self.inner.command_buffer.write(
            self.inner.scope,
            Command::CopyBufferImage {
                src_buffer,
                dst_image,
                dst_layout,
                regions,
            },
        )
    }

    pub fn blit_image(
        &mut self,
        src_image: &Image,
        src_layout: Layout,
        dst_image: &Image,
        dst_layout: Layout,
        regions: &[ImageBlit],
        filter: Filter,
    ) {
        assert!(self.inner.capabilities.supports_graphics());

        self.inner.command_buffer.write(
            self.inner.scope,
            Command::BlitImage {
                src_image,
                src_layout,
                dst_image,
                dst_layout,
                regions,
                filter,
            },
        )
    }

    pub fn dispatch(&mut self, x: u32, y: u32, z: u32) {
        assert!(self.inner.capabilities.supports_compute());

        self.inner
            .command_buffer
            .write(self.inner.scope, Command::Dispatch { x, y, z });
    }

    pub fn memory_barrier(
        &mut self,
        src: PipelineStageFlags,
        src_acc: AccessFlags,
        dst: PipelineStageFlags,
        dst_acc: AccessFlags,
    ) {
        self.inner.command_buffer.write(
            self.inner.scope,
            Command::PipelineBarrier {
                src,
                dst,
                images: &[],
                buffers: &[],
                memory: Some(MemoryBarrier {
                    src: src_acc,
                    dst: dst_acc,
                }),
            },
        );
    }

    pub fn image_barriers(
        &mut self,
        src: PipelineStageFlags,
        dst: PipelineStageFlags,
        images: &[ImageMemoryBarrier],
    ) {
        self.inner.command_buffer.write(
            self.inner.scope,
            Command::PipelineBarrier {
                src,
                dst,
                images,
                buffers: &[],
                memory: None,
            },
        );
    }

    pub fn buffer_barriers(
        &mut self,
        src: PipelineStageFlags,
        dst: PipelineStageFlags,
        buffers: &[BufferMemoryBarrier],
    ) {
        self.inner.command_buffer.write(
            self.inner.scope,
            Command::PipelineBarrier {
                src,
                dst,
                images: &[],
                buffers,
                memory: None,
            },
        );
    }

    /// Flushes commands recorded into this encoder to the underlying command
    /// buffer.
    pub fn finish(mut self) -> CommandBuffer {
        forget(self.drop);

        self.inner
            .command_buffer
            .end()
            .expect("TODO: Handle command buffer writing error");

        self.inner.command_buffer
    }
}

/// Command encoder that can encode commands inside render pass.
pub struct RenderPassEncoder<'a, 'b> {
    framebuffer: &'b Framebuffer,
    render_pass: &'b RenderPass,
    subpass: u32,
    inner: &'a mut EncoderCommon<'b>,
}

impl<'a, 'b> fmt::Debug for RenderPassEncoder<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderPassEncoder")
            .field("framebuffer", self.framebuffer)
            .field("render_pass", self.render_pass)
            .field("subpass", &self.subpass)
            .field("command_buffer", &self.inner.command_buffer)
            .field("capabilities", &self.inner.capabilities)
            .finish()
    }
}

impl<'a, 'b> RenderPassEncoder<'a, 'b> {
    pub fn render_pass(&self) -> &RenderPass {
        self.render_pass
    }

    pub fn framebuffer(&self) -> &Framebuffer {
        self.framebuffer
    }

    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.inner.command_buffer.write(
            self.scope,
            Command::Draw {
                vertices,
                instances,
            },
        );
    }

    pub fn draw_indexed(&mut self, indices: Range<u32>, vertex_offset: i32, instances: Range<u32>) {
        self.inner.command_buffer.write(
            self.scope,
            Command::DrawIndexed {
                indices,
                vertex_offset,
                instances,
            },
        );
    }

    pub fn bind_dynamic_graphics_pipeline(
        &mut self,
        pipeline: &mut DynamicGraphicsPipeline,
        device: &Device,
    ) -> Result<(), OutOfMemory> {
        assert!(self.capabilities.supports_graphics());

        let mut set_viewport = false;
        let mut set_scissor = false;

        if let Some(rasterizer) = &pipeline.desc.rasterizer {
            set_viewport = rasterizer.viewport.is_dynamic();
            set_scissor = rasterizer.scissor.is_dynamic();
        }

        if set_scissor {
            self.inner
                .set_scissor(self.framebuffer.info().extent.into());
        }

        if set_viewport {
            self.inner
                .set_viewport(self.framebuffer.info().extent.into());
        }

        let gp = pipeline.get(self.render_pass, self.subpass, device)?;
        self.inner.bind_graphics_pipeline(gp);
        Ok(())
    }
}

impl Drop for RenderPassEncoder<'_, '_> {
    fn drop(&mut self) {
        self.inner
            .command_buffer
            .write(self.scope, Command::EndRenderPass);
    }
}

impl<'a, 'b> std::ops::Deref for RenderPassEncoder<'a, 'b> {
    type Target = EncoderCommon<'b>;

    fn deref(&self) -> &EncoderCommon<'b> {
        self.inner
    }
}

impl<'a, 'b> std::ops::DerefMut for RenderPassEncoder<'a, 'b> {
    fn deref_mut(&mut self) -> &mut EncoderCommon<'b> {
        self.inner
    }
}
