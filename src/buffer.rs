use crate::align_up;
pub use crate::backend::{Buffer, MappableBuffer};

bitflags::bitflags! {
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct BufferUsage: u32 {
        const TRANSFER_SRC = 0x00000001;
        const TRANSFER_DST = 0x00000002;
        const UNIFORM_TEXEL = 0x00000004;
        const STORAGE_TEXEL = 0x00000008;
        const UNIFORM = 0x00000010;
        const STORAGE = 0x00000020;
        const INDEX = 0x00000040;
        const VERTEX = 0x00000080;
        const INDIRECT = 0x00000100;
        const CONDITIONAL_RENDERING = 0x00000200;
        const ACCELERATION_STRUCTURE_BUILD_INPUT = 0x00000400;
        const ACCELERATION_STRUCTURE_STORAGE = 0x00000800;
        const SHADER_BINDING_TABLE = 0x00001000;
        const TRANSFORM_FEEDBACK = 0x00002000;
        const TRANSFORM_FEEDBACK_COUNTER = 0x00004000;
        const DEVICE_ADDRESS = 0x0008000;
        const TRANSIENT = 0x0010000;
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

/// Buffer region.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BufferRegion {
    pub buffer: Buffer,
    pub offset: u64,
    pub size: u64,
}

impl BufferRegion {
    pub fn whole(buffer: Buffer) -> Self {
        BufferRegion {
            offset: 0,
            size: buffer.info().size,
            buffer,
        }
    }
}

impl From<Buffer> for BufferRegion {
    fn from(buffer: Buffer) -> Self {
        BufferRegion::whole(buffer)
    }
}

/// Buffer region with specified stride value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StridedBufferRegion {
    pub region: BufferRegion,
    pub stride: u64,
}
