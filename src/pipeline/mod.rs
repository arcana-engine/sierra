mod compute;
mod graphics;
mod ray_tracing;

pub use {
    self::{compute::*, graphics::*, ray_tracing::*},
    crate::{
        backend::PipelineLayout,
        descriptor::{UpdatedDescriptors, UpdatedPipelineDescriptors},
        encode::{Encoder, EncoderCommon},
    },
};

use bytemuck::Pod;

use crate::{descriptor::DescriptorSetLayout, shader::ShaderStageFlags, Device, OutOfMemory};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PushConstant {
    pub stages: ShaderStageFlags,
    pub offset: u32,
    pub size: u32,
}

/// Defines layout of pipeline inputs: all descriptor sets and push constants.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PipelineLayoutInfo {
    /// Array of descriptor set layouts.
    pub sets: Vec<DescriptorSetLayout>,
    pub push_constants: Vec<PushConstant>,
}

/// [`PipelineLayout`] for specific [`PipelineInput`].
pub trait PipelineInputLayout {
    fn new(device: &Device) -> Result<Self, OutOfMemory>
    where
        Self: Sized;

    fn raw(&self) -> &PipelineLayout;

    fn bind_graphics<D, const N: u32>(
        &self,
        updated_descriptors: &D,
        encoder: &mut EncoderCommon<'_>,
    ) where
        D: UpdatedPipelineDescriptors<Self, N>;

    fn bind_compute<D, const N: u32>(
        &self,
        updated_descriptors: &D,
        encoder: &mut EncoderCommon<'_>,
    ) where
        D: UpdatedPipelineDescriptors<Self, N>;

    fn bind_ray_tracing<D, const N: u32>(
        &self,
        updated_descriptors: &D,
        encoder: &mut EncoderCommon<'_>,
    ) where
        D: UpdatedPipelineDescriptors<Self, N>;

    fn push_constants<P>(&self, push_constants: &P, encoder: &mut EncoderCommon<'_>)
    where
        P: PipelinePushConstants<Self>;
}

pub trait PipelineInput {
    type Layout: PipelineInputLayout;

    fn layout(device: &Device) -> Result<Self::Layout, OutOfMemory>;
}

/// Extension trait for push constants, specifying stages, offset and size in the typed pipeline.
///
/// This trait is intended to be implemented by proc macro `#[derive(Pipeline)]`
/// for types marked as `#[push]`.
pub trait PipelinePushConstants<P: ?Sized> {
    /// Stage flags for which push constants are enabled.
    const STAGES: ShaderStageFlags;

    /// Offset of the instance of push constants.
    const OFFSET: u32;

    /// Shader repr type matching push constants layout.
    type Repr: Pod;

    /// Function to convert push constants into correct repr.
    fn to_repr(&self) -> Self::Repr;
}
