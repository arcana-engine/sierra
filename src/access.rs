bitflags::bitflags! {
    /// Flags for access types.
    pub struct AccessFlags: u32 {
        /// Read access to indirect command data
        /// read as part of an indirect build,
        /// trace, drawing or dispatch command.
        const INDIRECT_COMMAND_READ = 0x00000001;

        /// Read access to an index buffer as part of an indexed drawing command
        const INDEX_READ = 0x00000002;

        /// Read access to a vertex buffer as part of a drawing command
        const VERTEX_ATTRIBUTE_READ = 0x00000004;

        /// Read access to a uniform buffer..
        const UNIFORM_READ = 0x00000008;

        /// Read access to an input attachment
        /// within a render pass during fragment shading.
        const INPUT_ATTACHMENT_READ = 0x00000010;

        /// Read access to a storage buffer, physical storage buffer,
        /// shader binding table, uniform texel buffer, storage texel buffer,
        /// sampled image, or storage image.
        const SHADER_READ = 0x00000020;

        /// Write access to a storage buffer, physical storage buffer,
        /// storage texel buffer, or storage image.
        const SHADER_WRITE = 0x00000040;

        /// Read access to a color attachment,
        /// such as via blending, logic operations,
        /// or via certain subpass load operations.\
        /// It does not include advanced blend operations.
        const COLOR_ATTACHMENT_READ = 0x00000080;

        /// Write access to a color, resolve,
        /// or depth/stencil resolve attachment
        /// during a render pass
        /// or via certain subpass load and store operations.
        const COLOR_ATTACHMENT_WRITE = 0x00000100;

        /// Read access to a depth/stencil attachment,
        /// via depth or stencil operations
        /// or via certain subpass load operations.
        const DEPTH_STENCIL_ATTACHMENT_READ = 0x00000200;

        /// Write access to a depth/stencil attachment,
        /// via depth or stencil operations
        /// or via certain subpass load and store operations.
        const DEPTH_STENCIL_ATTACHMENT_WRITE = 0x00000400;

        /// Read access to an image or buffer in a copy operation.
        const TRANSFER_READ = 0x00000800;

        /// Write access to an image or buffer in a clear or copy operation.
        const TRANSFER_WRITE = 0x00001000;

        /// Read access by a host operation.
        const HOST_READ = 0x00002000;

        /// Write access by a host operation.
        const HOST_WRITE = 0x00004000;

        /// All read accesses.
        const MEMORY_READ = 0x00008000;

        /// All write accesses.
        const MEMORY_WRITE = 0x00010000;

        // /// Write access to a transform feedback buffer made
        // /// when transform feedback is active.
        // const TRANSFORM_FEEDBACK_WRITE = 0x00020000;

        // /// Read access to a transform feedback counter buffer
        // /// which is read when vkCmdBeginTransformFeedbackEXT executes.
        // const TRANSFORM_FEEDBACK_COUNTER_READ = 0x00040000;

        // /// Write access to a transform feedback counter buffer
        // /// which is written when vkCmdEndTransformFeedbackEXT executes.
        // const TRANSFORM_FEEDBACK_COUNTER_WRITE = 0x00080000;

        /// Read access to a predicate as part of conditional rendering.
        const CONDITIONAL_RENDERING_READ = 0x00100000;

        /// Similar to [`COLOR_ATTACHMENT_READ`],
        /// but also includes advanced blend operations.
        const COLOR_ATTACHMENT_READ_NONCOHERENT = 0x00200000;

        /// Read access to an acceleration structure
        /// as part of a trace, build, or copy command,
        /// or to an acceleration structure scratch buffer
        // as part of a build command.
        const ACCELERATION_STRUCTURE_READ = 0x00400000;

        /// Write access to an acceleration structure
        /// or acceleration structure scratch buffer
        /// as part of a build or copy command.
        const ACCELERATION_STRUCTURE_WRITE = 0x00800000;

        /// Read access to a fragment density map attachment
        /// during dynamic fragment density map operations.
        const FRAGMENT_DENSITY_MAP_READ = 0x01000000;

        /// Read access to a fragment shading rate attachment
        /// or shading rate image during rasterization.
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
                // | Self::TRANSFORM_FEEDBACK_WRITE
                // | Self::TRANSFORM_FEEDBACK_COUNTER_WRITE
                | Self::ACCELERATION_STRUCTURE_WRITE,
        )
    }
}
