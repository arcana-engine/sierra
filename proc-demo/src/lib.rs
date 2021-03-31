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
pub struct Pipeline {
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
    render_pass: &::sierra::RenderPass,
    framebuffer: &::sierra::Framebuffer,
    clears: &[::sierra::ClearValue],
    mut graphics_pipeline: ::sierra::GraphicsPipelineInfo,
    bump: &bumpalo::Bump,
) -> Result<(), sierra::OutOfMemory> {
    // Create pipeline layout
    let pbr = Pipeline::layout(device)?;

    // Finish creating graphics pipeline
    graphics_pipeline.layout = pbr.raw().clone();
    graphics_pipeline.render_pass = render_pass.clone();
    let graphics_pipeline = device.create_graphics_pipeline(graphics_pipeline)?;

    // Create globals instance
    let mut globals_instance = pbr.globals.instance();

    // The following should be repeated each frame.

    // Make vector to store descriptor writes.
    let mut writes = bumpalo::collections::Vec::new_in(bump);

    // Create encoder to encode commands before render pass.
    let mut encoder = queue.create_encoder()?;

    // Update globals.
    // This may extend descriptors writes and record some commands.
    let globals = globals_instance.update(globals, fence, device, &mut writes, &mut encoder)?;

    // Begin render pass encoding in parallel.
    let mut render_pass_encoder = queue.create_encoder()?;
    let mut render_pass = render_pass_encoder.with_render_pass(render_pass, framebuffer, clears);

    // Don't forget to bind graphics pipeline.
    render_pass.bind_graphics_pipeline(&graphics_pipeline);

    // Bind globals to graphics pipeline
    pbr.bind_graphics(globals, &mut render_pass);

    for (object, instance) in objects.iter_mut() {
        // Ensure object descriptors instance is attached to each object
        let instance = match instance {
            Some(instance) => instance,
            slot => slot.get_or_insert(pbr.object.instance()),
        };

        // Update object descriptors
        let object = instance.update(object, fence, device, &mut writes, &mut encoder)?;

        // Bind object descriptors to graphics pipeline.
        pbr.bind_graphics(object, &mut render_pass);

        // Currently vertices and instances binding is not covered by sierra's code-gen.
        // Here's dummy values.
        let vertices = 0..0;
        let instances = 0..0;

        render_pass.draw(vertices, instances);
    }

    // Render pass recording ends here.
    drop(render_pass);

    // Ensure to flush descriptors writes before submitting commands.
    device.update_descriptor_sets(&writes, &[]);

    // Submit commands.
    queue.submit(
        &[],
        std::array::IntoIter::new([encoder.finish(), render_pass_encoder.finish()]),
        &[],
        None,
    );

    Ok(())
}
