bitflags::bitflags! {
    /// Specifies the access types for a resource.
    pub struct Access: u64 {
        /// Specifies read access to command data read from indirect buffers
        /// as part of an indirect build, trace, drawing or dispatch command.
        const INDIRECT_COMMAND_READ = 0x00000001;

        /// Specifies read access to an index buffer as part of an indexed drawing command.
        const INDEX_READ = 0x00000002;

        /// Specifies read access to a vertex buffer as part of a drawing command.
        const VERTEX_ATTRIBUTE_READ = 0x00000004;

        /// Specifies read access to a uniform buffer in any shader pipeline stage.
        const UNIFORM_READ = 0x00000008;

        /// Specifies read access to a uniform texel buffer or sampled image in any shader pipeline stage.
        const SHADER_SAMPLED_READ = 0x00000010;

        /// Specifies read access to a storage buffer, physical storage buffer, storage texel buffer, or storage image in any shader pipeline stage.
        const SHADER_STORAGE_READ = 0x00000020;

        /// Specifies read access to a shader binding table in any shader pipeline stage.
        const SHADER_BINDING_TABLE_READ = 0x00000040;

        /// Specifies read access to an input attachment
        /// within a render pass during subpass shading or fragment shading.
        const INPUT_ATTACHMENT_READ = 0x00000080;

        /// Specifies read access to a color attachment, such as
        /// via blending, logic operations, or via certain subpass load operations.
        const COLOR_ATTACHMENT_READ = 0x00000100;

        /// Specifies read access to a depth/stencil attachment,
        /// via depth or stencil operations or via certain subpass load operations.
        const DEPTH_STENCIL_ATTACHMENT_READ = 0x00000200;

        /// Specifies read access to an acceleration structure
        /// as part of a trace, build, or copy command,
        /// or to an acceleration structure scratch buffer as part of a build command.
        const ACCELERATION_STRUCTURE_READ = 0x00001000;

        /// Specifies read access to an image or buffer in a copy operation.
        const TRANSFER_READ = 0x00000400;

        /// Specifies read access by a host operation.
        /// Accesses of this type are not performed through a resource, but directly on memory.
        const HOST_READ = 0x00000800;

        /// Specifies write access to a storage buffer,
        /// physical storage buffer, storage texel buffer,
        /// or storage image in any shader pipeline stage.
        const SHADER_STORAGE_WRITE = 0x00002000;

        /// Specifies write access to a color, resolve, or depth/stencil resolve attachment
        /// during a render pass or via certain subpass load and store operations.
        const COLOR_ATTACHMENT_WRITE = 0x00004000;

        /// Specifies write access to a depth/stencil attachment,
        /// via depth or stencil operations or via certain subpass load and store operations.
        const DEPTH_STENCIL_ATTACHMENT_WRITE = 0x00008000;

        /// Specifies write access to an acceleration structure or acceleration structure scratch buffer
        /// as part of a build or copy command
        const ACCELERATION_STRUCTURE_WRITE = 0x00010000;

        /// Specifies write access to an image or buffer in a clear or copy operation
        const TRANSFER_WRITE = 0x00020000;

        /// Specifies write access by a host operation.
        /// Accesses of this type are not performed through a resource, but directly on memory.
        const HOST_WRITE = 0x00040000;
    }
}
