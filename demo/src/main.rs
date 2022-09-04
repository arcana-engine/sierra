#[derive(sierra::Descriptors)]
struct Descriptors {
    #[sierra(buffer, vertex)]
    views: sierra::Buffer,

    #[sierra(image(storage), vertex)]
    image: sierra::Image,

    #[sierra(sampler, fragment)]
    sampler: sierra::Sampler,

    #[sierra(image(sampled), fragment)]
    albedo: sierra::ImageView,

    #[sierra(uniform, stages(vertex, fragment))]
    foo: Foo,
}

#[derive(sierra::ShaderRepr)]
#[sierra(std140)]
struct Foo {
    foo: u32,
    bar: f32,
}

#[allow(dead_code)]
#[derive(sierra::PipelineInput)]
struct PipelineInput {
    #[sierra(set)]
    descriptors: Descriptors,

    #[sierra(push(std140), compute)]
    foo: Foo,
}

#[derive(sierra::Pass)]
#[sierra(subpass(color = target))]
#[sierra(dependency([external, color_attachment_output] => [0, color_attachment_output]))]
pub struct Main {
    #[sierra(attachment(clear = const sierra::ClearColor(0.3, 0.1, 0.8, 1.0), store = const sierra::Layout::Present))]
    target: sierra::Image,
}

fn main() -> eyre::Result<()> {
    let mut scope = scoped_arena::Scope::new();
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop)?;

    let graphics = sierra::Graphics::get_or_init()?;
    let mut surface = graphics.create_surface(&window, &window)?;

    let physical = graphics
        .devices()?
        .into_iter()
        .max_by_key(|d| d.info().kind)
        .ok_or_else(|| eyre::eyre!("Failed to find physical device"))?;

    let dynamic_rendering = physical
        .info()
        .features
        .contains(&sierra::Feature::DynamicRendering);

    let mut features = vec![sierra::Feature::SurfacePresentation];
    if dynamic_rendering {
        features.push(sierra::Feature::DynamicRendering);
    }

    let (device, mut queue) =
        physical.create_device(&features[..], sierra::SingleQueueQuery::GRAPHICS)?;

    let shader_module = device.create_shader_module(sierra::ShaderModuleInfo {
        code: br#"
@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
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
    let pipeline_layout = PipelineInput::layout(&device)?;

    let mut graphics_pipeline =
        sierra::DynamicGraphicsPipeline::new(sierra::graphics_pipeline_desc!(
            layout: pipeline_layout.raw().clone(),
            vertex_shader: sierra::VertexShader::new(shader_module.clone(), "vs_main"),
            fragment_shader: Some(sierra::FragmentShader::new(shader_module, "fs_main")),
        ));

    let mut fences = [None, None, None];
    let fences_len = fences.len();
    let mut fence_index = 0;
    let non_optimal_limit = 100u32;
    let mut non_optimal_count = 0;

    let mut view_cache = sierra::ImageViewCache::new();

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

            winit::event::Event::MainEventsCleared => {
                window.request_redraw();
            }

            winit::event::Event::RedrawRequested(_) => (|| -> eyre::Result<()> {
                if let Some(fence) = &mut fences[fence_index] {
                    device.wait_fences(&mut [fence], true);
                    device.reset_fences(&mut [fence]);
                }

                let mut image = swapchain.acquire_image()?;

                let mut encoder = queue.create_encoder(&scope)?;

                if dynamic_rendering {
                    encoder.image_barriers(
                        sierra::PipelineStages::COLOR_ATTACHMENT_OUTPUT,
                        sierra::PipelineStages::COLOR_ATTACHMENT_OUTPUT,
                        &[sierra::ImageMemoryBarrier::initialize_whole(
                            image.image(),
                            sierra::Access::COLOR_ATTACHMENT_WRITE,
                            sierra::Layout::ColorAttachmentOptimal,
                        )],
                    );

                    let mut render_pass_encoder = encoder.begin_rendering(
                        sierra::RenderingInfo::new().color(
                            &sierra::RenderingColorInfo::new(
                                view_cache.make_image(image.image(), &device)?.clone(),
                            )
                            .clear(sierra::ClearColor(0.3, 0.1, 0.8, 1.0)),
                        ),
                    );

                    render_pass_encoder
                        .bind_dynamic_graphics_pipeline(&mut graphics_pipeline, &device)?;

                    render_pass_encoder.push_constants(&pipeline_layout, &Foo { foo: 0, bar: 1.0 });

                    render_pass_encoder.draw(0..3, 0..1);
                    drop(render_pass_encoder);

                    encoder.image_barriers(
                        sierra::PipelineStages::COLOR_ATTACHMENT_OUTPUT,
                        sierra::PipelineStages::TOP_OF_PIPE,
                        &[sierra::ImageMemoryBarrier::transition_whole(
                            image.image(),
                            sierra::Access::COLOR_ATTACHMENT_WRITE..sierra::Access::empty(),
                            sierra::Layout::ColorAttachmentOptimal..sierra::Layout::Present,
                        )],
                    );
                } else {
                    let mut render_pass_encoder = encoder.with_render_pass(
                        &mut main,
                        &Main {
                            target: image.image().clone(),
                        },
                        &device,
                    )?;

                    render_pass_encoder
                        .bind_dynamic_graphics_pipeline(&mut graphics_pipeline, &device)?;

                    render_pass_encoder.push_constants(&pipeline_layout, &Foo { foo: 0, bar: 1.0 });

                    render_pass_encoder.draw(0..3, 0..1);
                    drop(render_pass_encoder);
                }

                let [wait, signal] = image.wait_signal();

                let fence = match &mut fences[fence_index] {
                    Some(fence) => fence,
                    None => fences[fence_index].get_or_insert(device.create_fence()?),
                };
                fence_index += 1;
                fence_index %= fences_len;

                queue.submit(
                    &mut [(sierra::PipelineStages::COLOR_ATTACHMENT_OUTPUT, wait)],
                    Some(encoder.finish()),
                    &mut [signal],
                    Some(fence),
                    &scope,
                );

                if !image.is_optimal() {
                    non_optimal_count += 1;
                }

                queue.present(image)?;

                if non_optimal_count >= non_optimal_limit {
                    swapchain.update()?;
                    non_optimal_count = 0;
                }

                scope.reset();
                Ok(())
            })()
            .unwrap(),
            _ => {}
        }
    })
}
