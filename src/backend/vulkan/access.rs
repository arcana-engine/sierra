use erupt::vk1_0;

use crate::Access;

fn supported_stages_inner(access: vk1_0::AccessFlags) -> vk1_0::PipelineStageFlags {
    type AF = vk1_0::AccessFlags;
    type PS = vk1_0::PipelineStageFlags;

    match access {
        AF::INDIRECT_COMMAND_READ => PS::DRAW_INDIRECT,
        AF::INDEX_READ => PS::VERTEX_INPUT,
        AF::VERTEX_ATTRIBUTE_READ => PS::VERTEX_INPUT,
        AF::UNIFORM_READ => {
            PS::TASK_SHADER_NV
                | PS::MESH_SHADER_NV
                | PS::RAY_TRACING_SHADER_NV
                | PS::RAY_TRACING_SHADER_KHR
                | PS::VERTEX_SHADER
                | PS::TESSELLATION_CONTROL_SHADER
                | PS::TESSELLATION_EVALUATION_SHADER
                | PS::GEOMETRY_SHADER
                | PS::FRAGMENT_SHADER
                | PS::COMPUTE_SHADER
        }
        AF::SHADER_READ | AF::SHADER_WRITE => {
            PS::TASK_SHADER_NV
                | PS::MESH_SHADER_NV
                | PS::RAY_TRACING_SHADER_KHR
                | PS::VERTEX_SHADER
                | PS::TESSELLATION_CONTROL_SHADER
                | PS::TESSELLATION_EVALUATION_SHADER
                | PS::GEOMETRY_SHADER
                | PS::FRAGMENT_SHADER
                | PS::COMPUTE_SHADER
        }
        AF::INPUT_ATTACHMENT_READ => PS::FRAGMENT_SHADER,
        AF::COLOR_ATTACHMENT_READ | AF::COLOR_ATTACHMENT_WRITE => PS::COLOR_ATTACHMENT_OUTPUT,
        AF::DEPTH_STENCIL_ATTACHMENT_READ | AF::DEPTH_STENCIL_ATTACHMENT_WRITE => {
            PS::EARLY_FRAGMENT_TESTS | PS::LATE_FRAGMENT_TESTS
        }
        AF::TRANSFER_READ | AF::TRANSFER_WRITE => PS::TRANSFER,
        AF::HOST_READ | AF::HOST_WRITE => PS::HOST,
        AF::MEMORY_READ | AF::MEMORY_WRITE => PS::from_bits_truncate(!0),
        AF::ACCELERATION_STRUCTURE_READ_KHR | AF::ACCELERATION_STRUCTURE_WRITE_KHR => {
            PS::ACCELERATION_STRUCTURE_BUILD_KHR
        }
        _ if access.bits().count_ones() != 1 => {
            panic!("Only one-bit access flags must be supplied")
        }
        _ => PS::empty(),
    }
}

pub(crate) fn supported_access(stages: vk1_0::PipelineStageFlags) -> vk1_0::AccessFlags {
    let mut result = vk1_0::AccessFlags::empty();

    let mut bits: vk1_0::Flags = !0;

    while bits != 0 {
        let bit = 1 << bits.trailing_zeros();

        bits &= !bit;

        if let Some(flag) = vk1_0::AccessFlags::from_bits(bit) {
            if supported_stages_inner(flag).intersects(stages) {
                result |= flag;
            }
        }
    }

    result
}

pub(crate) struct AccessInfo {
	pub(crate) stage_mask: vk1_0::PipelineStageFlags,
	pub(crate) access_mask: vk1_0::AccessFlags,
	pub(crate) image_layout: vk1_0::ImageLayout,
}

pub(crate) trait GetAccessInfo {
    fn access_info(self) -> AccessInfo;
}

impl GetAccessInfo for Access {
	fn access_info(self) -> AccessInfo {
        match self {
            Access::None => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::empty(),
                access_mask: vk1_0::AccessFlags::empty(),
                image_layout: vk1_0::ImageLayout::UNDEFINED,
            },
            Access::IndirectBuffer => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::DRAW_INDIRECT,
                access_mask: vk1_0::AccessFlags::INDIRECT_COMMAND_READ,
                image_layout: vk1_0::ImageLayout::UNDEFINED,
            },
            Access::IndexBuffer => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::VERTEX_INPUT,
                access_mask: vk1_0::AccessFlags::INDEX_READ,
                image_layout: vk1_0::ImageLayout::UNDEFINED,
            },
            Access::VertexBuffer => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::VERTEX_INPUT,
                access_mask: vk1_0::AccessFlags::VERTEX_ATTRIBUTE_READ,
                image_layout: vk1_0::ImageLayout::UNDEFINED,
            },
            Access::VertexShaderReadUniformBuffer => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::VERTEX_SHADER,
                access_mask: vk1_0::AccessFlags::SHADER_READ,
                image_layout: vk1_0::ImageLayout::UNDEFINED,
            },
            Access::VertexShaderReadSampledImageOrUniformTexelBuffer => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::VERTEX_SHADER,
                access_mask: vk1_0::AccessFlags::SHADER_READ,
                image_layout: vk1_0::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            },
            Access::VertexShaderReadOther => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::VERTEX_SHADER,
                access_mask: vk1_0::AccessFlags::SHADER_READ,
                image_layout: vk1_0::ImageLayout::GENERAL,
            },
            Access::FragmentShaderReadUniformBuffer => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::FRAGMENT_SHADER,
                access_mask: vk1_0::AccessFlags::UNIFORM_READ,
                image_layout: vk1_0::ImageLayout::UNDEFINED,
            },
            Access::FragmentShaderReadSampledImageOrUniformTexelBuffer => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::FRAGMENT_SHADER,
                access_mask: vk1_0::AccessFlags::SHADER_READ,
                image_layout: vk1_0::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            },
            Access::FragmentShaderReadColorInputAttachment => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::FRAGMENT_SHADER,
                access_mask: vk1_0::AccessFlags::INPUT_ATTACHMENT_READ,
                image_layout: vk1_0::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            },
            Access::FragmentShaderReadDepthStencilInputAttachment => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::FRAGMENT_SHADER,
                access_mask: vk1_0::AccessFlags::INPUT_ATTACHMENT_READ,
                image_layout: vk1_0::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
            },
            Access::FragmentShaderReadOther => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::FRAGMENT_SHADER,
                access_mask: vk1_0::AccessFlags::SHADER_READ,
                image_layout: vk1_0::ImageLayout::GENERAL,
            },
            Access::ColorAttachmentRead => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                access_mask: vk1_0::AccessFlags::COLOR_ATTACHMENT_READ,
                image_layout: vk1_0::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            },
            Access::DepthStencilAttachmentRead => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                    | vk1_0::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                access_mask: vk1_0::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                image_layout: vk1_0::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
            },
            Access::ComputeShaderReadUniformBuffer => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::COMPUTE_SHADER,
                access_mask: vk1_0::AccessFlags::UNIFORM_READ,
                image_layout: vk1_0::ImageLayout::UNDEFINED,
            },
            Access::ComputeShaderReadSampledImageOrUniformTexelBuffer => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::COMPUTE_SHADER,
                access_mask: vk1_0::AccessFlags::SHADER_READ,
                image_layout: vk1_0::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            },
            Access::ComputeShaderReadOther => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::COMPUTE_SHADER,
                access_mask: vk1_0::AccessFlags::SHADER_READ,
                image_layout: vk1_0::ImageLayout::GENERAL,
            },
            Access::AnyShaderReadUniformBuffer => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::ALL_COMMANDS,
                access_mask: vk1_0::AccessFlags::UNIFORM_READ,
                image_layout: vk1_0::ImageLayout::UNDEFINED,
            },
            Access::AnyShaderReadUniformBufferOrVertexBuffer => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::ALL_COMMANDS,
                access_mask: vk1_0::AccessFlags::UNIFORM_READ
                    | vk1_0::AccessFlags::VERTEX_ATTRIBUTE_READ,
                image_layout: vk1_0::ImageLayout::UNDEFINED,
            },
            Access::AnyShaderReadSampledImageOrUniformTexelBuffer => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::ALL_COMMANDS,
                access_mask: vk1_0::AccessFlags::SHADER_READ,
                image_layout: vk1_0::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            },
            Access::AnyShaderReadOther => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::ALL_COMMANDS,
                access_mask: vk1_0::AccessFlags::SHADER_READ,
                image_layout: vk1_0::ImageLayout::GENERAL,
            },
            Access::TransferRead => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::TRANSFER,
                access_mask: vk1_0::AccessFlags::TRANSFER_READ,
                image_layout: vk1_0::ImageLayout::TRANSFER_SRC_OPTIMAL,
            },
            Access::HostRead => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::HOST,
                access_mask: vk1_0::AccessFlags::HOST_READ,
                image_layout: vk1_0::ImageLayout::GENERAL,
            },
            Access::Present => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::empty(),
                access_mask: vk1_0::AccessFlags::empty(),
                image_layout: vk1_0::ImageLayout::PRESENT_SRC_KHR,
            },
            Access::VertexShaderWrite => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::VERTEX_SHADER,
                access_mask: vk1_0::AccessFlags::SHADER_WRITE,
                image_layout: vk1_0::ImageLayout::GENERAL,
            },
            Access::FragmentShaderWrite => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::FRAGMENT_SHADER,
                access_mask: vk1_0::AccessFlags::SHADER_WRITE,
                image_layout: vk1_0::ImageLayout::GENERAL,
            },
            Access::ColorAttachmentWrite => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                access_mask: vk1_0::AccessFlags::COLOR_ATTACHMENT_WRITE,
                image_layout: vk1_0::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            },
            Access::DepthStencilAttachmentWrite => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                    | vk1_0::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                access_mask: vk1_0::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                image_layout: vk1_0::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            },
            Access::DepthAttachmentWriteStencilReadOnly => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                    | vk1_0::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                access_mask: vk1_0::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
                    | vk1_0::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                image_layout: vk1_0::ImageLayout::DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL,
            },
            Access::StencilAttachmentWriteDepthReadOnly => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                    | vk1_0::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                access_mask: vk1_0::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
                    | vk1_0::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                image_layout: vk1_0::ImageLayout::DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL,
            },
            Access::ComputeShaderWrite => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::COMPUTE_SHADER,
                access_mask: vk1_0::AccessFlags::SHADER_WRITE,
                image_layout: vk1_0::ImageLayout::GENERAL,
            },
            Access::AnyShaderWrite => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::ALL_COMMANDS,
                access_mask: vk1_0::AccessFlags::SHADER_WRITE,
                image_layout: vk1_0::ImageLayout::GENERAL,
            },
            Access::TransferWrite => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::TRANSFER,
                access_mask: vk1_0::AccessFlags::TRANSFER_WRITE,
                image_layout: vk1_0::ImageLayout::TRANSFER_DST_OPTIMAL,
            },
            Access::HostWrite => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::HOST,
                access_mask: vk1_0::AccessFlags::HOST_WRITE,
                image_layout: vk1_0::ImageLayout::GENERAL,
            },
            Access::ColorAttachmentReadWrite => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                access_mask: vk1_0::AccessFlags::COLOR_ATTACHMENT_READ
                    | vk1_0::AccessFlags::COLOR_ATTACHMENT_WRITE,
                image_layout: vk1_0::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            },
            Access::General => AccessInfo {
                stage_mask: vk1_0::PipelineStageFlags::ALL_COMMANDS,
                access_mask: vk1_0::AccessFlags::MEMORY_READ | vk1_0::AccessFlags::MEMORY_WRITE,
                image_layout: vk1_0::ImageLayout::GENERAL,
            },
        }
	}
}