mod binding;
mod layout;
mod sparse;

pub use self::{binding::*, layout::*, sparse::*};

pub use crate::{
    backend::{DescriptorSet, WritableDescriptorSet},
    queue::QueueId,
    stage::PipelineStages,
};

use crate::{
    accel::AccelerationStructure, backend::Device, buffer::BufferRange, encode::Encoder,
    image::Layout, sampler::Sampler, sealed::Sealed, view::ImageView, BufferView, General,
    OutOfMemory, ShaderReadOnlyOptimal, StaticLayout,
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
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct UpdateDescriptorSet<'a> {
    /// Target descriptor set.
    pub set: &'a mut WritableDescriptorSet,

    /// Writes to the descriptor set.
    pub writes: &'a [DescriptorSetWrite<'a>],

    /// Writes to the descriptor set.
    pub copies: &'a [DescriptorSetCopy<'a>],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DescriptorSetWrite<'a> {
    /// Binding index.
    pub binding: u32,

    /// First element index.
    /// Must be zero for non-array bindings.
    pub element: u32,

    /// Descriptors to write.
    pub descriptors: DescriptorSlice<'a>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DescriptorSetCopy<'a> {
    /// Source set from where descriptors are copied.
    pub src: &'a DescriptorSet,

    /// First binding to copy descriptors from.
    pub src_binding: u32,

    /// First array element of first binding to copy descriptors from.
    pub src_element: u32,

    /// First binding to copy descriptors to.
    pub dst_binding: u32,

    /// First array element of first binding to copy descriptors to.
    pub dst_element: u32,

    /// Number of descriptors to copy.
    pub count: u32,
}

/// Image view, layout and sampler.\
/// Unlike [`ImageViewDescriptor`] this descriptor contains a sampler.
/// to do sampled reads.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CombinedImageSampler {
    /// Descriptor image resource.
    pub view: ImageView,

    /// View's layout when descriptor is accessed.
    pub layout: Layout,

    /// Descriptor sampler resource.
    pub sampler: Sampler,
}

/// Collection of descriptors.\
/// This type is used in [`WriteDescriptorSet`] to specify descriptors
/// to write.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DescriptorSlice<'a> {
    /// Samplers.
    Sampler(&'a [Sampler]),

    /// Combined image and sampler descriptors.
    CombinedImageSampler(&'a [CombinedImageSampler]),

    /// Sampled image descriptors.
    SampledImage(&'a [(ImageView, Layout)]),

    /// Storage image descriptors.
    StorageImage(&'a [(ImageView, Layout)]),

    /// Uniform texel buffer descriptors.
    UniformTexelBuffer(&'a [BufferView]),

    /// Storage texel buffer descriptors.
    StorageTexelBuffer(&'a [BufferView]),

    /// Uniform buffer regions.
    UniformBuffer(&'a [BufferRange]),

    /// Storage buffer regions.
    StorageBuffer(&'a [BufferRange]),

    /// Dynamic uniform buffer regions.
    UniformBufferDynamic(&'a [BufferRange]),

    /// Dynamic storage buffer regions.
    StorageBufferDynamic(&'a [BufferRange]),

    /// Input attachments.
    InputAttachment(&'a [(ImageView, Layout)]),

    /// Acceleration structures.
    AccelerationStructure(&'a [AccelerationStructure]),
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum DynamicFormat {}

pub trait ValidLayout<S>: StaticLayout {}

impl ValidLayout<Sampled> for ShaderReadOnlyOptimal {}
impl ValidLayout<Sampled> for General {}
impl ValidLayout<Storage> for General {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum DynamicLayout {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum DynamicOffset {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum StaticOffset {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum Uniform {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum Storage {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum Sampled {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum SamplerDescriptor {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum CombinedImageSamplerDescriptor {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct ImageDescriptor<S = Sampled, L = ShaderReadOnlyOptimal> {
    pub storage: S,
    pub layout: L,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct BufferDescriptor<S, O = StaticOffset> {
    pub storage: S,
    pub offset: O,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct TexelBufferDescriptor<S, F> {
    pub storage: S,
    pub format: F,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum InputAttachmentDescriptor {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum AccelerationStructureDescriptor {}

#[doc(hidden)]
pub trait DescriptorKind: Sealed {
    const TYPE: DescriptorType;
    type Descriptor: std::hash::Hash + Eq;

    fn descriptors(slice: &[Self::Descriptor]) -> DescriptorSlice<'_>;
}

impl Sealed for SamplerDescriptor {}
impl DescriptorKind for SamplerDescriptor {
    const TYPE: DescriptorType = DescriptorType::Sampler;
    type Descriptor = Sampler;

    fn descriptors(slice: &[Sampler]) -> DescriptorSlice<'_> {
        DescriptorSlice::Sampler(slice)
    }
}

impl Sealed for CombinedImageSamplerDescriptor {}
impl DescriptorKind for CombinedImageSamplerDescriptor {
    const TYPE: DescriptorType = DescriptorType::CombinedImageSampler;
    type Descriptor = CombinedImageSampler;

    fn descriptors(slice: &[CombinedImageSampler]) -> DescriptorSlice<'_> {
        DescriptorSlice::CombinedImageSampler(slice)
    }
}

impl<L> Sealed for ImageDescriptor<Sampled, L> {}
impl<L> DescriptorKind for ImageDescriptor<Sampled, L> {
    const TYPE: DescriptorType = DescriptorType::SampledImage;
    type Descriptor = (ImageView, Layout);

    fn descriptors(slice: &[(ImageView, Layout)]) -> DescriptorSlice<'_> {
        DescriptorSlice::SampledImage(slice)
    }
}

impl<L> Sealed for ImageDescriptor<Storage, L> {}
impl<L> DescriptorKind for ImageDescriptor<Storage, L> {
    const TYPE: DescriptorType = DescriptorType::StorageImage;
    type Descriptor = (ImageView, Layout);

    fn descriptors(slice: &[(ImageView, Layout)]) -> DescriptorSlice<'_> {
        DescriptorSlice::StorageImage(slice)
    }
}

impl Sealed for BufferDescriptor<Uniform> {}
impl DescriptorKind for BufferDescriptor<Uniform> {
    const TYPE: DescriptorType = DescriptorType::UniformBuffer;
    type Descriptor = BufferRange;

    fn descriptors(slice: &[BufferRange]) -> DescriptorSlice<'_> {
        DescriptorSlice::UniformBuffer(slice)
    }
}

impl Sealed for BufferDescriptor<Storage> {}
impl DescriptorKind for BufferDescriptor<Storage> {
    const TYPE: DescriptorType = DescriptorType::StorageBuffer;
    type Descriptor = BufferRange;

    fn descriptors(slice: &[BufferRange]) -> DescriptorSlice<'_> {
        DescriptorSlice::StorageBuffer(slice)
    }
}

impl Sealed for BufferDescriptor<Uniform, DynamicOffset> {}
impl DescriptorKind for BufferDescriptor<Uniform, DynamicOffset> {
    const TYPE: DescriptorType = DescriptorType::UniformBufferDynamic;
    type Descriptor = BufferRange;

    fn descriptors(slice: &[BufferRange]) -> DescriptorSlice<'_> {
        DescriptorSlice::UniformBufferDynamic(slice)
    }
}

impl Sealed for BufferDescriptor<Storage, DynamicOffset> {}
impl DescriptorKind for BufferDescriptor<Storage, DynamicOffset> {
    const TYPE: DescriptorType = DescriptorType::StorageBufferDynamic;
    type Descriptor = BufferRange;

    fn descriptors(slice: &[BufferRange]) -> DescriptorSlice<'_> {
        DescriptorSlice::StorageBufferDynamic(slice)
    }
}

impl<F> Sealed for TexelBufferDescriptor<Uniform, F> {}
impl<F> DescriptorKind for TexelBufferDescriptor<Uniform, F> {
    const TYPE: DescriptorType = DescriptorType::UniformTexelBuffer;
    type Descriptor = BufferView;

    fn descriptors(slice: &[BufferView]) -> DescriptorSlice<'_> {
        DescriptorSlice::UniformTexelBuffer(slice)
    }
}

impl<F> Sealed for TexelBufferDescriptor<Storage, F> {}
impl<F> DescriptorKind for TexelBufferDescriptor<Storage, F> {
    const TYPE: DescriptorType = DescriptorType::StorageTexelBuffer;
    type Descriptor = BufferView;

    fn descriptors(slice: &[BufferView]) -> DescriptorSlice<'_> {
        DescriptorSlice::StorageTexelBuffer(slice)
    }
}

impl Sealed for InputAttachmentDescriptor {}
impl DescriptorKind for InputAttachmentDescriptor {
    const TYPE: DescriptorType = DescriptorType::InputAttachment;
    type Descriptor = (ImageView, Layout);

    fn descriptors(slice: &[(ImageView, Layout)]) -> DescriptorSlice<'_> {
        DescriptorSlice::InputAttachment(slice)
    }
}

impl Sealed for AccelerationStructureDescriptor {}
impl DescriptorKind for AccelerationStructureDescriptor {
    const TYPE: DescriptorType = DescriptorType::AccelerationStructure;
    type Descriptor = AccelerationStructure;

    fn descriptors(slice: &[AccelerationStructure]) -> DescriptorSlice<'_> {
        DescriptorSlice::AccelerationStructure(slice)
    }
}

/// Trait for descriptor layouts.
///
/// This trait is intended to be implemented by proc macro `#[derive(Descriptors)]` for generated types.
pub trait DescriptorsLayout {
    type Instance;

    fn raw(&self) -> &DescriptorSetLayout;

    fn instance(&self) -> Self::Instance;
}

/// Trait for descriptors updated and ready to be bound to pipeline.
///
/// This trait is intended to be implemented by proc macro `#[derive(Descriptors)]` for generated types.
pub trait UpdatedDescriptors {
    fn raw(&self) -> &DescriptorSet;
}

/// Trait for descriptors instance.
///
/// This trait is intended to be implemented by proc macro `#[derive(Descriptors)]` for generated types.
pub trait DescriptorsInstance<I: ?Sized> {
    type Updated: UpdatedDescriptors;

    /// Performs necessary updates to the descriptors according to the input.
    /// Returns update descriptors instance that can be bound to the encoder with correct pipline.
    fn update(
        &mut self,
        input: &I,
        device: &Device,
        encoder: &mut Encoder,
    ) -> Result<&Self::Updated, DescriptorsAllocationError>;

    fn raw_layout(&self) -> &DescriptorSetLayout;
}

/// Input structures for descriptors implement this trait.
///
/// This trait is intended to be implemented by proc macro `#[derive(Descriptors)]`.
pub trait Descriptors {
    /// Layout type for the input.
    ///
    /// Proc macro `#[derive(Descriptors)]` generates this type and all necessary code.
    type Layout;

    /// Instance type for the input.
    ///
    /// Proc macro `#[derive(Descriptors)]` generates this type and all necessary code.
    type Instance: DescriptorsInstance<Self>;

    /// Shortcut for instantiating layout for the input type.
    fn layout(device: &Device) -> Result<Self::Layout, OutOfMemory>;
}

/// Extension trait for updated descriptors, specifying offset in typed pipeline.
///
/// This trait is intended to be implemented by proc macro `#[derive(Pipeline)]`
/// for types generated by proc macro `#[derive(Descriptors)]`.
pub trait UpdatedPipelineDescriptors<P: ?Sized, const N: u32>: UpdatedDescriptors {}
