mod buffer;
mod image;
mod layout;
mod sampler;
// mod sparse;

use std::{rc::Rc, sync::Arc};

pub use self::{buffer::*, image::*, layout::* /*, sparse::**/};
pub use crate::{
    backend::{DescriptorSet, WritableDescriptorSet},
    queue::QueueId,
    stage::PipelineStageFlags,
};

use crate::{
    accel::AccelerationStructure, backend::Device, buffer::BufferRange, encode::Encoder,
    image::Layout, sampler::Sampler, view::ImageView, BufferView, OutOfMemory,
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

/// Image view and layout.\
/// Accesses to this descriptor will assume that view
/// is in that layout.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageDescriptor<I> {
    /// Descriptor image resource.
    pub image: I,

    /// View's layout when descriptor is accessed.
    pub layout: Layout,
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
    SampledImage(&'a [ImageDescriptor<ImageView>]),

    /// Storage image descriptors.
    StorageImage(&'a [ImageDescriptor<ImageView>]),

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
    InputAttachment(&'a [ImageDescriptor<ImageView>]),

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
pub enum UniformTexelBufferDescriptor {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum StorageTexelBufferDescriptor {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum InputAttachmentDescriptor {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub enum AccelerationStructureDescriptor {}

#[doc(hidden)]
pub trait Descriptor {
    const TYPE: DescriptorType;
    type RawDescriptor: std::hash::Hash + Eq;

    fn descriptors(slice: &[Self::RawDescriptor]) -> DescriptorSlice<'_>;
}

impl Descriptor for SamplerDescriptor {
    const TYPE: DescriptorType = DescriptorType::Sampler;
    type RawDescriptor = Sampler;

    fn descriptors(slice: &[Sampler]) -> DescriptorSlice<'_> {
        DescriptorSlice::Sampler(slice)
    }
}

impl Descriptor for CombinedImageSamplerDescriptor {
    const TYPE: DescriptorType = DescriptorType::CombinedImageSampler;
    type RawDescriptor = CombinedImageSampler;

    fn descriptors(slice: &[CombinedImageSampler]) -> DescriptorSlice<'_> {
        DescriptorSlice::CombinedImageSampler(slice)
    }
}

impl Descriptor for SampledImageDescriptor {
    const TYPE: DescriptorType = DescriptorType::SampledImage;
    type RawDescriptor = ImageDescriptor<ImageView>;

    fn descriptors(slice: &[ImageDescriptor<ImageView>]) -> DescriptorSlice<'_> {
        DescriptorSlice::SampledImage(slice)
    }
}

impl Descriptor for StorageImageDescriptor {
    const TYPE: DescriptorType = DescriptorType::StorageImage;
    type RawDescriptor = ImageDescriptor<ImageView>;

    fn descriptors(slice: &[ImageDescriptor<ImageView>]) -> DescriptorSlice<'_> {
        DescriptorSlice::StorageImage(slice)
    }
}

impl Descriptor for UniformBufferDescriptor {
    const TYPE: DescriptorType = DescriptorType::UniformBuffer;
    type RawDescriptor = BufferRange;

    fn descriptors(slice: &[BufferRange]) -> DescriptorSlice<'_> {
        DescriptorSlice::UniformBuffer(slice)
    }
}

impl Descriptor for StorageBufferDescriptor {
    const TYPE: DescriptorType = DescriptorType::StorageBuffer;
    type RawDescriptor = BufferRange;

    fn descriptors(slice: &[BufferRange]) -> DescriptorSlice<'_> {
        DescriptorSlice::StorageBuffer(slice)
    }
}

impl Descriptor for UniformBufferDynamicDescriptor {
    const TYPE: DescriptorType = DescriptorType::UniformBufferDynamic;
    type RawDescriptor = BufferRange;

    fn descriptors(slice: &[BufferRange]) -> DescriptorSlice<'_> {
        DescriptorSlice::UniformBufferDynamic(slice)
    }
}

impl Descriptor for StorageBufferDynamicDescriptor {
    const TYPE: DescriptorType = DescriptorType::StorageBufferDynamic;
    type RawDescriptor = BufferRange;

    fn descriptors(slice: &[BufferRange]) -> DescriptorSlice<'_> {
        DescriptorSlice::StorageBufferDynamic(slice)
    }
}

impl Descriptor for UniformTexelBufferDescriptor {
    const TYPE: DescriptorType = DescriptorType::UniformTexelBuffer;
    type RawDescriptor = BufferView;

    fn descriptors(slice: &[BufferView]) -> DescriptorSlice<'_> {
        DescriptorSlice::UniformTexelBuffer(slice)
    }
}

impl Descriptor for StorageTexelBufferDescriptor {
    const TYPE: DescriptorType = DescriptorType::StorageTexelBuffer;
    type RawDescriptor = BufferView;

    fn descriptors(slice: &[BufferView]) -> DescriptorSlice<'_> {
        DescriptorSlice::StorageTexelBuffer(slice)
    }
}

impl Descriptor for InputAttachmentDescriptor {
    const TYPE: DescriptorType = DescriptorType::InputAttachment;
    type RawDescriptor = ImageDescriptor<ImageView>;

    fn descriptors(slice: &[ImageDescriptor<ImageView>]) -> DescriptorSlice<'_> {
        DescriptorSlice::InputAttachment(slice)
    }
}

impl Descriptor for AccelerationStructureDescriptor {
    const TYPE: DescriptorType = DescriptorType::AccelerationStructure;
    type RawDescriptor = AccelerationStructure;

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
pub trait UpdatedPipelineDescriptors<P: ?Sized>: UpdatedDescriptors {
    const N: u32;
}

/// Trait for all types that can be used as descriptor.
pub trait DescriptorBinding {
    /// Number of descriptors in the binding.
    const COUNT: u32;

    /// Flags necessary for this binding type.
    const FLAGS: DescriptorBindingFlags;

    /// Descriptors value.
    type DescriptorArray;

    /// Compare with image view currently bound to descriptor set.
    /// Returns `true` if self is equivalent specified image view,
    /// and no update is required.
    fn eq(&self, descriptors: &Self::DescriptorArray) -> bool;

    /// Returns `Descriptors` equivalent to self.
    fn get_descriptors(&self, device: &Device) -> Result<Self::DescriptorArray, OutOfMemory>;
}

// /// Trait for all types that can be used as descriptor.
// pub trait TypedImageDescriptorBinding {
//     /// Number of descriptors in the binding.
//     const COUNT: u32;

//     /// Flags necessary for this binding type.
//     const FLAGS: DescriptorBindingFlags;

//     /// Descriptors value.
//     type Descriptors: AsRef<[ImageDescriptor<ImageView>]>;

//     /// Compare with image view currently bound to descriptor set.
//     /// Returns `true` if self is equivalent specified image view,
//     /// and no update is required.
//     fn eq(&self, descriptors: &Self::DescriptorArray, layout: Layout) -> bool;

//     /// Returns `Descriptors` equivalent to self.
//     fn get_descriptors(
//         &self,
//         device: &Device,
//         layout: Layout,
//     ) -> Result<Self::DescriptorArray, OutOfMemory>;
// }

// impl<T> DescriptorBinding for T
// where
//     T: TypedImageDescriptorBinding,
// {
//     const COUNT: u32 = T::COUNT;
//     const FLAGS: DescriptorBindingFlags = T::FLAGS;
//     type Descriptors = T::DescriptorArray;

//     #[inline]
//     fn eq(&self, descriptors: &Self::DescriptorArray) -> bool {
//         self.eq(descriptors, Layout::ShaderReadOnlyOptimal)
//     }

//     #[inline]
//     fn get_descriptors(&self, device: &Device) -> Result<Self::DescriptorArray, OutOfMemory> {
//         self.get_descriptors(device, Layout::ShaderReadOnlyOptimal)
//     }
// }

macro_rules! impl_for_refs {
    () => {
        impl_for_refs!(&T);
        impl_for_refs!(&mut T);
        impl_for_refs!(Box<T>);
        impl_for_refs!(Rc<T>);
        impl_for_refs!(Arc<T>);
    };
    ($ref_ty:ty) => {
        impl<T> DescriptorBinding for $ref_ty
        where
            T: DescriptorBinding,
        {
            const COUNT: u32 = T::COUNT;
            const FLAGS: DescriptorBindingFlags = T::FLAGS;
            type DescriptorArray = T::DescriptorArray;

            #[inline]
            fn eq(&self, descriptors: &Self::DescriptorArray) -> bool {
                T::eq(self, descriptors)
            }

            #[inline]
            fn get_descriptors(
                &self,
                device: &Device,
            ) -> Result<Self::DescriptorArray, OutOfMemory> {
                T::get_descriptors(&self, device)
            }
        }
    };
}

impl_for_refs!();
