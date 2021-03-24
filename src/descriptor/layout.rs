pub use crate::backend::DescriptorSetLayout;
use crate::shader::ShaderStageFlags;

bitflags::bitflags! {
    /// Flags that can be set in each [`DescriptorSetLayoutBinding`]
    /// to specify options for the corresponding descriptor set layout binding.
    /// Note that Vulkan 1.2 is required for any of these flags.
    /// That is, the only valid value prior Vulkan 1.2 is `DescriptorBindingFlags::empty()`.
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct DescriptorBindingFlags: u32 {
        /// Allows update binding after set is bound to encoder.
        /// Updating binding without this flag would invalidate encoder or built command buffer
        /// where set is used.
        const UPDATE_AFTER_BIND = 0x00000001;

        /// Allows updating descriptors in this binding that are not used.
        /// while set is bound to pending command buffer.\
        /// i.e. when shader may access other descriptors in the set.
        ///
        /// If [`DescriptorBindingFlags::PARTIALLY_BOUND`] is also set then descriptors that are not
        /// dynamically used by any shader invocation can be updated.
        /// Otherwise only descriptors that are not statically used
        /// by any shader invocation can be updated.
        const UPDATE_UNUSED_WHILE_PENDING = 0x00000002;

        /// Allows descriptors that are not dynamically used by
        /// any shader invocation to be unbound.
        const PARTIALLY_BOUND = 0x00000004;

        /// Binding with this flag does not have descriptors count defined by layout.
        /// Instead count is specified when set instance is created.
        /// Allowing sets with same layout to have differently sized
        /// arrays of descriptors bound to the binding.
        const VARIABLE_DESCRIPTOR_COUNT = 0x00000008;
    }
}

bitflags::bitflags! {
    /// Flags that can be sed to
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct DescriptorSetLayoutFlags: u32 {
        /// Specifies that set with this layout must not be allocated.
        /// And descriptors should be pushed to encoder directly.
        const PUSH_DESCRIPTOR = 0x00000001;

        /// Allows bindings in this layout to have [`DescriptorBindingFlags::UPDATE_AFTER_BIND`] flags.
        const UPDATE_AFTER_BIND_POOL = 0x00000002;
    }
}

/// Defines layout for descriptor sets.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct DescriptorSetLayoutInfo {
    /// Array of bindings in this layout.
    /// Every element must have different `.binding` field.
    pub bindings: Vec<DescriptorSetLayoutBinding>,

    /// Flags to specify options for the descriptor set layout.
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

    /// Number of descriptors in the binding.
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
    /// Sampler descriptor.\
    /// Contains [`Sampler`] instances.
    Sampler,

    /// Combined image and sampler.\
    /// Contains both [`ImageView`] and [`Sampler`] instances.
    CombinedImageSampler,

    /// Image that can be sampled.
    /// Contains [`ImageView`] instance.
    SampledImage,

    /// Image that can be used as storage.
    /// Allows accessing individual pixels.
    /// Unlike [`SampledImage`] [`StorageImage`] can be overwritten by shader.
    StorageImage,

    /// Buffer with shader uniform data.
    UniformBuffer,

    /// Buffer that can be used as storage.
    /// Unlike [`UniformBuffer`] [`StorageBuffer`] can be overwritten by shader.
    StorageBuffer,

    /// Same as [`UniformBuffer`] but allows specifying offset each time set is bound to encoder.
    UniformBufferDynamic,

    /// Same as [`StorageBuffer`] but allows specifying offset each time set is bound to encoder.
    StorageBufferDynamic,

    /// Input attachment descriptor is an image with restricted access.
    /// Only fragment shader can read from input attachment.
    /// And only to the fragment's location.
    /// Must correspond to input attachment configured in render-pass.
    InputAttachment,

    /// Acceleration structure for ray-tracing shaders and ray queries.
    AccelerationStructure,
}
