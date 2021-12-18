pub use crate::backend::CommandBuffer;
use {
    crate::{
        accel::AccelerationStructureBuildGeometryInfo,
        access::Access,
        arith_le,
        buffer::{Buffer, BufferMemoryBarrier},
        descriptor::{DescriptorSet, UpdatedPipelineDescriptors},
        framebuffer::{Framebuffer, FramebufferError},
        image::{Image, ImageBlit, ImageMemoryBarrier, Layout, SubresourceLayers},
        memory::GlobalMemoryBarrier,
        pipeline::{
            ComputePipeline, DynamicGraphicsPipeline, GraphicsPipeline, PipelineLayout,
            RayTracingPipeline, ShaderBindingTable, TypedPipelineLayout, Viewport,
        },
        queue::QueueCapabilityFlags,
        render_pass::{ClearValue, RenderPass, RenderPassInstance},
        sampler::Filter,
        shader::ShaderStageFlags,
        Device, Extent3d, IndexType, Offset3d, OutOfMemory, Rect2d,
    },
    arrayvec::ArrayVec,
    bytemuck::{cast_slice, Pod},
    scoped_arena::Scope,
    std::{fmt::Debug, mem::size_of_val, ops::Range},
};

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
        global: Option<GlobalMemoryBarrier<'a>>,
        images: &'a [ImageMemoryBarrier<'a>],
        buffers: &'a [BufferMemoryBarrier<'a>],
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

#[derive(Debug)]
struct Commands<'a> {
    buckets: &'a mut ArrayVec<&'a mut ArrayVec<Command<'a>, 1024>, 1024>,
}

impl<'a> Commands<'a> {
    pub fn new(scope: &'a Scope<'a>) -> Self {
        Commands {
            buckets: scope.to_scope_with(ArrayVec::new),
        }
    }

    pub fn push(&mut self, scope: &'a Scope<'a>, mut command: Command<'a>) {
        if let Some(last) = self.buckets.last_mut() {
            match last.try_push(command) {
                Ok(()) => return,
                Err(err) => command = err.element(),
            }
        }

        let bucket = scope.to_scope_with(ArrayVec::new);
        bucket.push(command);
        self.buckets.push(bucket);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = Command<'a>> + '_ {
        self.buckets.drain(..).flat_map(|bucket| bucket.drain(..))
    }
}

/// Basis for encoding capabilities.
/// Implements encoding of commands that can be inside and outside of render
/// pass.
#[derive(Debug)]
pub struct EncoderCommon<'a> {
    capabilities: QueueCapabilityFlags,
    commands: Commands<'a>,
    scope: &'a Scope<'a>,
}

impl<'a> EncoderCommon<'a> {
    pub fn scope(&self) -> &'a Scope<'a> {
        self.scope
    }

    pub fn set_viewport(&mut self, viewport: Viewport) {
        assert!(self.capabilities.supports_graphics());

        self.commands
            .push(self.scope, Command::SetViewport { viewport })
    }

    pub fn set_scissor(&mut self, scissor: Rect2d) {
        assert!(self.capabilities.supports_graphics());

        self.commands
            .push(self.scope, Command::SetScissor { scissor })
    }

    pub fn bind_graphics_pipeline(&mut self, pipeline: &'a GraphicsPipeline) {
        assert!(self.capabilities.supports_graphics());

        self.commands
            .push(self.scope, Command::BindGraphicsPipeline { pipeline })
    }

    pub fn bind_compute_pipeline(&mut self, pipeline: &'a ComputePipeline) {
        assert!(self.capabilities.supports_compute());
        self.commands
            .push(self.scope, Command::BindComputePipeline { pipeline })
    }

    pub fn bind_ray_tracing_pipeline(&mut self, pipeline: &'a RayTracingPipeline) {
        assert!(self.capabilities.supports_compute());

        self.commands
            .push(self.scope, Command::BindRayTracingPipeline { pipeline })
    }

    pub fn bind_vertex_buffers(&mut self, first: u32, buffers: &'a [(&'a Buffer, u64)]) {
        assert!(self.capabilities.supports_graphics());

        self.commands
            .push(self.scope, Command::BindVertexBuffers { first, buffers })
    }

    pub fn bind_index_buffer(&mut self, buffer: &'a Buffer, offset: u64, index_type: IndexType) {
        assert!(self.capabilities.supports_graphics());

        self.commands.push(
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
        layout: &'a PipelineLayout,
        first_set: u32,
        sets: &'a [&'a DescriptorSet],
        dynamic_offsets: &'a [u32],
    ) {
        assert!(self.capabilities.supports_graphics());

        self.commands.push(
            self.scope,
            Command::BindGraphicsDescriptorSets {
                layout,
                first_set,
                sets,
                dynamic_offsets,
            },
        );
    }

    pub fn bind_graphics_descriptors<P>(
        &mut self,
        layout: &'a P,
        descriptors: &impl UpdatedPipelineDescriptors<P>,
    ) where
        P: TypedPipelineLayout,
    {
        layout.bind_graphics(descriptors, self);
    }

    pub fn bind_compute_descriptor_sets(
        &mut self,
        layout: &'a PipelineLayout,
        first_set: u32,
        sets: &'a [&'a DescriptorSet],
        dynamic_offsets: &'a [u32],
    ) {
        assert!(self.capabilities.supports_compute());

        self.commands.push(
            self.scope,
            Command::BindComputeDescriptorSets {
                layout,
                first_set,
                sets,
                dynamic_offsets,
            },
        );
    }

    pub fn bind_compute_descriptors<P>(
        &mut self,
        layout: &'a P,
        descriptors: &impl UpdatedPipelineDescriptors<P>,
    ) where
        P: TypedPipelineLayout,
    {
        layout.bind_compute(descriptors, self);
    }

    pub fn bind_ray_tracing_descriptor_sets(
        &mut self,
        layout: &'a PipelineLayout,
        first_set: u32,
        sets: &'a [&'a DescriptorSet],
        dynamic_offsets: &'a [u32],
    ) {
        assert!(self.capabilities.supports_compute());

        self.commands.push(
            self.scope,
            Command::BindRayTracingDescriptorSets {
                layout,
                first_set,
                sets,
                dynamic_offsets,
            },
        );
    }

    pub fn bind_ray_tracing_descriptors<P>(
        &mut self,
        layout: &'a P,
        descriptors: &impl UpdatedPipelineDescriptors<P>,
    ) where
        P: TypedPipelineLayout,
    {
        layout.bind_ray_tracing(descriptors, self);
    }

    pub fn push_constants<T>(
        &mut self,
        layout: &'a PipelineLayout,
        stages: ShaderStageFlags,
        offset: u32,
        data: &'a [T],
    ) where
        T: Pod,
    {
        assert!(arith_le(size_of_val(data), u32::max_value()));

        self.commands.push(
            self.scope,
            Command::PushConstants {
                layout,
                stages,
                offset,
                data: cast_slice(data),
            },
        );
    }
}

/// Command encoder that can encode commands outside render pass.
#[derive(Debug)]
pub struct Encoder<'a> {
    inner: EncoderCommon<'a>,
    command_buffer: CommandBuffer,
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
                commands: Commands::new(scope),
                scope,
            },
            command_buffer,
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
        clears: &'a [ClearValue],
    ) -> RenderPassEncoder<'_, 'a> {
        assert!(self.inner.capabilities.supports_graphics());

        self.inner.commands.push(
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
    pub fn update_buffer<T>(&mut self, buffer: &'a Buffer, offset: u64, data: &'a [T])
    where
        T: Pod,
    {
        assert_eq!(offset % 4, 0);
        assert!(size_of_val(data) <= 65_536, "Data length greater than 65536 MUST NOT be uploaded with encoder, consider buffer mapping. Actual data is {} bytes", data.len());

        if data.is_empty() {
            return;
        }

        let data = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data))
        };

        self.inner.commands.push(
            self.scope,
            Command::UpdateBuffer {
                buffer,
                offset,
                data,
            },
        )
    }

    /// Builds acceleration structures.
    pub fn build_acceleration_structure(
        &mut self,
        infos: &'a [AccelerationStructureBuildGeometryInfo<'a>],
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

        self.inner.commands.push(
            self.inner.scope,
            Command::BuildAccelerationStructure { infos },
        )
    }

    pub fn trace_rays(&mut self, shader_binding_table: &'a ShaderBindingTable, extent: Extent3d) {
        assert!(self.inner.capabilities.supports_compute());

        self.inner.commands.push(
            self.inner.scope,
            Command::TraceRays {
                shader_binding_table,
                extent,
            },
        )
    }

    pub fn copy_buffer(
        &mut self,
        src_buffer: &'a Buffer,
        dst_buffer: &'a Buffer,
        regions: &'a [BufferCopy],
    ) {
        #[cfg(debug_assertions)]
        {
            for region in regions {
                assert!(src_buffer.info().size >= region.src_offset + region.size);
                assert!(dst_buffer.info().size >= region.dst_offset + region.size);
            }
        }

        self.inner.commands.push(
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
        src_image: &'a Image,
        src_layout: Layout,
        dst_image: &'a Image,
        dst_layout: Layout,
        regions: &'a [ImageCopy],
    ) {
        self.inner.commands.push(
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
        src_buffer: &'a Buffer,
        dst_image: &'a Image,
        dst_layout: Layout,
        regions: &'a [BufferImageCopy],
    ) {
        self.inner.commands.push(
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
        src_image: &'a Image,
        src_layout: Layout,
        dst_image: &'a Image,
        dst_layout: Layout,
        regions: &'a [ImageBlit],
        filter: Filter,
    ) {
        assert!(self.inner.capabilities.supports_graphics());

        self.inner.commands.push(
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
            .commands
            .push(self.inner.scope, Command::Dispatch { x, y, z });
    }

    pub fn pipeline_barrier(
        &mut self,
        global: Option<GlobalMemoryBarrier<'a>>,
        images: &'a [ImageMemoryBarrier<'a>],
        buffers: &'a [BufferMemoryBarrier<'a>],
    ) {
        self.inner.commands.push(
            self.inner.scope,
            Command::PipelineBarrier {
                images,
                buffers,
                global,
            }
        )
    }

    pub fn global_barrier(
        &mut self,
        prev_accesses: &'a [Access],
        next_accesses: &'a [Access],
    ) {
        self.inner.commands.push(
            self.inner.scope,
            Command::PipelineBarrier {
                images: &[],
                buffers: &[],
                global: Some(GlobalMemoryBarrier {
                    prev_accesses,
                    next_accesses,
                }),
            },
        );
    }

    pub fn image_barriers(
        &mut self,
        images: &'a [ImageMemoryBarrier<'a>],
    ) {
        self.inner.commands.push(
            self.inner.scope,
            Command::PipelineBarrier {
                images,
                buffers: &[],
                global: None,
            },
        );
    }

    pub fn buffer_barriers(
        &mut self,
        buffers: &'a [BufferMemoryBarrier<'a>],
    ) {
        self.inner.commands.push(
            self.inner.scope,
            Command::PipelineBarrier {
                images: &[],
                buffers,
                global: None,
            },
        );
    }

    /// Flushes commands recorded into this encoder to the underlying command
    /// buffer.
    pub fn finish(mut self) -> CommandBuffer {
        self.command_buffer
            .write(self.inner.commands.drain(), self.inner.scope)
            .expect("TODO: Handle command buffer writing error");

        self.command_buffer
    }
}

/// Command encoder that can encode commands inside render pass.
#[derive(Debug)]
pub struct RenderPassEncoder<'a, 'b> {
    framebuffer: &'b Framebuffer,
    render_pass: &'b RenderPass,
    subpass: u32,
    inner: &'a mut EncoderCommon<'b>,
}

impl<'a, 'b> RenderPassEncoder<'a, 'b> {
    pub fn render_pass(&self) -> &RenderPass {
        self.render_pass
    }

    pub fn framebuffer(&self) -> &Framebuffer {
        self.framebuffer
    }

    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.inner.commands.push(
            self.scope,
            Command::Draw {
                vertices,
                instances,
            },
        );
    }

    pub fn draw_indexed(&mut self, indices: Range<u32>, vertex_offset: i32, instances: Range<u32>) {
        self.inner.commands.push(
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
        pipeline: &'b mut DynamicGraphicsPipeline,
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
        self.inner.commands.push(self.scope, Command::EndRenderPass);
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
