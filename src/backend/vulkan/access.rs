use erupt::vk1_0;

// pub(crate) fn supported_stages(access: AccessFlags) ->
// PipelineStageFlags {     let mut result =
// PipelineStageFlags::empty();     let mut bits = access.as_raw();
//     while bits != 0 {
//         let bit = 1 << bits.trailing_zeros();
//         bits &= !bit;
//         result |= supported_stages_inner(AccessFlags::from_raw(bit));
//     }
//     result
// }

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
