mod layout;

pub use {self::layout::*, crate::backend::DescriptorSet};

use crate::{
    accel::AccelerationStructure, buffer::BufferRegion, image::Layout, sampler::Sampler,
    view::ImageView,
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
