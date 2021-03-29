//! This crate contains example for sierra's code-generation feature.\
//! It contains uniform structures and descriptor set layout.\
//! Generated types allows creating descriptor set layout
//! and descriptor sets with functions, no additional args aside from device is
//! required.\
//! Additionally it allows updating uniforms and descriptors binding in
//! straightforward manner in single function call `instance.update(&input)`,\
//! and then bind descriptor set to encoder.

/// Dummy structure
#[sierra::shader_repr]
pub struct InstanceInfo {
    pub transform: sierra::mat4,
    pub pos: sierra::vec3,
    pub fits_with_no_pad: f32,
}

/// Another dummy structure
#[sierra::shader_repr]
pub struct ComplexInfo {
    pub instance: InstanceInfo,
}

/// Descriptor set
#[sierra::descriptors]
pub struct PBRDescriptors {
    pub s: sierra::Sampler,

    #[combined_image_sampler(s)]
    #[stages(Fragment)]
    pub albedo: sierra::Image,

    #[combined_image_sampler(s)]
    #[stages(Fragment)]
    pub metalness_normals: sierra::Image,

    #[combined_image_sampler(s)]
    #[stages(Fragment)]
    pub shadows: sierra::Image,

    #[uniform]
    #[stages(Vertex)]
    pub camera_view: sierra::mat4,

    #[uniform]
    #[stages(Vertex)]
    pub camera_proj: sierra::mat4,

    #[uniform]
    #[stages(Vertex)]
    pub complex: ComplexInfo,

    #[uniform]
    #[stages(Fragment)]
    pub rgb: sierra::vec3,

    #[uniform]
    #[stages(Fragment)]
    pub x: f32,
}
