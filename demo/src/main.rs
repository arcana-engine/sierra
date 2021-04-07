//! This crate contains example for sierra's code-generation feature.\
//! It contains uniform structures and descriptor set layout.\
//! Generated types allows creating descriptor set layout
//! and descriptor sets with functions, no additional args aside from device is
//! required.\
//! Additionally it allows updating uniforms and descriptors binding in
//! straightforward manner in single function call `instance.update(&input)`,\
//! and then bind descriptor set to encoder.

use {
    bumpalo::{collections::Vec as BVec, Bump},
    sierra::RenderPassInstance as _,
    tracing_subscriber::layer::SubscriberExt as _,
};

/// Descriptor set
#[sierra::descriptors]
pub struct Globals {
    #[uniform]
    #[stages(Vertex)]
    pub camera_view: sierra::mat4,

    #[uniform]
    #[stages(Vertex)]
    pub camera_proj: sierra::mat4,
}

#[sierra::descriptors]
pub struct Object {
    #[sampler]
    pub s: sierra::Sampler,

    #[sampled_image]
    #[stages(Fragment)]
    pub albedo: sierra::Image,

    #[uniform]
    #[stages(Vertex)]
    pub transform: sierra::mat4,

    #[uniform]
    #[stages(Fragment)]
    pub rgb: sierra::vec3,
}

/// Pipeline definition
#[sierra::pipeline]
pub struct Pipeline {
    #[set]
    globals: Globals,

    #[set]
    object: Object,
}

#[sierra::pass]
#[subpass(color = target, depth = depth)]
pub struct Main {
    #[attachment(clear(bg), store(const sierra::Layout::Present))]
    target: sierra::Image,

    bg: sierra::ClearColor,

    #[attachment(clear(const sierra::ClearDepth(1.0)))]
    depth: sierra::Format,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .pretty()
            .finish()
            .with(tracing_error::ErrorLayer::default()),
    )?;

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop)?;

    let graphics = sierra::Graphics::get_or_init()?;
    let mut surface = graphics.create_surface(&window)?;

    let physical = graphics
        .devices()?
        .into_iter()
        .max_by_key(|d| d.info().kind)
        .ok_or_else(|| eyre::eyre!("Failed to find physical device"))?;

    let (device, mut queue) = physical.create_device(
        &[sierra::Feature::SurfacePresentation],
        sierra::SingleQueueQuery::GRAPHICS,
    )?;

    let shader_module = device.create_shader_module(sierra::ShaderModuleInfo {
        code: std::include_bytes!("main.wgsl").to_vec().into_boxed_slice(),
        language: sierra::ShaderLanguage::WGSL,
    })?;

    let mut swapchain = device.create_swapchain(&mut surface)?;

    let mut main = Main::instance();
    let pipeline_layout = Pipeline::layout(&device)?;
    let mut globals = pipeline_layout.globals.instance();

    // let mut graphisc_pipeline = None;

    let bump = bumpalo::Bump::new();

    event_loop.run(move |event, target, flow| {
        *flow = winit::event_loop::ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => *flow = winit::event_loop::ControlFlow::Exit,

            winit::event::Event::RedrawRequested(_) => {
                let result = (|| -> eyre::Result<()> {
                    let image = loop {
                        // Note. Hide this loop inside `acquire_image`?
                        if let Some(image) = swapchain.acquire_image()? {
                            break image;
                        }

                        swapchain.configure(
                            sierra::ImageUsage::COLOR_ATTACHMENT,
                            sierra::Format::BGRA8Srgb,
                            sierra::PresentMode::Fifo,
                        )?;
                    };

                    let mut encoder = queue.create_encoder(&bump)?;
                    let mut render_pass_encoder = main.begin_render_pass(
                        &Main {
                            target: image.info().image.clone(),
                            bg: sierra::ClearColor(0.3, 0.1, 0.8, 1.0),
                            depth: sierra::Format::D32Sfloat,
                        },
                        &device,
                        &mut encoder,
                    )?;
                    drop(render_pass_encoder);
                    queue.submit(
                        &[(
                            sierra::PipelineStageFlags::TOP_OF_PIPE,
                            image.info().wait.clone(),
                        )],
                        Some(encoder.finish()),
                        &[image.info().signal.clone()],
                        None,
                        &bump,
                    );

                    queue.present(image)?;

                    Ok(())
                })();

                if let Err(err) = result {
                    Err(err).unwrap()
                }
            }
            _ => {}
        }
    })
}

pub fn example(
    device: &sierra::Device,
    queue: &mut sierra::Queue,
    main: &Main,
    fence: usize,
    globals: &Globals,
    objects: &mut Vec<(&Object, Option<ObjectInstance>)>,
    mut graphics_pipeline_info: ::sierra::GraphicsPipelineInfo,
    bump: &bumpalo::Bump,
) -> Result<(), sierra::FramebufferError> {
    let graphics_pipeline;

    // Create pipeline layout
    let mut main_instance = Main::instance();

    // Create globals instance
    let pipeline_layout = Pipeline::layout(device)?;
    let mut globals_instance = pipeline_layout.globals.instance();

    // The following should be repeated each frame.

    // Make vector to store descriptor writes.
    let mut writes = bumpalo::collections::Vec::new_in(bump);

    // Create encoder to encode commands before render pass.
    let mut encoder = queue.create_encoder(bump)?;

    // Update globals.
    // This may extend descriptors writes and record some commands.
    let globals = globals_instance.update(globals, fence, device, &mut writes, &mut encoder)?;

    // Begin render pass encoding in parallel.
    let mut render_pass_encoder = queue.create_encoder(bump)?;
    let mut render_pass = render_pass_encoder.with_render_pass(&mut main_instance, main, device)?;

    // Finish creating graphics pipeline
    graphics_pipeline_info.layout = pipeline_layout.raw().clone();
    graphics_pipeline_info.render_pass = render_pass.render_pass().clone();
    graphics_pipeline = device.create_graphics_pipeline(graphics_pipeline_info)?;

    // Don't forget to bind graphics pipeline.
    render_pass.bind_graphics_pipeline(&graphics_pipeline);

    // Bind globals to graphics pipeline
    pipeline_layout.bind_graphics(globals, &mut render_pass);

    for (object, instance) in objects.iter_mut() {
        // Ensure object descriptors instance is attached to each object
        let instance = match instance {
            Some(instance) => instance,
            slot => slot.get_or_insert(pipeline_layout.object.instance()),
        };

        // Update object descriptors
        let object = instance.update(object, fence, device, &mut writes, &mut encoder)?;

        // Bind object descriptors to graphics pipeline.
        pipeline_layout.bind_graphics(object, &mut render_pass);

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
        bump,
    );

    Ok(())
}
