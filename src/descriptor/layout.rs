pub use crate::backend::DescriptorSetLayout;
use crate::shader::ShaderStageFlags;

bitflags::bitflags! {
    /// Bits which can be set in each element of VkDescriptorSetLayoutBindingFlagsCreateInfo::pBindingFlags to specify options for the corresponding descriptor set layout binding are:
    /// Note that Vulkan 1.2 is required for any of the flags.
    // That is, the only valid value prior Vulkan 1.2 is `DescriptorBindingFlags::empty()`.
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct DescriptorBindingFlags: u32 {
        const UPDATE_AFTER_BIND = 0x00000001;
        const UPDATE_UNUSED_WHILE_PENDING = 0x00000002;
        const PARTIALLY_BOUND = 0x00000004;
        const VARIABLE_DESCRIPTOR_COUNT = 0x00000008;
    }
}

bitflags::bitflags! {
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct DescriptorSetLayoutFlags: u32 {
        const PUSH_DESCRIPTOR = 0x00000001;
        const UPDATE_AFTER_BIND_POOL = 0x00000002;
    }
}

/// Defines layout for descriptor sets.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct DescriptorSetLayoutInfo {
    pub bindings: Vec<DescriptorSetLayoutBinding>,
    pub flags: DescriptorSetLayoutFlags,
}

/// Defines layout for one binding in descriptor set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct DescriptorSetLayoutBinding {
    /// Binding index.
    pub binding: u32,

    /// Type of descriptor in the binding.
    pub ty: DescriptorType,

    /// Number of dfescriptors in the binding.
    pub count: u32,

    /// Shader stages where this binding is accessible.
    pub stages: ShaderStageFlags,

    /// Flags to specify options for the descriptor set layout binding.
    pub flags: DescriptorBindingFlags,
}

/// Types of descriptors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum DescriptorType {
    Sampler,
    CombinedImageSampler,
    SampledImage,
    StorageImage,
    UniformTexelBuffer,
    StorageTexelBuffer,
    UniformBuffer,
    StorageBuffer,
    UniformBufferDynamic,
    StorageBufferDynamic,
    InputAttachment,
    AccelerationStructure,
}
