#[sierra::pipeline]
pub struct Pipeline;

#[sierra::pass]
#[subpass(color = target)]
pub struct Main {
    #[attachment(clear(bg), store(const sierra::Layout::Present))]
    target: sierra::Image,
    bg: sierra::ClearColor,
}

fn main() -> eyre::Result<()> {
    let bump = bumpalo::Bump::new();
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
    const x = f32(i32(in_vertex_index) - 1);
    const y = f32(i32(in_vertex_index & 1u) * 2 - 1);
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

    let mut graphisc_pipeline =
        sierra::DynamicGraphicsPipeline::new(sierra::graphics_pipeline_desc!(
            layout: pipeline_layout.raw().clone(),
            vertex_shader: sierra::VertexShader::new(shader_module.clone(), "vs_main"),
            fragment_shader: Some(sierra::FragmentShader::new(shader_module.clone(), "fs_main")),
        ));

    event_loop.run(move |event, target, flow| {
        *flow = winit::event_loop::ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => *flow = winit::event_loop::ControlFlow::Exit,

            winit::event::Event::RedrawRequested(_) => (|| -> eyre::Result<()> {
                let image = swapchain.acquire_image(false)?;

                let mut encoder = queue.create_encoder(&bump)?;
                let mut render_pass_encoder = encoder.with_render_pass(
                    &mut main,
                    &Main {
                        target: image.info().image.clone(),
                        bg: sierra::ClearColor(0.3, 0.1, 0.8, 1.0),
                    },
                    &device,
                )?;

                let extent = render_pass_encoder.framebuffer().info().extent;
                let gp = graphisc_pipeline.get(render_pass_encoder.render_pass(), 0, &device)?;
                render_pass_encoder.bind_graphics_pipeline(gp);
                render_pass_encoder.set_viewport(extent.into());
                render_pass_encoder.set_scissor(extent.into());
                render_pass_encoder.draw(0..3, 0..1);
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
            })()
            .unwrap(),
            _ => {}
        }
    })
}
