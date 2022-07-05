use crate::access::Access;

bitflags::bitflags! {
    /// Memory usage type.
    /// Bits set define intended usage for requested memory.
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct MemoryUsage: u8 {
        /// Specifies that memory will be used for data downloading.
        /// Allocator will strongly prefer host-cached memory.
        const DOWNLOAD = 0x04;

        /// Specifies that memory will be used for data uploading.
        /// If `DOWNLOAD` flag is not set then allocator will assume that
        /// host will access memory in write-only manner and may
        /// pick not host-cached.
        const UPLOAD = 0x08;

        /// Hints allocator to find memory with fast device access.
        const FAST_DEVICE_ACCESS = 0x10;

        /// Hits allocator that memory will be released shortly.
        const TRANSIENT = 0x20;
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct MemoryBarrier {
    pub src: Access,
    pub dst: Access,
}
