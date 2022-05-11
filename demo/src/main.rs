#[derive(sierra::TypedDescriptors)]
struct DescriptorSet {
    #[sierra(buffer(uniform, texel), vertex)]
    views: sierra::BufferView,

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

#[derive(sierra::TypedPipeline)]
struct Pipeline {
    #[sierra(set)]
    descriptors: DescriptorSet,

    #[sierra(push(std140), compute)]
    foo: Foo,
}

#[derive(sierra::TypedRenderPass)]
#[sierra(subpass(color = target))]
pub struct Main {
    #[sierra(attachment(clear = const sierra::ClearColor(0.3, 0.1, 0.8, 1.0), store = const sierra::Layout::Present))]
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
            fragment_shader: Some(sierra::FragmentShader::new(shader_module, "fs_main")),
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

                render_pass_encoder.push_constants(&pipeline_layout, &Foo { foo: 0, bar: 1.0 });

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

// #[allow(dead_code)]
// pub struct Main {
//     target: sierra::Image,
// }
// impl Main {
//     pub fn instance() -> MainInstance {
//         MainInstance::new()
//     }
// }
// pub struct MainInstance {
//     render_pass: Option<::sierra::RenderPass>,
//     framebuffers: ::std::vec::Vec<::sierra::Framebuffer>,
// }
// impl MainInstance {
//     pub fn new() -> Self {
//         MainInstance {
//             render_pass: None,
//             framebuffers: ::std::vec::Vec::new(),
//         }
//     }
//     pub fn update_framebuffer(
//         &mut self,
//         input: &Main,
//         device: &::sierra::Device,
//     ) -> ::std::result::Result<&::sierra::Framebuffer, ::sierra::FramebufferError> {
//         let mut render_pass_compatible;
//         if let Some(render_pass) = &self.render_pass {
//             render_pass_compatible = true;
//             let a = render_pass.info().attachments[0usize];
//             if a.format != ::sierra::Attachment::format(&input.target) {
//                 render_pass_compatible = false;
//             } else if let Some(samples) = ::sierra::Attachment::samples(&input.target) {
//                 if a.samples != samples {
//                     render_pass_compatible = false;
//                 }
//             } else if a.initial_layout
//                 != ::std::option::Option::map(::std::option::Option::None, ::sierra::Layout::from)
//             {
//                 render_pass_compatible = false;
//             } else if a.final_layout != ::sierra::Layout::from(sierra::Layout::Present) {
//                 render_pass_compatible = false;
//             }
//         } else {
//             render_pass_compatible = false;
//         }
//         if !render_pass_compatible {
//             if self.render_pass.is_some() {
//             } else {
//             }
//             self.render_pass = None;
//             let mut attachments = ::std::vec::Vec::with_capacity(1usize);
//             attachments.push(::sierra::AttachmentInfo {
//                 format: ::sierra::Attachment::format(&input.target),
//                 samples: ::sierra::Attachment::samples(&input.target).unwrap_or_default(),
//                 load_op: ::sierra::LoadOp::Clear,
//                 store_op: ::sierra::StoreOp::Store,
//                 initial_layout: ::std::option::Option::None,
//                 final_layout: sierra::Layout::Present,
//             });
//             let mut subpasses = ::std::vec::Vec::with_capacity(1usize);
//             subpasses.push(::sierra::Subpass {
//                 colors: {
//                     let mut colors = ::std::vec::Vec::with_capacity(1usize);
//                     colors.push((0u32, ::sierra::Layout::ColorAttachmentOptimal));
//                     colors
//                 },
//                 depth: None,
//             });
//             let mut dependencies = ::std::vec::Vec::with_capacity(0usize);
//             let render_pass = self
//                 .render_pass
//                 .get_or_insert(::sierra::Device::create_render_pass(
//                     device,
//                     ::sierra::RenderPassInfo {
//                         attachments,
//                         subpasses,
//                         dependencies,
//                     },
//                 )?);
//             let framebuffer_info = match self.framebuffers.iter().find(|fb| {
//                 let fbinfo = fb.info();
//                 if true {
//                     {
//                         match (&fbinfo.attachments.len(), &1usize) {
//                             (left_val, right_val) => {
//                                 if !(*left_val == *right_val) {
//                                     panic!();
//                                 }
//                             }
//                         }
//                     };
//                 };
//                 if !::sierra::Attachment::eq(&input.target, &fbinfo.attachments[0usize]) {
//                     return false;
//                 }
//                 true
//             }) {
//                 Some(fb) => ::sierra::FramebufferInfo {
//                     render_pass: ::std::clone::Clone::clone(render_pass),
//                     attachments: ::std::clone::Clone::clone(&fb.info().attachments),
//                     extent: fb.info().extent,
//                 },
//                 None => {
//                     let mut fb_extent = ::sierra::Extent2d {
//                         width: !0,
//                         height: !0,
//                     };
//                     fb_extent = ::sierra::Extent2d::min(
//                         &fb_extent,
//                         &::sierra::Attachment::max_extent(&input.target),
//                     );
//                     let mut attachments = ::std::vec::Vec::with_capacity(1usize);
//                     attachments.push(::sierra::Attachment::get_view(
//                         &input.target,
//                         device,
//                         ::sierra::ImageUsage::empty() | ::sierra::ImageUsage::COLOR_ATTACHMENT,
//                         fb_extent,
//                     )?);
//                     ::sierra::FramebufferInfo {
//                         render_pass: ::std::clone::Clone::clone(render_pass),
//                         attachments,
//                         extent: fb_extent,
//                     }
//                 }
//             };
//             self.framebuffers.clear();
//             self.framebuffers.push(::sierra::Device::create_framebuffer(
//                 device,
//                 framebuffer_info,
//             )?);
//         } else {
//             let render_pass = self.render_pass.as_ref().unwrap();
//             let framebuffer = match self.framebuffers.iter().position(|fb| {
//                 let fbinfo = fb.info();
//                 if true {
//                     {
//                         match (&fbinfo.attachments.len(), &1usize) {
//                             (left_val, right_val) => {
//                                 if !(*left_val == *right_val) {
//                                     panic!()
//                                 }
//                             }
//                         }
//                     };
//                 };
//                 if !::sierra::Attachment::eq(&input.target, &fbinfo.attachments[0usize]) {
//                     return false;
//                 }
//                 true
//             }) {
//                 Some(fb_index) => {
//                     let fb = self.framebuffers.remove(fb_index);
//                     self.framebuffers.push(fb);
//                 }
//                 None => {
//                     let mut fb_extent = ::sierra::Extent2d {
//                         width: !0,
//                         height: !0,
//                     };
//                     fb_extent = ::sierra::Extent2d::min(
//                         &fb_extent,
//                         &::sierra::Attachment::max_extent(&input.target),
//                     );
//                     let mut attachments = ::std::vec::Vec::with_capacity(1usize);
//                     attachments.push(::sierra::Attachment::get_view(
//                         &input.target,
//                         device,
//                         ::sierra::ImageUsage::empty() | ::sierra::ImageUsage::COLOR_ATTACHMENT,
//                         fb_extent,
//                     )?);
//                     let framebuffer = ::sierra::Device::create_framebuffer(
//                         device,
//                         ::sierra::FramebufferInfo {
//                             render_pass: ::std::clone::Clone::clone(render_pass),
//                             attachments,
//                             extent: fb_extent,
//                         },
//                     )?;
//                     self.framebuffers.push(framebuffer);
//                     while self.framebuffers.len() > 3 {
//                         self.framebuffers.remove(0);
//                     }
//                 }
//             };
//         }
//         Ok(self.framebuffers.last().unwrap())
//     }
// }
// impl ::sierra::RenderPassInstance for MainInstance {
//     type Input = Main;
//     fn begin_render_pass<'a, 'b>(
//         &'a mut self,
//         input: &Main,
//         device: &::sierra::Device,
//         encoder: &'b mut ::sierra::Encoder<'a>,
//     ) -> ::std::result::Result<::sierra::RenderPassEncoder<'b, 'a>, ::sierra::FramebufferError>
//     {
//         let fb = self.update_framebuffer(input, device)?;
//         Ok(encoder.with_framebuffer(
//             fb,
//             encoder
//                 .scope()
//                 .to_scope([::sierra::ClearValue::from(sierra::ClearColor(
//                     0.3, 0.1, 0.8, 1.0,
//                 ))]),
//         ))
//     }
// }
