pub use crate::backend::{Buffer, MappableBuffer};
use crate::{
    access::AccessFlags,
    align_up,
    encode::Encoder,
    queue::{Ownership, QueueId},
    stage::PipelineStageFlags,
};

bitflags::bitflags! {
    /// Flags to specify allowed usages for buffer.
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct BufferUsage: u32 {
        /// Buffer with this usage flag can be used as source for various transfer operations.
        const TRANSFER_SRC = 0x00000001;

        /// Buffer with this usage flag can be used as destination for various transfer operations.
        const TRANSFER_DST = 0x00000002;

        /// Buffer with this usage flag can used as `UniformTexel` descriptor.
        const UNIFORM_TEXEL = 0x00000004;

        /// Buffer with this usage flag can used as `StorageTexel` descriptor.
        const STORAGE_TEXEL = 0x00000008;

        /// Buffer with this usage flag can used as `Uniform` descriptor.
        const UNIFORM = 0x00000010;

        /// Buffer with this usage flag can used as `Storage` descriptor.
        const STORAGE = 0x00000020;

        /// Buffer with this usage flag can used in `bind_index_buffer` encoder method.
        const INDEX = 0x00000040;

        /// Buffer with this usage flag can used in `bind_vertex_buffers` encoder method.
        const VERTEX = 0x00000080;

        /// Buffer with this usage flag can used for indirect drawing.
        const INDIRECT = 0x00000100;

        /// Buffer with this usage flag can used for conditional rendering.
        const CONDITIONAL_RENDERING = 0x00000200;

        /// Buffer with this usage flag can used as input for acceleration structure build.
        const ACCELERATION_STRUCTURE_BUILD_INPUT = 0x00000400;

        /// Buffer with this usage flag can used to store acceleration structure.
        const ACCELERATION_STRUCTURE_STORAGE = 0x00000800;

        /// Buffer with this usage flag can used to specify shader binding table.
        const SHADER_BINDING_TABLE = 0x00001000;
        const TRANSFORM_FEEDBACK = 0x00002000;
        const TRANSFORM_FEEDBACK_COUNTER = 0x00004000;

        /// Buffer with this usage flag can be used to retrieve a buffer device address.
        const DEVICE_ADDRESS = 0x0008000;
    }
}

/// Information required to create a buffer.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct BufferInfo {
    /// Alignment mask for content buffer can hold.
    pub align: u64,

    /// Size of content buffer can hold.
    pub size: u64,

    /// Usage types supported by buffer.
    pub usage: BufferUsage,
}

impl BufferInfo {
    #[inline(always)]
    pub(crate) fn is_valid(&self) -> bool {
        let is_mask = self
            .align
            .checked_add(1)
            .map_or(false, u64::is_power_of_two);

        is_mask && (align_up(self.align, self.size).is_some())
    }
}

/// Buffer range.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BufferRange {
    pub buffer: Buffer,
    pub offset: u64,
    pub size: u64,
}

impl BufferRange {
    pub fn whole(buffer: Buffer) -> Self {
        BufferRange {
            offset: 0,
            size: buffer.info().size,
            buffer,
        }
    }
}

impl From<Buffer> for BufferRange {
    fn from(buffer: Buffer) -> Self {
        BufferRange::whole(buffer)
    }
}

/// Buffer range with specified stride value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StridedBufferRange {
    pub range: BufferRange,
    pub stride: u64,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct BufferMemoryBarrier<'a> {
    pub buffer: &'a Buffer,
    pub offset: u64,
    pub size: u64,
    pub old_access: AccessFlags,
    pub new_access: AccessFlags,
    pub family_transfer: Option<(u32, u32)>,
}

/// Buffer range with access mask,
/// specifying how it may be accessed "before".
///
/// Note that "before" is loosely defined,
/// as whatever previous owners do.
/// Which should be translated to "earlier GPU operations"
/// but this crate doesn't attempt to enforce that.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BufferRangeState {
    pub range: BufferRange,
    pub access: AccessFlags,
    pub stages: PipelineStageFlags,
    pub family: Ownership,
}

impl BufferRangeState {
    ///
    pub fn access<'a>(
        &'a mut self,
        access: AccessFlags,
        stages: PipelineStageFlags,
        queue: QueueId,
        encoder: &mut Encoder<'a>,
    ) -> &'a BufferRange {
        match self.family {
            Ownership::NotOwned => encoder.buffer_barriers(
                self.stages,
                stages,
                &[BufferMemoryBarrier {
                    buffer: &self.range.buffer,
                    old_access: self.access,
                    new_access: access,
                    family_transfer: None,
                    offset: self.range.offset,
                    size: self.range.size,
                }],
            ),
            Ownership::Owned { family } => {
                assert_eq!(family, queue.family, "Wrong queue family owns the buffer");

                encoder.buffer_barriers(
                    self.stages,
                    stages,
                    &[BufferMemoryBarrier {
                        buffer: &self.range.buffer,
                        old_access: self.access,
                        new_access: access,
                        family_transfer: None,
                        offset: self.range.offset,
                        size: self.range.size,
                    }],
                )
            }
            Ownership::Transition { from, to } => {
                assert_eq!(
                    to, queue.family,
                    "Buffer is being transitioned to wrong queue family"
                );

                encoder.buffer_barriers(
                    self.stages,
                    stages,
                    &[BufferMemoryBarrier {
                        buffer: &self.range.buffer,
                        old_access: self.access,
                        new_access: access,
                        family_transfer: Some((from, to)),
                        offset: self.range.offset,
                        size: self.range.size,
                    }],
                )
            }
        }
        self.family = Ownership::Owned {
            family: queue.family,
        };
        self.stages = stages;
        self.access = access;
        &self.range
    }

    pub fn overwrite<'a>(
        &'a mut self,
        access: AccessFlags,
        stages: PipelineStageFlags,
        queue: QueueId,
        encoder: &mut Encoder<'a>,
    ) -> &'a BufferRange {
        encoder.buffer_barriers(
            self.stages,
            stages,
            &[BufferMemoryBarrier {
                buffer: &self.range.buffer,
                old_access: AccessFlags::empty(),
                new_access: access,
                family_transfer: None,
                offset: self.range.offset,
                size: self.range.size,
            }],
        );
        self.family = Ownership::Owned {
            family: queue.family,
        };
        self.stages = stages;
        self.access = access;
        &self.range
    }
}
