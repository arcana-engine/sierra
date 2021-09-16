mod buffer;
mod image;
mod layout;
mod sampler;
// mod sparse;

pub use {
    self::{buffer::*, image::*, layout::* /*, sparse::**/},
    crate::{backend::DescriptorSet, queue::QueueId, stage::PipelineStageFlags},
};

use crate::{
    accel::AccelerationStructure,
    backend::Device,
    buffer::BufferRange,
    encode::Encoder,
    // image::Image,
    image::RawLayout,
    // image::{ImageExtent, SubresourceRange},
    sampler::Sampler,
    view::ImageView,
    // view::ImageViewKind,
    OutOfMemory,
};

/// AllocationError that may occur during descriptor sets allocation.
#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum DescriptorsAllocationError {
    /// Out of device memory
    #[error(transparent)]
    OutOfMemory {
        #[from]
        source: OutOfMemory,
    },

    /// The total number of descriptors across all pools created\
    /// with flag `CREATE_UPDATE_AFTER_BIND_BIT` set exceeds `max_update_after_bind_descriptors_in_all_pools`
    /// Or fragmentation of the underlying hardware resources occurs.
    #[error("Failed to allocate descriptors due to fragmentation")]
    Fragmentation,
}

/// Contains information required to create `DescriptorSet` instance.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DescriptorSetInfo {
    /// Layout of the descriptor set to create.
    pub layout: DescriptorSetLayout,
}

/// Defines how to write descriptors into set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WriteDescriptorSet<'a> {
    /// Target descriptor set.
    pub set: &'a DescriptorSet,

    /// Binding index.
    pub binding: u32,

    /// First element index.
    /// Must be zero for non-array bindings.
    pub element: u32,

    /// Descriptors to write.
    pub descriptors: Descriptors<'a>,
}

/// Image view and layout.\
/// Accesses to this descriptor will assume that view
/// is in that layout.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageViewDescriptor {
    /// Descriptor image resource.
    pub view: ImageView,

    /// View's layout when descriptor is accessed.
    pub layout: RawLayout,
}

/// Image view, layout and sampler.\
/// Unlike [`ImageViewDescriptor`] this descriptor contains a sampler.
/// to do sampled reads.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CombinedImageSampler {
    /// Descriptor image resource.
    pub view: ImageView,

    /// View's layout when descriptor is accessed.
    pub layout: RawLayout,

    /// Descriptor sampler resource.
    pub sampler: Sampler,
}

/// Collection of descriptors.\
/// This type is used in [`WriteDescriptorSet`] to specify descriptors
/// to write.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Descriptors<'a> {
    /// Samplers.
    Sampler(&'a [Sampler]),

    /// Combined image and sampler descriptors.
    CombinedImageSampler(&'a [CombinedImageSampler]),

    /// Sampled image descriptors.
    SampledImage(&'a [ImageViewDescriptor]),

    /// Storage image descriptors.
    StorageImage(&'a [ImageViewDescriptor]),

    // UniformTexelBuffer(&'a BufferView),
    // StorageTexelBuffer(&'a BufferView),
    /// Uniform buffer regions.
    UniformBuffer(&'a [BufferRange]),

    /// Storage buffer regions.
    StorageBuffer(&'a [BufferRange]),

    /// Dynamic uniform buffer regions.
    UniformBufferDynamic(&'a [BufferRange]),

    /// Dynamic storage buffer regions.
    StorageBufferDynamic(&'a [BufferRange]),

    /// Input attachments.
    InputAttachment(&'a [ImageViewDescriptor]),

    /// Acceleration structures.
    AccelerationStructure(&'a [AccelerationStructure]),
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum SamplerDescriptor {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum CombinedImageSamplerDescriptor {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum SampledImageDescriptor {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum StorageImageDescriptor {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum UniformBufferDescriptor {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum StorageBufferDescriptor {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum UniformBufferDynamicDescriptor {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum StorageBufferDynamicDescriptor {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum InputAttachmentDescriptor {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum AccelerationStructureDescriptor {}

#[doc(hidden)]
pub trait TypedDescriptor {
    const TYPE: DescriptorType;
    type Descriptor: std::hash::Hash + Eq;

    fn descriptors<'a>(slice: &'a [Self::Descriptor]) -> Descriptors<'a>;
}

impl TypedDescriptor for SamplerDescriptor {
    const TYPE: DescriptorType = DescriptorType::Sampler;
    type Descriptor = Sampler;

    fn descriptors<'a>(slice: &'a [Sampler]) -> Descriptors<'a> {
        Descriptors::Sampler(slice)
    }
}

impl TypedDescriptor for CombinedImageSamplerDescriptor {
    const TYPE: DescriptorType = DescriptorType::CombinedImageSampler;
    type Descriptor = CombinedImageSampler;

    fn descriptors<'a>(slice: &'a [CombinedImageSampler]) -> Descriptors<'a> {
        Descriptors::CombinedImageSampler(slice)
    }
}

impl TypedDescriptor for SampledImageDescriptor {
    const TYPE: DescriptorType = DescriptorType::SampledImage;
    type Descriptor = ImageViewDescriptor;

    fn descriptors<'a>(slice: &'a [ImageViewDescriptor]) -> Descriptors<'a> {
        Descriptors::SampledImage(slice)
    }
}

impl TypedDescriptor for StorageImageDescriptor {
    const TYPE: DescriptorType = DescriptorType::StorageImage;
    type Descriptor = ImageViewDescriptor;

    fn descriptors<'a>(slice: &'a [ImageViewDescriptor]) -> Descriptors<'a> {
        Descriptors::StorageImage(slice)
    }
}

impl TypedDescriptor for UniformBufferDescriptor {
    const TYPE: DescriptorType = DescriptorType::UniformBuffer;
    type Descriptor = BufferRange;

    fn descriptors<'a>(slice: &'a [BufferRange]) -> Descriptors<'a> {
        Descriptors::UniformBuffer(slice)
    }
}

impl TypedDescriptor for StorageBufferDescriptor {
    const TYPE: DescriptorType = DescriptorType::StorageBuffer;
    type Descriptor = BufferRange;

    fn descriptors<'a>(slice: &'a [BufferRange]) -> Descriptors<'a> {
        Descriptors::StorageBuffer(slice)
    }
}

impl TypedDescriptor for UniformBufferDynamicDescriptor {
    const TYPE: DescriptorType = DescriptorType::UniformBufferDynamic;
    type Descriptor = BufferRange;

    fn descriptors<'a>(slice: &'a [BufferRange]) -> Descriptors<'a> {
        Descriptors::UniformBufferDynamic(slice)
    }
}

impl TypedDescriptor for StorageBufferDynamicDescriptor {
    const TYPE: DescriptorType = DescriptorType::StorageBufferDynamic;
    type Descriptor = BufferRange;

    fn descriptors<'a>(slice: &'a [BufferRange]) -> Descriptors<'a> {
        Descriptors::StorageBufferDynamic(slice)
    }
}

impl TypedDescriptor for InputAttachmentDescriptor {
    const TYPE: DescriptorType = DescriptorType::InputAttachment;
    type Descriptor = ImageViewDescriptor;

    fn descriptors<'a>(slice: &'a [ImageViewDescriptor]) -> Descriptors<'a> {
        Descriptors::InputAttachment(slice)
    }
}

impl TypedDescriptor for AccelerationStructureDescriptor {
    const TYPE: DescriptorType = DescriptorType::AccelerationStructure;
    type Descriptor = AccelerationStructure;

    fn descriptors<'a>(slice: &'a [AccelerationStructure]) -> Descriptors<'a> {
        Descriptors::AccelerationStructure(slice)
    }
}

/// Defines operation to copy descriptors range from one set to another.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CopyDescriptorSet<'a> {
    /// Source set from where descriptors are copied.
    pub src: &'a DescriptorSet,

    /// First binding to copy descriptors from.
    pub src_binding: u32,

    /// First array element of first binding to copy descriptors from.
    pub src_element: u32,

    /// Destination set into which descriptors are copied.
    pub dst: &'a DescriptorSet,

    /// First binding to copy descriptors to.
    pub dst_binding: u32,

    /// First array element of first binding to copy descriptors to.
    pub dst_element: u32,

    /// Number of descriptors to copy.
    pub count: u32,
}

/// Trait for descriptor layouts.
///
/// This trait is intended to be implemented by proc macro `#[descriptors]` for generated types.
pub trait DescriptorsLayout {
    type Instance;

    fn raw(&self) -> &DescriptorSetLayout;

    fn instance(&self) -> Self::Instance;
}

/// Trait for descriptors updated and ready to be bound to pipeline.
///
/// This trait is intended to be implemented by proc macro `#[descriptors]` for generated types.
pub trait UpdatedDescriptors {
    fn raw(&self) -> &DescriptorSet;
}

/// Trait for descriptors instance.
///
/// This trait is intended to be implemented by proc macro `#[descriptors]` for generated types.
pub trait DescriptorsInstance<I: ?Sized> {
    type Updated: UpdatedDescriptors;

    fn update<'a>(
        &'a mut self,
        input: &I,
        fence: usize,
        device: &Device,
        writes: &mut impl Extend<WriteDescriptorSet<'a>>,
        encoder: &mut Encoder<'a>,
    ) -> Result<&'a Self::Updated, DescriptorsAllocationError>;

    fn raw_layout(&self) -> &DescriptorSetLayout;
}

/// Input structures for descriptors implement this trait.
///
/// This trait is intended to be implemented by proc macro `#[descriptors]`.
pub trait DescriptorsInput {
    /// Layout type for the input.
    ///
    /// Proc macro `#[descriptors]` generates this type and all necessary code.
    type Layout;

    /// Instance type for the input.
    ///
    /// Proc macro `#[descriptors]` generates this type and all necessary code.
    type Instance: DescriptorsInstance<Self>;

    /// Shortcut for instantiating layout for the input type.
    fn layout(device: &Device) -> Result<Self::Layout, OutOfMemory>;
}

/// Extension trait for updated descriptors, specifying offset in typed pipeline.
///
/// This trait is intended to be implemented by proc macro `#[pipeline]`
/// for types generated by proc macro `#[descriptors]`.
pub trait UpdatedPipelineDescriptors<P: ?Sized>: UpdatedDescriptors {
    const N: u32;
}

/// Trait for all types that can be used as descriptor.
pub trait TypedDescriptorBinding {
    /// Number of descriptors in the binding.
    const COUNT: u32;

    /// Flags necessary for this binding type.
    const FLAGS: DescriptorBindingFlags;

    /// Descriptors value.
    type Descriptors;

    /// Compare with image view currently bound to descriptor set.
    /// Returns `true` if self is equivalent specified image view,
    /// and no update is required.
    fn eq(&self, descriptors: &Self::Descriptors) -> bool;

    /// Returns `BufferRange` equivalent to self.
    fn get_descriptors(&self, device: &Device) -> Result<Self::Descriptors, OutOfMemory>;
}
