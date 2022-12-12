pub use crate::backend::PhysicalDevice;
use crate::{assert_error, queue::FamilyInfo, OutOfMemory};

/// Error occured during device enumeration.
#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
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
    /// Allows taking GPU 64-bit address for a buffer
    /// pass it as plain data to the shader
    /// and access it there.
    BufferDeviceAddress,

    /// Allows using indices that depend on shader input to access sampled image arrays.
    /// `ShaderSampledImageNonUniformIndexing` is required to use indices that depend on non-uniform values.
    ShaderSampledImageDynamicIndexing,

    /// Allows using indices that depend on shader input to access storage image arrays.
    /// `ShaderStorageImageNonUniformIndexing` is required to use indices that depend on non-uniform values.
    ShaderStorageImageDynamicIndexing,

    /// Allows using indices that depend on shader input to access uniform buffer arrays.
    /// `ShaderUniformBufferNonUniformIndexing` is required to use indices that depend on non-uniform values.
    ShaderUniformBufferDynamicIndexing,

    /// Allows using indices that depend on shader input to access storage buffer arrays.
    /// `ShaderStorageBufferNonUniformIndexing` is required to use indices that depend on non-uniform values.
    ShaderStorageBufferDynamicIndexing,

    /// Allows using indices that depend on non-uniform shader input to access sampled image arrays.
    ShaderSampledImageNonUniformIndexing,

    /// Allows using indices that depend on non-uniform shader input to access storage image arrays.
    ShaderStorageImageNonUniformIndexing,

    /// Allows using indices that depend on non-uniform shader input to access uniform buffer arrays.
    ShaderUniformBufferNonUniformIndexing,

    /// Allows using indices that depend on non-uniform shader input to access storage buffer arrays.
    ShaderStorageBufferNonUniformIndexing,

    /// Allows using `DescriptorBindingFlags::UPDATE_AFTER_BIND` flag on sampled image descriptors.
    DescriptorBindingSampledImageUpdateAfterBind,

    /// Allows using `DescriptorBindingFlags::UPDATE_AFTER_BIND` flag on storage image descriptors.
    DescriptorBindingStorageImageUpdateAfterBind,

    /// Allows using `DescriptorBindingFlags::UPDATE_AFTER_BIND` flag on uniform buffer descriptors.
    DescriptorBindingUniformBufferUpdateAfterBind,

    /// Allows using `DescriptorBindingFlags::UPDATE_AFTER_BIND` flag on storage buffer descriptors.
    DescriptorBindingStorageBufferUpdateAfterBind,

    /// Allows using `DescriptorBindingFlags::UPDATE_AFTER_BIND` flag on uniform texel buffer descriptors.
    DescriptorBindingUniformTexelBufferUpdateAfterBind,

    /// Allows using `DescriptorBindingFlags::UPDATE_AFTER_BIND` flag on storage texel buffer descriptors.
    DescriptorBindingStorageTexelBufferUpdateAfterBind,

    /// Allows using `DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING` flag on descriptors.
    DescriptorBindingUpdateUnusedWhilePending,

    /// Allows using `DescriptorBindingFlags::PARTIALLY_BOUND` flag on descriptors.
    DescriptorBindingPartiallyBound,

    /// Allows creation, building and usage of acceleration structures.
    AccelerationStructure,

    /// Allows creation and usage of ray-tracing pipelines.
    RayTracingPipeline,

    /// Allows creating runtime sized arrays of descriptors.
    RuntimeDescriptorArray,

    ScalarBlockLayout,

    /// Allows creating surface and swapchain to display images.
    SurfacePresentation,

    /// Allows fetching display timings.
    DisplayTiming,

    /// Allows rendering without render-pass.
    DynamicRendering,

    /// Allows moving depth and stencil aspects of a image into different layouts.
    SeparateDepthStencilLayouts,
}

#[allow(dead_code)]
fn check() {
    assert_error::<EnumerateDeviceError>();
}
