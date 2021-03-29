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
pub struct Globals {
    #[uniform]
    #[stages(Vertex)]
    pub camera_view: sierra::mat4,

    #[uniform]
    #[stages(Vertex)]
    pub camera_proj: sierra::mat4,

    pub s: sierra::Sampler,
    #[combined_image_sampler(s)]
    #[stages(Fragment)]
    pub shadows: sierra::Image,
}

#[sierra::descriptors]
pub struct Object {
    pub s: sierra::Sampler,

    #[combined_image_sampler(s)]
    #[stages(Fragment)]
    pub albedo: sierra::Image,

    #[combined_image_sampler(s)]
    #[stages(Fragment)]
    pub metalness_normals: sierra::Image,

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

/// Pipeline definition
#[sierra::pipeline]
pub struct PBR {
    #[set]
    globals: Globals,

    #[set]
    object: Object,
}

pub fn example(
    device: &sierra::Device,
    queue: &mut sierra::Queue,
    fence: usize,
    globals: &Globals,
    objects: &mut Vec<(&Object, Option<ObjectInstance>)>,
) -> Result<(), sierra::OutOfMemory> {
    let mut encoder = queue.create_encoder()?;

    // Create pipeline layout
    let pbr = PBR::layout(device)?;

    // Create instances
    let mut globals_instance = pbr.globals.instance();

    // Then on each frame do the rest.
    let mut writes = Vec::new();
    globals_instance.update(globals, fence, device, &mut writes, &mut encoder)?;

    for (object, object_instance) in objects.iter_mut() {
        let object_instance = match object_instance {
            Some(object_instance) => object_instance,
            slot => slot.get_or_insert(pbr.object.instance()),
        };
        object_instance.update(object, fence, device, &mut writes, &mut encoder)?;
    }

    device.update_descriptor_sets(&writes, &[]);

    // Do draws here

    Ok(())
}
