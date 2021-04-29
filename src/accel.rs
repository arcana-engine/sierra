pub use crate::backend::AccelerationStructure;
use crate::{buffer::BufferRange, format::Format, DeviceAddress, IndexType};

bitflags::bitflags! {
    /// Bits which can be set in `AccelerationStructureInfo` specifying additional parameters for acceleration structure builds.
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct AccelerationStructureBuildFlags: u32 {
        /// Allow acceleration structure update operation.
        /// Acceleration structure update allows changing internal structure
        /// without recreating whole `AccelerationStructure` instance.
        const ALLOW_UPDATE      = 0x00000001;

        /// Allow acceleration structure compaction operation.
        /// Compaction allows to reduce memory consumption.
        const ALLOW_COMPACTION  = 0x00000002;

        /// Hint implementation to make `AccelerationStructure` faster to trace.
        const PREFER_FAST_TRACE = 0x00000004;

        /// Hint implementation to make `AccelerationStructure` faster to build.
        const PREFER_FAST_BUILD = 0x00000008;

        /// Hint implementation to use minimal amount of memory.
        const LOW_MEMORY        = 0x00000010;
    }
}

/// Information required to create an instance of `AccelerationStructure`.
#[derive(Clone, Debug)]
pub struct AccelerationStructureInfo {
    /// Acceleration structure level.
    /// Either top level that reference to bottom level structures.
    /// Or bottom level that refer to geometry.
    pub level: AccelerationStructureLevel,

    /// Region of the buffer that will be used to store `AccelerationStructure`.
    ///
    /// Buffer must be created with `ACCELERATION_STRUCTURE_STORAGE` usage flag.
    /// required size can be queried using `Device::get_acceleration_structure_build_sizes`
    pub region: BufferRange,
}

/// Contains information about various size requirements for acceleration structure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct AccelerationStructureBuildSizesInfo {
    /// Size of buffer range required to bake acceleration structure.
    pub acceleration_structure_size: u64,

    /// Scratch size required for acceleration structure update operation.
    pub update_scratch_size: u64,

    /// Scratch size required for acceleration structure build operation.
    pub build_scratch_size: u64,
}

/// Acceleration structure level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum AccelerationStructureLevel {
    /// Bottom level acceleration structure (or BLAS) reference to
    /// triangle meshes and AABBs with custom intersection shaders.
    Bottom,

    /// Top level acceleration structure (or TLAS) reference to BLASes.
    Top,
}

/// Specifies the shape of geometries that will be built into an acceleration
/// structure.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum AccelerationStructureGeometryInfo {
    /// Defines that geometry type is triangle mesh and its data shape.
    Triangles {
        /// Maximum number of primitives that can be built into an acceleration
        /// structure geometry.
        max_primitive_count: u32,

        /// Type of indices if any.
        index_type: Option<IndexType>,

        /// Maximum number of vertex count.
        max_vertex_count: u32,

        /// Format of each vertex.
        vertex_format: Format,

        /// Whether mesh allows to be transformed.
        allows_transforms: bool,
    },
    /// Defines that geometry type is array of AABBs.
    AABBs {
        /// Maximum number of primitives that can be built into an acceleration
        /// structure geometry.
        max_primitive_count: u32,
    },
    /// Defines that geometry type is array of instances of BLASes.
    Instances {
        /// Maximum number of primitives that can be built into an acceleration
        /// structure geometry.
        max_primitive_count: u32,
    },
}

// impl AccelerationStructureGeometryInfo {
//     pub fn is_triangles(&self) -> bool {
//         match self {
//             Self::Triangles { .. } => true,
//             _ => false,
//         }
//     }

//     pub fn is_aabbs(&self) -> bool {
//         match self {
//             Self::AABBs { .. } => true,
//             _ => false,
//         }
//     }

//     pub fn is_instances(&self) -> bool {
//         match self {
//             Self::Instances { .. } => true,
//             _ => false,
//         }
//     }
// }

bitflags::bitflags! {
    /// Bits specifying additional parameters for geometries in acceleration structure builds
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct GeometryFlags: u32 {
        /// Consider all geometry in the acceleration structure to be opaque.
        /// Ray traverse operation won't invoke any-hit shader for opaque geometry
        /// to determine if hit occurs.
        const OPAQUE                            = 0x00000001;

        /// Enforce any-hit to not be invoked more than once for one ray for this geometry.
        const NO_DUPLICATE_ANY_HIT_INVOCATION   = 0x00000002;
    }
}

bitflags::bitflags! {
    /// Possible values of flags in the instance modifying the behavior of that instance.
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct GeometryInstanceFlags: u8 {
        /// Disables triangle face culling.
        const TRIANGLE_FACING_CULL_DISABLE    = 0x00000001;

        /// Indicates that triangle front is determined by counterclockwise direction.
        const TRIANGLE_FRONT_COUNTERCLOCKWISE = 0x00000002;

        /// Forces all geometry in instance to be considered opaque.
        const FORCE_OPAQUE                    = 0x00000004;

        /// Forces all geometry in instance to be considered non-opaque.
        const FORCE_NO_OPAQUE                 = 0x00000008;
    }
}

/// Data provided to acceleration structure build operation.
#[derive(Clone, Copy, Debug)]
pub struct AccelerationStructureBuildGeometryInfo<'a> {
    /// Source acceleration structure to copy from.
    pub src: Option<&'a AccelerationStructure>,

    /// Destination acceleration structure to build into.
    pub dst: &'a AccelerationStructure,

    /// Flags to specify options for the acceleration structure building.
    pub flags: AccelerationStructureBuildFlags,

    /// Array of geometries to build into acceleration structure.
    pub geometries: &'a [AccelerationStructureGeometry],

    /// Scratch memory that will be using during acceleration structure building process.
    /// See [`AccelerationStructureBuildSizesInfo`] to see how much memory is required.
    pub scratch: DeviceAddress,
}

/// Geometry data to build into acceleration structure.
#[derive(Clone, Debug)]
pub enum AccelerationStructureGeometry {
    /// Triangles data to build into acceleration structure.
    Triangles {
        flags: GeometryFlags,
        vertex_format: Format,
        vertex_data: BufferRange,
        vertex_stride: u64,
        vertex_count: u32,
        first_vertex: u32,
        primitive_count: u32,
        index_data: Option<IndexData>,
        transform_data: Option<BufferRange>,
    },
    AABBs {
        flags: GeometryFlags,
        data: BufferRange,
        stride: u64,
        primitive_count: u32,
    },
    Instances {
        flags: GeometryFlags,
        data: BufferRange,
        primitive_count: u32,
    },
}

#[derive(Clone, Debug)]
pub enum IndexData {
    U16(BufferRange),
    U32(BufferRange),
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct TransformMatrix {
    pub matrix: [[f32; 4]; 3],
}

impl TransformMatrix {
    pub fn identity() -> Self {
        TransformMatrix {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ],
        }
    }
}

impl Default for TransformMatrix {
    fn default() -> Self {
        Self::identity()
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[repr(align(8))]
#[repr(C)]
pub struct AabbPositions {
    pub min_x: f32,
    pub min_y: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub max_z: f32,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct InstanceCustomIndexAndMask(pub u32);

impl InstanceCustomIndexAndMask {
    pub fn new(custom_index: u32, mask: u8) -> Self {
        assert!(custom_index < 1u32 << 24);

        InstanceCustomIndexAndMask(custom_index | ((mask as u32) << 24))
    }
}

impl From<(u32, u8)> for InstanceCustomIndexAndMask {
    fn from((index, mask): (u32, u8)) -> Self {
        InstanceCustomIndexAndMask::new(index, mask)
    }
}

impl From<u32> for InstanceCustomIndexAndMask {
    fn from(index: u32) -> InstanceCustomIndexAndMask {
        InstanceCustomIndexAndMask::new(index, !0)
    }
}

impl Default for InstanceCustomIndexAndMask {
    fn default() -> Self {
        InstanceCustomIndexAndMask::new(0, !0)
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct InstanceShaderBindingOffsetAndFlags(pub u32);

impl InstanceShaderBindingOffsetAndFlags {
    pub fn new(instance_shader_binding_offset: u32, flags: GeometryInstanceFlags) -> Self {
        assert!(instance_shader_binding_offset < 1u32 << 24);

        InstanceShaderBindingOffsetAndFlags(
            instance_shader_binding_offset | ((flags.bits() as u32) << 24),
        )
    }
}

impl From<u32> for InstanceShaderBindingOffsetAndFlags {
    fn from(offset: u32) -> InstanceShaderBindingOffsetAndFlags {
        InstanceShaderBindingOffsetAndFlags::new(offset, GeometryInstanceFlags::empty())
    }
}

impl From<(u32, GeometryInstanceFlags)> for InstanceShaderBindingOffsetAndFlags {
    fn from((offset, flags): (u32, GeometryInstanceFlags)) -> Self {
        InstanceShaderBindingOffsetAndFlags::new(offset, flags)
    }
}

impl Default for InstanceShaderBindingOffsetAndFlags {
    fn default() -> Self {
        InstanceShaderBindingOffsetAndFlags::new(0, GeometryInstanceFlags::empty())
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(align(16))]
#[repr(C)]
pub struct AccelerationStructureInstance {
    pub transform: TransformMatrix,
    pub custom_index_mask: InstanceCustomIndexAndMask,
    pub shader_binding_offset_flags: InstanceShaderBindingOffsetAndFlags,
    pub acceleration_structure_reference: DeviceAddress,
}

unsafe impl bytemuck::Zeroable for AccelerationStructureInstance {}
unsafe impl bytemuck::Pod for AccelerationStructureInstance {}

impl AccelerationStructureInstance {
    pub fn new(blas_address: DeviceAddress) -> Self {
        AccelerationStructureInstance {
            transform: Default::default(),
            custom_index_mask: Default::default(),
            shader_binding_offset_flags: Default::default(),
            acceleration_structure_reference: blas_address,
        }
    }

    pub fn with_transform(mut self, transform: TransformMatrix) -> Self {
        self.transform = transform;

        self
    }

    pub fn set_transform(&mut self, transform: TransformMatrix) -> &mut Self {
        self.transform = transform;

        self
    }
}
