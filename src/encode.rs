pub use crate::backend::CommandBuffer;
use crate::{
    accel::AccelerationStructureBuildGeometryInfo,
    access::AccessFlags,
    arith_le,
    buffer::Buffer,
    descriptor::DescriptorSet,
    framebuffer::Framebuffer,
    image::{
        Image, ImageBlit, ImageMemoryBarrier, ImageSubresourceLayers, Layout,
    },
    memory::MemoryBarrier,
    pipeline::{
        ComputePipeline, GraphicsPipeline, PipelineLayout, RayTracingPipeline,
        ShaderBindingTable, Viewport,
    },
    queue::QueueCapabilityFlags,
    render_pass::{ClearValue, RenderPass},
    sampler::Filter,
    shader::ShaderStageFlags,
    stage::PipelineStageFlags,
    Extent3d, IndexType, Offset3d, Rect2d,
};
use bytemuck::{cast_slice, Pod};
use std::{fmt::Debug, mem::size_of_val, ops::Range};

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
    pub src_subresource: ImageSubresourceLayers,
    pub src_offset: Offset3d,
    pub dst_subresource: ImageSubresourceLayers,
    pub dst_offset: Offset3d,
    pub extent: Extent3d,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct BufferImageCopy {
    pub buffer_offset: u64,
    pub buffer_row_length: u32,
    pub buffer_image_height: u32,
    pub image_subresource: ImageSubresourceLayers,
    pub image_offset: Offset3d,
    pub image_extent: Extent3d,
}

#[derive(Debug)]
pub enum Command<'a> {
    BeginRenderPass {
        pass: &'a RenderPass,
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
        sets: &'a [DescriptorSet],
        dynamic_offsets: &'a [u32],
    },

    BindComputeDescriptorSets {
        layout: &'a PipelineLayout,
        first_set: u32,
        sets: &'a [DescriptorSet],
        dynamic_offsets: &'a [u32],
    },

    BindRayTracingDescriptorSets {
        layout: &'a PipelineLayout,
        first_set: u32,
        sets: &'a [DescriptorSet],
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
        buffers: &'a [(Buffer, u64)],
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
#[derive(Debug)]
pub struct EncoderCommon<'a> {
    capabilities: QueueCapabilityFlags,
    commands: Vec<Command<'a>>,
}

impl<'a> EncoderCommon<'a> {
    pub fn set_viewport(&mut self, viewport: Viewport) {
        assert!(self.capabilities.supports_graphics());

        self.commands.push(Command::SetViewport { viewport })
    }

    pub fn set_scissor(&mut self, scissor: Rect2d) {
        assert!(self.capabilities.supports_graphics());

        self.commands.push(Command::SetScissor { scissor })
    }

    pub fn bind_graphics_pipeline(&mut self, pipeline: &'a GraphicsPipeline) {
        assert!(self.capabilities.supports_graphics());

        self.commands
            .push(Command::BindGraphicsPipeline { pipeline })
    }

    pub fn bind_compute_pipeline(&mut self, pipeline: &'a ComputePipeline) {
        assert!(self.capabilities.supports_compute());
        self.commands
            .push(Command::BindComputePipeline { pipeline })
    }

    pub fn bind_ray_tracing_pipeline(
        &mut self,
        pipeline: &'a RayTracingPipeline,
    ) {
        assert!(self.capabilities.supports_compute());

        self.commands
            .push(Command::BindRayTracingPipeline { pipeline })
    }

    pub fn bind_vertex_buffers(
        &mut self,
        first: u32,
        buffers: &'a [(Buffer, u64)],
    ) {
        assert!(self.capabilities.supports_graphics());

        self.commands
            .push(Command::BindVertexBuffers { first, buffers })
    }

    pub fn bind_index_buffer(
        &mut self,
        buffer: &'a Buffer,
        offset: u64,
        index_type: IndexType,
    ) {
        assert!(self.capabilities.supports_graphics());

        self.commands.push(Command::BindIndexBuffer {
            buffer,
            offset,
            index_type,
        })
    }

    pub fn bind_graphics_descriptor_sets(
        &mut self,
        layout: &'a PipelineLayout,
        first_set: u32,
        sets: &'a [DescriptorSet],
        dynamic_offsets: &'a [u32],
    ) {
        assert!(self.capabilities.supports_graphics());

        self.commands.push(Command::BindGraphicsDescriptorSets {
            layout,
            first_set,
            sets,
            dynamic_offsets,
        });
    }

    pub fn bind_compute_descriptor_sets(
        &mut self,
        layout: &'a PipelineLayout,
        first_set: u32,
        sets: &'a [DescriptorSet],
        dynamic_offsets: &'a [u32],
    ) {
        assert!(self.capabilities.supports_compute());

        self.commands.push(Command::BindComputeDescriptorSets {
            layout,
            first_set,
            sets,
            dynamic_offsets,
        });
    }

    pub fn bind_ray_tracing_descriptor_sets(
        &mut self,
        layout: &'a PipelineLayout,
        first_set: u32,
        sets: &'a [DescriptorSet],
        dynamic_offsets: &'a [u32],
    ) {
        assert!(self.capabilities.supports_compute());

        self.commands.push(Command::BindRayTracingDescriptorSets {
            layout,
            first_set,
            sets,
            dynamic_offsets,
        });
    }

    pub fn memory_barrier(
        &mut self,
        src: PipelineStageFlags,
        src_acc: AccessFlags,
        dst: PipelineStageFlags,
        dst_acc: AccessFlags,
    ) {
        self.commands.push(Command::PipelineBarrier {
            src,
            dst,
            images: &[],
            memory: Some(MemoryBarrier {
                src: src_acc,
                dst: dst_acc,
            }),
        });
    }

    pub fn image_barriers(
        &mut self,
        src: PipelineStageFlags,
        dst: PipelineStageFlags,
        images: &'a [ImageMemoryBarrier<'a>],
    ) {
        self.commands.push(Command::PipelineBarrier {
            src,
            dst,
            images,
            memory: None,
        });
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

        self.commands.push(Command::PushConstants {
            layout,
            stages,
            offset,
            data: cast_slice(data),
        });
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
    ) -> Self {
        Encoder {
            inner: EncoderCommon {
                capabilities,
                commands: Vec::new(),
            },
            command_buffer,
        }
    }

    /// Begins render pass and returns `RenderPassEncoder` to encode commands of
    /// the render pass. `RenderPassEncoder` borrows `Encoder`.
    /// To continue use this `Encoder` returned `RenderPassEncoder` must be
    /// dropped which implicitly ends render pass.
    ///
    /// `pass` - render pass to encode.
    /// `framebuffer` - a framebuffer (set of attachments) for render pass to
    /// use. `clears` - an array of clear values.
    ///            render pass will clear attachments with `load_op ==
    /// AttachmentLoadOp::Clear` using those values.            they will be
    /// used in order.

    pub fn with_render_pass(
        &mut self,
        pass: &'a RenderPass,
        framebuffer: &'a Framebuffer,
        clears: &'a [ClearValue],
    ) -> RenderPassEncoder<'_, 'a> {
        assert!(self.inner.capabilities.supports_graphics());

        self.inner.commands.push(Command::BeginRenderPass {
            pass,
            framebuffer,
            clears,
        });

        RenderPassEncoder {
            inner: &mut self.inner,
        }
    }

    /// Updates a buffer's contents from host memory

    pub fn update_buffer<T>(
        &mut self,
        buffer: &'a Buffer,
        offset: u64,
        data: &'a [T],
    ) where
        T: Pod,
    {
        let data = unsafe {
            std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                std::mem::size_of_val(data),
            )
        };

        self.inner.commands.push(Command::UpdateBuffer {
            buffer,
            offset,
            data,
        })
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
            if let Some(src) = &info.src {
                for (j, info) in infos[..i].iter().enumerate() {
                    assert_ne!(
                        &info.dst, src,
                        "`infos[{}].src` and `infos[{}].dst` collision",
                        i, j,
                    );
                }
            }

            for (j, info) in infos[..i].iter().enumerate() {
                assert_ne!(
                    info.src.as_ref(),
                    Some(&info.dst),
                    "`infos[{}].src` and `infos[{}].dst` collision",
                    j,
                    i,
                );
            }

            // assert!(todo!());
        }

        self.inner
            .commands
            .push(Command::BuildAccelerationStructure { infos })
    }

    pub fn trace_rays(
        &mut self,
        shader_binding_table: &'a ShaderBindingTable,
        extent: Extent3d,
    ) {
        assert!(self.inner.capabilities.supports_compute());

        self.commands.push(Command::TraceRays {
            shader_binding_table,
            extent,
        })
    }

    pub fn copy_buffer(
        &mut self,
        src_buffer: &'a Buffer,
        dst_buffer: &'a Buffer,
        regions: &'a [BufferCopy],
    ) {
        self.commands.push(Command::CopyBuffer {
            src_buffer,
            dst_buffer,
            regions,
        })
    }

    pub fn copy_image(
        &mut self,
        src_image: &'a Image,
        src_layout: Layout,
        dst_image: &'a Image,
        dst_layout: Layout,
        regions: &'a [ImageCopy],
    ) {
        self.commands.push(Command::CopyImage {
            src_image,
            src_layout,
            dst_image,
            dst_layout,
            regions,
        })
    }

    pub fn copy_buffer_to_image(
        &mut self,
        src_buffer: &'a Buffer,
        dst_image: &'a Image,
        dst_layout: Layout,
        regions: &'a [BufferImageCopy],
    ) {
        self.commands.push(Command::CopyBufferImage {
            src_buffer,
            dst_image,
            dst_layout,
            regions,
        })
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
        assert!(self.capabilities.supports_graphics());

        self.commands.push(Command::BlitImage {
            src_image,
            src_layout,
            dst_image,
            dst_layout,
            regions,
            filter,
        })
    }

    pub fn dispatch(&mut self, x: u32, y: u32, z: u32) {
        assert!(self.capabilities.supports_compute());

        self.commands.push(Command::Dispatch { x, y, z });
    }

    /// Flushes commands recorded into this encoder to the underlying command
    /// buffer.
    pub fn finish(mut self) -> CommandBuffer {
        self.command_buffer
            .write(&self.inner.commands)
            .expect("TODO: Handle command buffer writing error");

        self.command_buffer
    }
}

/// Command encoder that can encode commands inside render pass.
#[derive(Debug)]

pub struct RenderPassEncoder<'a, 'b> {
    inner: &'a mut EncoderCommon<'b>,
}

impl<'a, 'b> RenderPassEncoder<'a, 'b> {
    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.inner.commands.push(Command::Draw {
            vertices,
            instances,
        });
    }

    pub fn draw_indexed(
        &mut self,
        indices: Range<u32>,
        vertex_offset: i32,
        instances: Range<u32>,
    ) {
        self.inner.commands.push(Command::DrawIndexed {
            indices,
            vertex_offset,
            instances,
        });
    }
}

impl Drop for RenderPassEncoder<'_, '_> {
    fn drop(&mut self) {
        self.inner.commands.push(Command::EndRenderPass);
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
