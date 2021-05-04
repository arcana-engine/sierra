//!
//! Sierra is Vulkan-lite API, focused on ease of use
//! while maintaining high level of control.
//!
//! While resembles Vulkan in most ways,\
//! sierra does both memory and descriptor allocation.
//! And tracks resources usage to free them automatically once
//! no references left.
//!
//! Sierra provides reach proc-macro system for declarative
//! descriptor sets and render passes.
//!

// #![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]

use std::{
    cmp::{min, Ord, Ordering},
    convert::{TryFrom as _, TryInto as _},
    error::Error,
    fmt::Debug,
    num::TryFromIntError,
};

pub mod backend;

mod accel;
mod access;
mod buffer;
mod descriptor;
mod encode;
mod fence;
mod format;
mod framebuffer;
// mod glsl;
mod image;
mod memory;
mod physical;
mod pipeline;
mod queue;
mod render_pass;
mod repr;
mod sampler;
mod semaphore;
mod shader;
mod stage;
mod surface;
mod swapchain;
mod view;

pub use self::{
    accel::*,
    access::*,
    backend::{Device, Graphics},
    buffer::*,
    descriptor::*,
    encode::*,
    fence::*,
    format::*,
    framebuffer::*,
    image::*,
    memory::*,
    physical::*,
    pipeline::*,
    queue::*,
    render_pass::*,
    repr::*,
    sampler::*,
    semaphore::*,
    shader::*,
    stage::*,
    surface::*,
    swapchain::*,
    view::*,
};

pub use sierra_proc::{descriptors, graphics_pipeline_desc, pass, pipeline, shader_repr};

/// Re-exporting bumpalo for code-gen.
#[doc(hidden)]
pub use bumpalo;

/// Re-exporting OrderedFloat for code-gen.
#[doc(hidden)]
pub use ordered_float::OrderedFloat;

/// Image size is defined to `u32` which is standard for graphics API of today.
pub type ImageSize = u32;

/// Two dimensional extent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct Extent2d {
    /// Width of the extent.
    pub width: ImageSize,

    /// Height of the extent.
    pub height: ImageSize,
}

impl Extent2d {
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn into_3d(self) -> Extent3d {
        Extent3d {
            width: self.width,
            height: self.height,
            depth: 1,
        }
    }

    pub fn min(&self, rhs: &Self) -> Self {
        Extent2d {
            width: min(self.width, rhs.width),
            height: min(self.height, rhs.height),
        }
    }
}

impl PartialOrd for Extent2d {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let width = Ord::cmp(&self.width, &other.width);

        let height = Ord::cmp(&self.height, &other.height);

        merge_ordering(width, height)
    }
}

/// Three dimensional extent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct Extent3d {
    /// Width of the extent.
    pub width: ImageSize,

    /// Height of the extent.
    pub height: ImageSize,

    /// Depth of the extent.
    pub depth: ImageSize,
}

impl PartialOrd for Extent3d {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let width = Ord::cmp(&self.width, &other.width);
        let height = Ord::cmp(&self.height, &other.height);
        let depth = Ord::cmp(&self.depth, &other.depth);

        merge_ordering(merge_ordering(width, height)?, depth)
    }
}

impl Extent3d {
    pub fn into_2d(self) -> Extent2d {
        Extent2d {
            width: self.width,
            height: self.height,
        }
    }

    pub fn min(&self, rhs: &Self) -> Self {
        Extent3d {
            width: min(self.width, rhs.width),
            height: min(self.height, rhs.height),
            depth: min(self.depth, rhs.depth),
        }
    }
}

/// Image offset is defined to `i32` which is standard for graphics API today.
pub type ImageOffset = i32;

/// Two dimensional offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct Offset2d {
    /// Width offset.
    pub x: ImageOffset,

    /// Height offset.
    pub y: ImageOffset,
}

impl Offset2d {
    pub const ZERO: Self = Offset2d { x: 0, y: 0 };

    pub fn from_extent(extent: Extent2d) -> Result<Self, TryFromIntError> {
        Ok(Offset2d {
            x: extent.width.try_into()?,
            y: extent.height.try_into()?,
        })
    }
}

/// Three dimensional offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct Offset3d {
    /// Width offset.
    pub x: ImageOffset,

    /// Height offset.
    pub y: ImageOffset,

    /// Depth offset.
    pub z: ImageOffset,
}

impl Offset3d {
    pub const ZERO: Self = Offset3d { x: 0, y: 0, z: 0 };

    pub fn from_extent(extent: Extent3d) -> Result<Self, TryFromIntError> {
        Ok(Offset3d {
            x: extent.width.try_into()?,
            y: extent.height.try_into()?,
            z: extent.depth.try_into()?,
        })
    }
}

/// Error that may occur when allocation fails because of either
/// device memory is exhausted.
///
/// Deallocation of device memory or other resources may increase chance
/// that operation would succeed.
#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[error("Out of device memory")]
pub struct OutOfMemory;

fn merge_ordering(left: Ordering, right: Ordering) -> Option<Ordering> {
    match (left, right) {
        (Ordering::Equal, right) => Some(right),
        (left, Ordering::Equal) => Some(left),
        (Ordering::Less, Ordering::Less) => Some(Ordering::Less),
        (Ordering::Greater, Ordering::Greater) => Some(Ordering::Greater),
        _ => None,
    }
}

/// Device address is `u64` value pointing into device resource.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct DeviceAddress(pub std::num::NonZeroU64);

impl DeviceAddress {
    pub fn offset(&mut self, offset: u64) -> DeviceAddress {
        let value = self.0.get().checked_add(offset).unwrap();

        DeviceAddress(unsafe { std::num::NonZeroU64::new_unchecked(value) })
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum IndexType {
    U16,
    U32,
}

impl IndexType {
    pub fn size(&self) -> u8 {
        match self {
            IndexType::U16 => 2,
            IndexType::U32 => 4,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateDeviceError<E: Error + 'static> {
    #[error(transparent)]
    OutOfMemory {
        #[from]
        source: OutOfMemory,
    },

    #[error("Non-existed families are requested")]
    BadFamiliesRequested,

    #[error(transparent)]
    CannotFindRequeredQueues { source: E },

    /// Implementation specific error.
    #[error("Failed to load functions")]
    FunctionLoadFailed,
}

/// Possible error which can be returned from `create_buffer_*`.
#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
pub enum CreateBufferError {
    #[error(transparent)]
    OutOfMemory {
        #[from]
        source: OutOfMemory,
    },

    #[error("Buffer usage {usage:?} is unsupported")]
    UnsupportedUsage { usage: BufferUsage },
}

/// Possible error which can be returned from `create_image_*)`.
#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
pub enum CreateImageError {
    #[error(transparent)]
    OutOfMemory {
        #[from]
        source: OutOfMemory,
    },

    #[error("Parameters combination `{info:?}` is unsupported")]
    Unsupported { info: ImageInfo },
}

/// Possible error that may occur during memory mapping.
#[derive(Clone, Copy, Debug, thiserror::Error)]
pub enum MapError {
    /// Device memory is exhausted.
    #[error(transparent)]
    OutOfMemory {
        #[from]
        source: OutOfMemory,
    },

    /// Memory is not host-visible.
    #[error("Memory is not host-visible")]
    NonHostVisible,

    /// Memory is already mapped
    #[error("Memory is already mapped")]
    AlreadyMapped,

    /// Map failed for implementation specific reason
    #[error("Map failed for implementation specific reason")]
    MapFailed,
}

#[doc(hidden)]
pub trait OrdArith<T>: Copy {
    fn cmp(self, rhs: T) -> Ordering;
}

impl<T> OrdArith<T> for T
where
    T: Ord + Copy,
{
    fn cmp(self, rhs: T) -> Ordering {
        <T as Ord>::cmp(&self, &rhs)
    }
}

impl OrdArith<u32> for usize {
    fn cmp(self, rhs: u32) -> Ordering {
        match u32::try_from(self) {
            Ok(lhs) => Ord::cmp(&lhs, &rhs),
            Err(_) => Ordering::Greater,
        }
    }
}

impl OrdArith<u64> for usize {
    fn cmp(self, rhs: u64) -> Ordering {
        match u64::try_from(self) {
            Ok(lhs) => Ord::cmp(&lhs, &rhs),
            Err(_) => Ordering::Greater,
        }
    }
}

impl OrdArith<u128> for usize {
    fn cmp(self, rhs: u128) -> Ordering {
        match u128::try_from(self) {
            Ok(lhs) => Ord::cmp(&lhs, &rhs),
            Err(_) => Ordering::Greater,
        }
    }
}

impl OrdArith<usize> for u32 {
    fn cmp(self, rhs: usize) -> Ordering {
        match u32::try_from(rhs) {
            Ok(rhs) => Ord::cmp(&self, &rhs),
            Err(_) => Ordering::Less,
        }
    }
}

impl OrdArith<usize> for u64 {
    fn cmp(self, rhs: usize) -> Ordering {
        match u64::try_from(rhs) {
            Ok(rhs) => Ord::cmp(&self, &rhs),
            Err(_) => Ordering::Less,
        }
    }
}

impl OrdArith<usize> for u128 {
    fn cmp(self, rhs: usize) -> Ordering {
        match u128::try_from(rhs) {
            Ok(rhs) => Ord::cmp(&self, &rhs),
            Err(_) => Ordering::Less,
        }
    }
}

impl OrdArith<u32> for u64 {
    fn cmp(self, rhs: u32) -> Ordering {
        Ord::cmp(&self, &u64::from(rhs))
    }
}

impl OrdArith<u32> for u128 {
    fn cmp(self, rhs: u32) -> Ordering {
        Ord::cmp(&self, &u128::from(rhs))
    }
}

impl OrdArith<u64> for u128 {
    fn cmp(self, rhs: u64) -> Ordering {
        Ord::cmp(&self, &u128::from(rhs))
    }
}

#[doc(hidden)]
pub fn arith_cmp<T>(lhs: impl OrdArith<T>, rhs: T) -> Ordering {
    lhs.cmp(rhs)
}

#[doc(hidden)]
pub fn arith_eq<T>(lhs: impl OrdArith<T>, rhs: T) -> bool {
    lhs.cmp(rhs) == Ordering::Equal
}

#[doc(hidden)]
pub fn arith_ne<T>(lhs: impl OrdArith<T>, rhs: T) -> bool {
    lhs.cmp(rhs) != Ordering::Equal
}

#[doc(hidden)]
pub fn arith_lt<T>(lhs: impl OrdArith<T>, rhs: T) -> bool {
    lhs.cmp(rhs) == Ordering::Less
}

#[doc(hidden)]
pub fn arith_gt<T>(lhs: impl OrdArith<T>, rhs: T) -> bool {
    lhs.cmp(rhs) == Ordering::Greater
}

#[doc(hidden)]
pub fn arith_le<T>(lhs: impl OrdArith<T>, rhs: T) -> bool {
    lhs.cmp(rhs) != Ordering::Greater
}

#[doc(hidden)]
pub fn arith_ge<T>(lhs: impl OrdArith<T>, rhs: T) -> bool {
    lhs.cmp(rhs) != Ordering::Less
}

/// Handles host OOM the same way global allocator does.
/// This function should be called on host OOM error returned from Vulkan API.
pub fn out_of_host_memory() -> ! {
    use std::alloc::{handle_alloc_error, Layout};

    handle_alloc_error(unsafe { Layout::from_size_align_unchecked(1, 1) })
}

/// Handles host OOM the same way global allocator does.
/// This function should be called on host OOM error returned from Vulkan API.
pub fn host_memory_space_overlow() -> ! {
    panic!("Memory address space overlow")
}

fn assert_object<T: Debug + Send + Sync + 'static>() {}
fn assert_error<T: Error + Send + Sync + 'static>() {}

pub trait Align<T> {
    fn align_up(self, value: T) -> Option<T>;
}

impl<T> Align<u64> for T
where
    T: Into<u64>,
{
    fn align_up(self, value: u64) -> Option<u64> {
        let align = self.into();
        Some(align.checked_add(value)? & !align)
    }
}

impl<T> Align<u32> for T
where
    T: Into<u32>,
{
    fn align_up(self, value: u32) -> Option<u32> {
        let align = self.into();
        Some(align.checked_add(value)? & !align)
    }
}

impl<T> Align<u16> for T
where
    T: Into<u16>,
{
    fn align_up(self, value: u16) -> Option<u16> {
        let align = self.into();
        Some(align.checked_add(value)? & !align)
    }
}

impl<T> Align<u8> for T
where
    T: Into<u8>,
{
    fn align_up(self, value: u8) -> Option<u8> {
        let align = self.into();
        Some(align.checked_add(value)? & !align)
    }
}

impl<T> Align<usize> for T
where
    T: Into<usize>,
{
    fn align_up(self, value: usize) -> Option<usize> {
        let align = self.into();
        Some(align.checked_add(value)? & !align)
    }
}

pub fn align_up<A, T>(align_mask: A, value: T) -> Option<T>
where
    A: Align<T>,
{
    align_mask.align_up(value)
}

pub fn align_down(align_mask: u64, value: u64) -> u64 {
    value & !align_mask
}

#[macro_export]
macro_rules! descriptor_set_layout_bindings {
    ($($ty:ident $(($count:expr))? $(@$binding:literal)? for $($stages:ident),+ $($(| $flags:ident)+)?),*) => {
        {
            let mut binding = 0;
            vec![
                $({
                    $(binding = $binding + 1)?;
                    $crate::DescriptorSetLayoutBinding {
                        binding: binding - 1,
                        ty: $crate::DescriptorType::$ty,
                        count: 1 $(- 1 + $count)?,
                        stages: $($crate::ShaderStageFlags::$stages)|+,
                        flags: $crate::DescriptorBindingFlags::empty() $(| $crate::DescriptorBindingFlags::$flags)*,
                    }
                },)*
            ]
        }
    };
}

#[macro_export]
macro_rules! descriptor_set_layout {
    ($(|$flags:ident) *$($ty:ident $(($count:expr))? $(@$binding:literal)? for $($stages:ident)+ $($(| $bflags:ident)+)?),*) => {
        $crate::DescriptorSetLayoutInfo {
            flags: $crate::DescriptorSetLayoutFlags::empty() $(| $crate::DescriptorSetLayoutFlags::$flags)*,
            bindings: descriptor_set_layout_bindings!($($ty $(@$binding)? $(* $count)? for $($stages)+ $($(| $bflags)+)?)*),
        }
    }
}
