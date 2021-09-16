use crate::access::Access;

bitflags::bitflags! {
    /// Memory usage type.
    /// Bits set define intended usage for requested memory.
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct MemoryUsage: u8 {
        /// Hints allocator that memory will be used for data downloading.
        /// Allocator will strongly prefer host-cached memory.
        /// Implies `HOST_ACCESS` flag.
        const DOWNLOAD = 0x04;

        /// Hints allocator that memory will be used for data uploading.
        /// If `DOWNLOAD` flag is not set then allocator will assume that
        /// host will access memory in write-only manner and may
        /// pick not host-cached.
        /// Implies `HOST_ACCESS` flag.
        const UPLOAD = 0x08;

        /// Hints allocator to find memory with fast device access.
        const FAST_DEVICE_ACCESS = 0x10;

        /// Hits allocator that memory will be released shortly.
        const TRANSIENT = 0x20;
    }
}

/// Global barriers define a set of accesses on multiple resources at once.
/// If a buffer or image doesn't require a queue ownership transfer, or an image
/// doesn't require a layout transition (e.g. you're using one of the
/// `ImageLayout::General*` layouts) then a global barrier should be preferred.
///
/// Simply define the previous and next access types of resources affected.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct GlobalMemoryBarrier<'a> {
    pub prev_accesses: &'a [Access],
    pub next_accesses: &'a [Access],

}
