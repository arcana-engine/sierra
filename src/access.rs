use crate::PipelineStageFlags;

// Modified from vk-sync-rs, originally Copyright 2019 Graham Wihlidal
// licensed under MIT license.
//
// https://github.com/gwihlidal/vk-sync-rs/blob/master/LICENSE-MIT

/// Defines all potential resource usages
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Access {
    /// No access. Useful primarily for initialization
    None,

    /// Read as an indirect buffer for drawing or dispatch
    IndirectBuffer,

    /// Read as an index buffer for drawing
    IndexBuffer,

    /// Read as a vertex buffer for drawing
    VertexBuffer,

    /// Read as a uniform buffer in a vertex shader
    VertexShaderReadUniformBuffer,

    /// Read as a sampled image/uniform texel buffer in a vertex shader
    VertexShaderReadSampledImageOrUniformTexelBuffer,

    /// Read as any other resource in a vertex shader
    VertexShaderReadOther,

    /// Read as a uniform buffer in a fragment shader
    FragmentShaderReadUniformBuffer,

    /// Read as a sampled image/uniform texel buffer in a fragment shader
    FragmentShaderReadSampledImageOrUniformTexelBuffer,

    /// Read as an input attachment with a color format in a fragment shader
    FragmentShaderReadColorInputAttachment,

    /// Read as an input attachment with a depth/stencil format in a fragment shader
    FragmentShaderReadDepthStencilInputAttachment,

    /// Read as any other resource in a fragment shader
    FragmentShaderReadOther,

    /// Read by blending/logic operations or subpass load operations
    ColorAttachmentRead,

    /// Read by depth/stencil tests or subpass load operations
    DepthStencilAttachmentRead,

    /// Read as a uniform buffer in a compute shader
    ComputeShaderReadUniformBuffer,

    /// Read as a sampled image/uniform texel buffer in a compute shader
    ComputeShaderReadSampledImageOrUniformTexelBuffer,

    /// Read as any other resource in a compute shader
    ComputeShaderReadOther,

    /// Read as a uniform buffer in any shader
    AnyShaderReadUniformBuffer,

    /// Read as a uniform buffer in any shader, or a vertex buffer
    AnyShaderReadUniformBufferOrVertexBuffer,

    /// Read as a sampled image in any shader
    AnyShaderReadSampledImageOrUniformTexelBuffer,

    /// Read as any other resource (excluding attachments) in any shader
    AnyShaderReadOther,

    /// Read as the source of a transfer operation
    TransferRead,

    /// Read on the host
    HostRead,

    /// Read by the presentation engine (i.e. `vkQueuePresentKHR`)
    Present,

    /// Written as any resource in a vertex shader
    VertexShaderWrite,

    /// Written as any resource in a fragment shader
    FragmentShaderWrite,

    /// Written as a color attachment during rendering, or via a subpass store op
    ColorAttachmentWrite,

    /// Written as a depth/stencil attachment during rendering, or via a subpass store op
    DepthStencilAttachmentWrite,

    /// Written as a depth aspect of a depth/stencil attachment during rendering, whilst the
    /// stencil aspect is read-only. Requires `VK_KHR_maintenance2` to be enabled.
    DepthAttachmentWriteStencilReadOnly,

    /// Written as a stencil aspect of a depth/stencil attachment during rendering, whilst the
    /// depth aspect is read-only. Requires `VK_KHR_maintenance2` to be enabled.
    StencilAttachmentWriteDepthReadOnly,

    /// Written as any resource in a compute shader
    ComputeShaderWrite,

    /// Written as any resource in any shader
    AnyShaderWrite,

    /// Written as the destination of a transfer operation
    TransferWrite,

    /// Written on the host
    HostWrite,

    /// Read or written as a color attachment during rendering
    ColorAttachmentReadWrite,

    /// Covers any access - useful for debug, generally avoid for performance reasons
    General,
}

impl Default for Access {
    fn default() -> Self {
        Access::None
    }
}

impl Access {
    pub fn is_write(self) -> bool {
        match self {
            Access::VertexShaderWrite => true,
            Access::FragmentShaderWrite => true,
            Access::ColorAttachmentWrite => true,
            Access::DepthStencilAttachmentWrite => true,
            Access::DepthAttachmentWriteStencilReadOnly => true,
            Access::StencilAttachmentWriteDepthReadOnly => true,
            Access::ComputeShaderWrite => true,
            Access::AnyShaderWrite => true,
            Access::TransferWrite => true,
            Access::HostWrite => true,
            Access::ColorAttachmentReadWrite => true,
            Access::General => true,
            _ => false,
        }
    }

    pub fn is_read_only(self) -> bool {
        !self.is_write()
    }

    pub fn stage_flags(self) -> crate::PipelineStageFlags {
        match self {
            Access::None => PipelineStageFlags::empty(),
            Access::IndirectBuffer => PipelineStageFlags::DRAW_INDIRECT,
            Access::IndexBuffer => PipelineStageFlags::VERTEX_INPUT,
            Access::VertexBuffer => PipelineStageFlags::VERTEX_INPUT,
            Access::VertexShaderReadUniformBuffer => PipelineStageFlags::VERTEX_SHADER,
            Access::VertexShaderReadSampledImageOrUniformTexelBuffer => {
                PipelineStageFlags::VERTEX_SHADER
            }

            Access::VertexShaderReadOther => PipelineStageFlags::VERTEX_SHADER,

            Access::FragmentShaderReadUniformBuffer => PipelineStageFlags::FRAGMENT_SHADER,

            Access::FragmentShaderReadSampledImageOrUniformTexelBuffer => {
                PipelineStageFlags::FRAGMENT_SHADER
            }

            Access::FragmentShaderReadColorInputAttachment => PipelineStageFlags::FRAGMENT_SHADER,

            Access::FragmentShaderReadDepthStencilInputAttachment => {
                PipelineStageFlags::FRAGMENT_SHADER
            }

            Access::FragmentShaderReadOther => PipelineStageFlags::FRAGMENT_SHADER,

            Access::ColorAttachmentRead => PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,

            Access::DepthStencilAttachmentRead => {
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS
            }

            Access::ComputeShaderReadUniformBuffer => PipelineStageFlags::COMPUTE_SHADER,

            Access::ComputeShaderReadSampledImageOrUniformTexelBuffer => {
                PipelineStageFlags::COMPUTE_SHADER
            }

            Access::ComputeShaderReadOther => PipelineStageFlags::COMPUTE_SHADER,

            Access::AnyShaderReadUniformBuffer => PipelineStageFlags::ALL_COMMANDS,

            Access::AnyShaderReadUniformBufferOrVertexBuffer => PipelineStageFlags::ALL_COMMANDS,

            Access::AnyShaderReadSampledImageOrUniformTexelBuffer => {
                PipelineStageFlags::ALL_COMMANDS
            }

            Access::AnyShaderReadOther => PipelineStageFlags::ALL_COMMANDS,

            Access::TransferRead => PipelineStageFlags::TRANSFER,

            Access::HostRead => PipelineStageFlags::HOST,

            Access::Present => PipelineStageFlags::empty(),

            Access::VertexShaderWrite => PipelineStageFlags::VERTEX_SHADER,

            Access::FragmentShaderWrite => PipelineStageFlags::FRAGMENT_SHADER,

            Access::ColorAttachmentWrite => PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,

            Access::DepthStencilAttachmentWrite => {
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS
            }

            Access::DepthAttachmentWriteStencilReadOnly => {
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS
            }

            Access::StencilAttachmentWriteDepthReadOnly => {
                PipelineStageFlags::EARLY_FRAGMENT_TESTS | PipelineStageFlags::LATE_FRAGMENT_TESTS
            }

            Access::ComputeShaderWrite => PipelineStageFlags::COMPUTE_SHADER,

            Access::AnyShaderWrite => PipelineStageFlags::ALL_COMMANDS,

            Access::TransferWrite => PipelineStageFlags::TRANSFER,

            Access::HostWrite => PipelineStageFlags::HOST,

            Access::ColorAttachmentReadWrite => PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,

            Access::General => PipelineStageFlags::ALL_COMMANDS,
        }
    }
}
