bitflags::bitflags! {
    pub struct AccessFlags: u32 {
        const INDIRECT_COMMAND_READ = 0x00000001;
        const INDEX_READ = 0x00000002;
        const VERTEX_ATTRIBUTE_READ = 0x00000004;
        const UNIFORM_READ = 0x00000008;
        const INPUT_ATTACHMENT_READ = 0x00000010;
        const SHADER_READ = 0x00000020;
        const SHADER_WRITE = 0x00000040;
        const COLOR_ATTACHMENT_READ = 0x00000080;
        const COLOR_ATTACHMENT_WRITE = 0x00000100;
        const DEPTH_STENCIL_ATTACHMENT_READ = 0x00000200;
        const DEPTH_STENCIL_ATTACHMENT_WRITE = 0x00000400;
        const TRANSFER_READ = 0x00000800;
        const TRANSFER_WRITE = 0x00001000;
        const HOST_READ = 0x00002000;
        const HOST_WRITE = 0x00004000;
        const MEMORY_READ = 0x00008000;
        const MEMORY_WRITE = 0x00010000;
        const TRANSFORM_FEEDBACK_WRITE = 0x00020000;
        const TRANSFORM_FEEDBACK_COUNTER_READ = 0x00040000;
        const TRANSFORM_FEEDBACK_COUNTER_WRITE = 0x00080000;
        const CONDITIONAL_RENDERING_READ = 0x00100000;
        const COLOR_ATTACHMENT_READ_NONCOHERENT = 0x00200000;
        const ACCELERATION_STRUCTURE_READ = 0x00400000;
        const ACCELERATION_STRUCTURE_WRITE = 0x00800000;
        const FRAGMENT_DENSITY_MAP_READ = 0x01000000;
        const FRAGMENT_SHADING_RATE_ATTACHMENT_READ = 0x02000000;
    }
}

impl AccessFlags {
    pub fn is_readonly(self) -> bool {
        !self.intersects(
            Self::SHADER_WRITE
                | Self::COLOR_ATTACHMENT_WRITE
                | Self::DEPTH_STENCIL_ATTACHMENT_WRITE
                | Self::TRANSFER_WRITE
                | Self::HOST_WRITE
                | Self::MEMORY_WRITE
                | Self::TRANSFORM_FEEDBACK_WRITE
                | Self::TRANSFORM_FEEDBACK_COUNTER_WRITE
                | Self::ACCELERATION_STRUCTURE_WRITE,
        )
    }
}
