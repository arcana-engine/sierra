#[sierra::descriptors]
pub struct Descriptors {
    #[sampled_image]
    pub foo: sierra::ImageView,

    #[sampler]
    pub bar: sierra::Sampler,
}

#[sierra::pipeline]
pub struct Pipeline {
    descriptors: Descriptors,
    sparse: sierra::SparseDescriptorSet<sierra::SampledImageDescriptor, 32>,
}

#[sierra::pass]
#[subpass(color = target)]
pub struct Main {
    #[attachment(clear(const sierra::ClearColor(0.3, 0.1, 0.8, 1.0)), store(const sierra::Layout::Present))]
    target: sierra::Image,
}

fn main() -> eyre::Result<()> {
    let mut scope = scoped_arena::Scope::new();
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
        code: br#"
[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] in_vertex_index: u32) -> [[builtin(position)]] vec4<f32> {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
        "#
        .to_vec()
        .into_boxed_slice(),
        language: sierra::ShaderLanguage::WGSL,
    })?;

    let mut swapchain = device.create_swapchain(&mut surface)?;
    swapchain.configure(
        sierra::ImageUsage::COLOR_ATTACHMENT,
        sierra::Format::BGRA8Srgb,
        sierra::PresentMode::Fifo,
    )?;

    let mut main = Main::instance();
    let pipeline_layout = Pipeline::layout(&device)?;

    let mut graphics_pipeline =
        sierra::DynamicGraphicsPipeline::new(sierra::graphics_pipeline_desc!(
            layout: pipeline_layout.raw().clone(),
            vertex_shader: sierra::VertexShader::new(shader_module.clone(), "vs_main"),
            fragment_shader: Some(sierra::FragmentShader::new(shader_module.clone(), "fs_main")),
        ));

    event_loop.run(move |event, _target, flow| {
        *flow = winit::event_loop::ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => {
                device.wait_idle();
                *flow = winit::event_loop::ControlFlow::Exit;
            }

            winit::event::Event::RedrawRequested(_) => (|| -> eyre::Result<()> {
                let mut image = swapchain.acquire_image()?;

                let mut encoder = queue.create_encoder(&scope)?;
                let mut render_pass_encoder = encoder.with_render_pass(
                    &mut main,
                    &Main {
                        target: image.image().clone(),
                    },
                    &device,
                )?;

                render_pass_encoder
                    .bind_dynamic_graphics_pipeline(&mut graphics_pipeline, &device)?;
                render_pass_encoder.draw(0..3, 0..1);
                drop(render_pass_encoder);

                let [wait, signal] = image.wait_signal();

                queue.submit(
                    &mut [(sierra::PipelineStageFlags::TOP_OF_PIPE, wait)],
                    Some(encoder.finish()),
                    &mut [signal],
                    None,
                    &scope,
                );

                let optimal = image.is_optimal();

                queue.present(image)?;

                if !optimal {
                    swapchain.update()?;
                }

                scope.reset();
                Ok(())
            })()
            .unwrap(),
            _ => {}
        }
    })
}
