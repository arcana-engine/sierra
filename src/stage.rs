bitflags::bitflags! {
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct PipelineStageFlags: u32 {
        /// Pseudo-stage that precedes all other stages and doesn't execute any commands.
        /// Using it in first scope of dependency will
        /// not cause any waiting, because no operations should be waited upon.
        /// Using it in second scope will make all operations in second scope to wait for operations first scope.
        const TOP_OF_PIPE = 0x00000001;

        /// Stage at which indirect draw buffer is read.
        const DRAW_INDIRECT = 0x00000002;

        /// Stage at which vertex buffers are read.
        const VERTEX_INPUT = 0x00000004;

        /// Stage at which vertex shader is executed.
        const VERTEX_SHADER = 0x00000008;
        // const TESSELLATION_CONTROL_SHADER = 0x00000010;
        // const TESSELLATION_EVALUATION_SHADER = 0x00000020;
        // const GEOMETRY_SHADER = 0x00000040;

        /// Stage at which early fragment depth and stencil test is performed
        /// before fragment shader execution.
        const EARLY_FRAGMENT_TESTS = 0x00000100;

        /// Stage at which fragment shader is executed.
        const FRAGMENT_SHADER = 0x00000080;

        /// Stage at which late fragment depth and stencil test is performed
        /// after fragment shader execution.
        const LATE_FRAGMENT_TESTS = 0x00000200;

        /// Stage at which color output of fragment shader is written
        /// and multisample resolve operation happens.
        const COLOR_ATTACHMENT_OUTPUT = 0x00000400;

        /// Stage at which compute shader is executed.
        const COMPUTE_SHADER = 0x00000800;

        /// Stage at which transfer commands (Copy, Blit etc) are executed.
        const TRANSFER = 0x00001000;

        /// Pseudo-stage that follows all other stages and doesn't execute any commands.
        /// Using it in first scope will make operations in second scope to wait for all operations first scope.
        /// Using it in second scope of dependency will
        /// not cause any waiting, because no operations should be waited upon.
        const BOTTOM_OF_PIPE = 0x00002000;

        /// Pseudo-stage at which HOST access to resources is performed.
        /// It has very limited use because command submission creates
        /// memory dependency between host access and device operations.
        const HOST = 0x00004000;

        /// Flag that can be used instead of specifying all graphics stages
        /// including those from enabled extensions.
        const ALL_GRAPHICS = 0x00008000;

        /// Flag that can be used instead of specifying all compute stages
        /// including those from enabled extensions.
        const ALL_COMMANDS = 0x00010000;

        /// Stage at which ray-tracing pipeline is executed.
        const RAY_TRACING_SHADER = 0x00200000;

        /// Stage at which acceleration structures are built.
        const ACCELERATION_STRUCTURE_BUILD = 0x02000000;
    }
}
