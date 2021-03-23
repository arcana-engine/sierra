mod layout;

pub use {self::layout::*, crate::backend::DescriptorSet};

use crate::{
    accel::AccelerationStructure, buffer::BufferRegion, image::Layout,
    sampler::Sampler, view::ImageView,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DescriptorSetInfo {
    pub layout: DescriptorSetLayout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WriteDescriptorSet<'a> {
    pub set: &'a DescriptorSet,
    pub binding: u32,
    pub element: u32,
    pub descriptors: Descriptors<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DescriptorImageView {
    pub view: ImageView,
    pub layout: Layout,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CombinedImageSampler {
    pub view: ImageView,
    pub layout: Layout,
    pub sampler: Sampler,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Descriptors<'a> {
    Sampler(&'a [Sampler]),
    CombinedImageSampler(&'a [CombinedImageSampler]),
    SampledImage(&'a [DescriptorImageView]),
    StorageImage(&'a [DescriptorImageView]),
    // UniformTexelBuffer(&'a BufferView),
    // StorageTexelBuffer(&'a BufferView),
    UniformBuffer(&'a [BufferRegion]),
    StorageBuffer(&'a [BufferRegion]),
    UniformBufferDynamic(&'a [BufferRegion]),
    StorageBufferDynamic(&'a [BufferRegion]),
    InputAttachment(&'a [DescriptorImageView]),
    AccelerationStructure(&'a [AccelerationStructure]),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CopyDescriptorSet<'a> {
    pub src: &'a DescriptorSet,
    pub src_binding: u32,
    pub src_element: u32,
    pub dst: &'a DescriptorSet,
    pub dst_binding: u32,
    pub dst_element: u32,
    pub count: u32,
}
