mod layout;

pub use {self::layout::*, crate::backend::DescriptorSet};

use {
    crate::{
        accel::AccelerationStructure,
        backend::{Device, PipelineLayout},
        buffer::BufferRegion,
        encode::{Encoder, EncoderCommon},
        image::Image,
        image::Layout,
        image::{ImageExtent, ImageSubresourceRange},
        sampler::Sampler,
        view::ImageView,
        view::ImageViewKind,
        OutOfMemory,
    },
    std::convert::TryFrom,
};

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
    UniformBuffer(&'a [BufferRegion]),

    /// Storage buffer regions.
    StorageBuffer(&'a [BufferRegion]),

    /// Dynamic uniform buffer regions.
    UniformBufferDynamic(&'a [BufferRegion]),

    /// Dynamic storage buffer regions.
    StorageBufferDynamic(&'a [BufferRegion]),

    /// Input attachments.
    InputAttachment(&'a [ImageViewDescriptor]),

    /// Acceleration structures.
    AccelerationStructure(&'a [AccelerationStructure]),
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

#[doc(hidden)]
#[derive(Debug)]
pub struct CombinedImageSamplerEq<'a, I> {
    pub image: &'a I,
    pub layout: Layout,
    pub sampler: &'a Sampler,
}

impl<I> Copy for CombinedImageSamplerEq<'_, I> {}
impl<I> Clone for CombinedImageSamplerEq<'_, I> {
    fn clone(&self) -> Self {
        *self
    }
}

impl PartialEq<CombinedImageSampler> for CombinedImageSamplerEq<'_, ImageView> {
    fn eq(&self, rhs: &CombinedImageSampler) -> bool {
        *self.image == rhs.view && self.layout == rhs.layout && *self.sampler == rhs.sampler
    }
}

impl PartialEq<CombinedImageSampler> for CombinedImageSamplerEq<'_, Image> {
    fn eq(&self, rhs: &CombinedImageSampler) -> bool {
        image_eq_view(self.image, &rhs.view)
            && self.layout == rhs.layout
            && *self.sampler == rhs.sampler
    }
}

pub fn image_eq_view(image: &Image, view: &ImageView) -> bool {
    let view_info = view.info();
    let image_info = image.info();

    if view_info.view_kind
        != match image_info.extent {
            ImageExtent::D1 { .. } => ImageViewKind::D1,
            ImageExtent::D2 { .. } => ImageViewKind::D2,
            ImageExtent::D3 { .. } => ImageViewKind::D3,
        }
    {
        return false;
    }

    if view_info.subresource
        != ImageSubresourceRange::new(
            image_info.format.aspect_flags(),
            0..image_info.levels,
            0..image_info.layers,
        )
    {
        return false;
    }

    *image == view_info.image
}

pub trait DescriptorsLayout {
    type Instance;

    fn new(device: &Device) -> Result<Self, OutOfMemory>
    where
        Self: Sized;

    fn raw(&self) -> &DescriptorSetLayout;

    fn instance(&self) -> Self::Instance;
}

pub trait UpdatedDescriptors {
    fn raw(&self) -> &DescriptorSet;
}

pub trait DescriptorsInstance {
    type Input;
    type Updated: UpdatedDescriptors;

    fn update<'a>(
        &'a mut self,
        input: &Self::Input,
        fence: usize,
        device: &Device,
        writes: &mut impl Extend<WriteDescriptorSet<'a>>,
        encoder: &mut Encoder<'a>,
    ) -> Result<&'a Self::Updated, OutOfMemory>;

    fn raw_layout(&self) -> &DescriptorSetLayout;
}

pub trait DescriptorsInput {
    type Layout: DescriptorsLayout<Instance = Self::Instance>;
    type Instance: DescriptorsInstance<Input = Self>;

    fn layout(device: &Device) -> Result<Self::Layout, OutOfMemory> {
        Self::Layout::new(device)
    }
}

pub trait UpdatedPipelineDescriptors<P: ?Sized>: UpdatedDescriptors {
    const N: u32;
}
