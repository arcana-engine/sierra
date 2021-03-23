pub use crate::backend::PhysicalDevice;
use crate::{assert_error, queue::FamilyInfo, OutOfMemory};

/// Error occured during device enumeration.
#[derive(Debug, thiserror::Error)]
pub enum EnumerateDeviceError {
    #[error(transparent)]
    OutOfMemory {
        #[from]
        source: OutOfMemory,
    },
}

/// Contains descriptive information about device.
#[derive(Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceInfo {
    /// Name of the device.
    pub name: String,

    /// Kind of the device.
    pub kind: Option<DeviceKind>,

    /// Features supported by device.
    pub features: Vec<Feature>,

    /// Information about queue families that device has.
    pub families: Vec<FamilyInfo>,
}

/// Kind of the device.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum DeviceKind {
    /// Device is sowtware emulated.
    Software,

    /// Device is integrate piece of hardware (typically into CPU)
    Integrated,

    /// Device is discrete piece of hardware.
    Discrete,
}

/// Features that optionally can be supported by devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum Feature {
    BufferDeviceAddress,

    ShaderSampledImageDynamicIndexing,
    ShaderStorageImageDynamicIndexing,
    ShaderUniformBufferDynamicIndexing,
    ShaderStorageBufferDynamicIndexing,

    ShaderSampledImageNonUniformIndexing,
    ShaderStorageImageNonUniformIndexing,
    ShaderUniformBufferNonUniformIndexing,
    ShaderStorageBufferNonUniformIndexing,

    DescriptorBindingSampledImageUpdateAfterBind,
    DescriptorBindingStorageImageUpdateAfterBind,
    DescriptorBindingStorageBufferUpdateAfterBind,
    DescriptorBindingStorageTexelBufferUpdateAfterBind,
    DescriptorBindingUniformBufferUpdateAfterBind,
    DescriptorBindingUniformTexelBufferUpdateAfterBind,
    DescriptorBindingUpdateUnusedWhilePending,
    DescriptorBindingPartiallyBound,
    AccelerationStructure,
    RayTracingPipeline,
    RuntimeDescriptorArray,
    ScalarBlockLayout,
    SurfacePresentation,
}

#[allow(dead_code)]
fn check() {
    assert_error::<EnumerateDeviceError>();
}
