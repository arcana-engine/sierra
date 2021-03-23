pub use crate::backend::RayTracingPipeline;
use crate::{buffer::StridedBufferRegion, shader::Shader, PipelineLayout};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RayTracingPipelineInfo {
    /// Array of shaders referenced by indices in shader groups below.
    pub shaders: Vec<Shader>,

    /// Pipline-creation-time layer of indirection between individual shaders
    /// and acceleration structures.
    pub groups: Vec<RayTracingShaderGroupInfo>,

    /// Maximum recursion depth to trace rays.
    pub max_recursion_depth: u32,

    /// Pipeline layout.
    pub layout: PipelineLayout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum RayTracingShaderGroupInfo {
    Raygen {
        /// Index of raygen shader in `RayTracingPipelineInfo::shaders`.
        raygen: u32,
    },
    Miss {
        /// Index of miss shader in `RayTracingPipelineInfo::shaders`.
        miss: u32,
    },
    Triangles {
        /// Index of any-hit shader in `RayTracingPipelineInfo::shaders`.
        any_hit: Option<u32>,
        /// Index of closest-hit shader in `RayTracingPipelineInfo::shaders`.
        closest_hit: Option<u32>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ShaderBindingTableInfo<'a> {
    pub raygen: Option<u32>,
    pub miss: &'a [u32],
    pub hit: &'a [u32],
    pub callable: &'a [u32],
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ShaderBindingTable {
    pub raygen: Option<StridedBufferRegion>,
    pub miss: Option<StridedBufferRegion>,
    pub hit: Option<StridedBufferRegion>,
    pub callable: Option<StridedBufferRegion>,
}
