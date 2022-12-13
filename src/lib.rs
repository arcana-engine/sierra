//! Sierra is Vulkan-lite API, focused on ease of use
//! while maintaining high level of control.
//!
//! While resembles Vulkan in most ways,\
//! sierra does both memory and descriptor allocation automatically.
//! Additionally sierra tracks resources usage to free them once no references left.
//!
//! Sierra provides rich proc-macro system for declarative descriptor sets and render passes.

// Someday this will be uncommented.
// #![warn(missing_docs)]

#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]

use std::{
    cmp::{Ord, Ordering},
    convert::TryFrom,
    error::Error,
    fmt::Debug,
};

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! trace {
    ($($tokens:tt)*) => {
        tracing::trace!($($tokens)*)
    };
}

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! debug {
    ($($tokens:tt)*) => {
        tracing::debug!($($tokens)*)
    };
}

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! info {
    ($($tokens:tt)*) => {
        tracing::info!($($tokens)*)
    };
}

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! warn {
    ($($tokens:tt)*) => {
        tracing::warn!($($tokens)*)
    };
}

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! error {
    ($($tokens:tt)*) => {
        tracing::error!($($tokens)*)
    };
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! trace {
    ($($e:expr),*) => {{ $(let _ = &$e;)* }};
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! debug {
    ($($e:expr),*) => {{ $( let _ = &$e;)* }};
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! info {
    ($($e:expr),*) => {{ $(let _ = &$e;)* }};
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! warn {
    ($($e:expr),*) => {{ $(let _ = &$e;)* }};
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! error {
    ($($e:expr),*) => {{ $(let _ = &$e;)* }};
}

pub mod backend;

mod accel;
mod access;
mod buffer;
mod cache;
mod descriptor;
mod dimensions;
mod encode;
mod fence;
mod format;
mod framebuffer;
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
mod view;

pub use self::{
    accel::*,
    access::*,
    backend::{Device, Graphics},
    buffer::*,
    cache::*,
    descriptor::*,
    dimensions::*,
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
    view::*,
};

pub use sierra_proc::{
    binding_flags, format, graphics_pipeline_desc, shader_stages, swizzle, Descriptors, Pass,
    PipelineInput, ShaderRepr,
};

/// Re-exporting for code-gen.
#[doc(hidden)]
pub use {arrayvec, bytemuck, scoped_arena, smallvec};

/// Error that may occur when allocation fails because of either
/// device memory is exhausted.
///
/// Deallocation of device memory or other resources may increase chance
/// that operation would succeed.
#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[error("Out of device memory")]
pub struct OutOfMemory;

/// Error that may occur during execution on the device
/// and then signalled on command submission or waiting operations.
///
/// This error is unrecoverable lost `Device` state cannot be changed to not-lost.
/// It must be recreated.
///
/// Any mapped memory allocated from lost device is still valid for access, but its content is undefined.
///
/// If this error is returned by `PhysicalDevice::create_device` function
/// then physical device is lost and cannot be used.
/// This may indicate that device was physically disconnected or developed a fault.
#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[error("Device lost")]
pub struct DeviceLost;

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
#[track_caller]
pub fn out_of_host_memory() -> ! {
    use std::alloc::{handle_alloc_error, Layout};

    handle_alloc_error(unsafe { Layout::from_size_align_unchecked(1, 1) })
}

/// Handles host OOM the same way global allocator does.
/// This function should be called on host OOM error returned from Vulkan API.
pub fn host_memory_space_overflow() -> ! {
    panic!("Memory address space overflow")
}

fn assert_object<T: Debug + Send + Sync + 'static>() {}
fn assert_error<T: Error + Send + Sync + 'static>() {}

/// Returns minimal aligned integer not smaller than value.
pub fn align_up(align_mask: u64, value: u64) -> Option<u64> {
    Some(value.checked_add(align_mask)? & !align_mask)
}

/// Returns maximal aligned integer not greater than value.
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

mod sealed {
    #[doc(hidden)]
    pub trait Sealed {}
}

trait IteratorExt: Iterator {
    fn filter_min_by_key<B, F>(self, f: F) -> Option<Self::Item>
    where
        Self: Sized,
        B: Ord,
        F: FnMut(&Self::Item) -> Option<B>,
    {
        #[inline]
        fn key<T, B>(mut f: impl FnMut(&T) -> Option<B>) -> impl FnMut(T) -> Option<(B, T)> {
            move |x| Some((f(&x)?, x))
        }

        #[inline]
        fn compare<T, B: Ord>((x_p, _): &(B, T), (y_p, _): &(B, T)) -> Ordering {
            x_p.cmp(y_p)
        }

        let (_, x) = self.filter_map(key(f)).min_by(compare)?;
        Some(x)
    }
}

impl<T> IteratorExt for T where T: Iterator {}
