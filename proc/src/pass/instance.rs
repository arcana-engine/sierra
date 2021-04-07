use {
    super::parse::{ClearValue, Input, Layout, LoadOp, StoreOp},
    proc_macro2::TokenStream,
    std::convert::TryFrom,
};

pub(super) fn generate(input: &Input) -> TokenStream {
    let vis = &input.item_struct.vis;
    let ident = &input.item_struct.ident;
    let instance = quote::format_ident!("{}Instance", input.item_struct.ident);

    let attachment_checks = input
        .attachments
        .iter()
        .enumerate()
        .map(|(index, a)| {
            let member = &a.member;

            let initial_layout = initial_layout(&a.load_op);
            let check_final_layout = final_layout(&a.store_op).map(|final_layout| {
                quote::quote!(else if a.final_layout != ::sierra::Layout::from(#final_layout) {
                    tracing::debug!("Final layout is incompatible. Old {:?}, new {:?}", a.final_layout, #final_layout);
                    render_pass_compatible = false;
                } )
            });

            quote::quote!(
                let a = render_pass.info().attachments[#index];
                if a.format != ::sierra::Attachment::format(&input.#member) {
                    tracing::debug!("Format is incompatible. Old {:?}, new {:?}", a.format, ::sierra::Attachment::format(&input.#member));
                    render_pass_compatible = false;
                } else if let Some(samples) = ::sierra::Attachment::samples(&input.#member) {
                    if a.samples != samples {
                        tracing::debug!("Samples count is incompatible. Old {:?}, new {:?}", a.samples, ::sierra::Attachment::samples(&input.#member));
                        render_pass_compatible = false;
                    }
                } else if a.initial_layout != ::std::option::Option::map(#initial_layout, ::sierra::Layout::from) {
                    tracing::debug!("Initial layout is incompatible. Old {:?}, new {:?}", a.initial_layout, ::std::option::Option::map(#initial_layout, ::sierra::Layout::from));
                    render_pass_compatible = false;
                } #check_final_layout
            )
        })
        .collect::<TokenStream>();

    let push_attachment_infos = input
        .attachments
        .iter()
        .enumerate()
        .map(|(index, a)| {
            let index = index as u32;

            let member = &a.member;
            let load_op = match a.load_op {
                LoadOp::Clear(_) => quote::format_ident!("Clear"),
                LoadOp::Load(_) => quote::format_ident!("Load"),
                LoadOp::DontCare => quote::format_ident!("DontCare"),
            };

            let initial_layout = initial_layout(&a.load_op);
            let final_layout = match final_layout(&a.store_op) {
                None => {
                    // find last use. General if unused.
                    input
                        .subpasses
                        .iter()
                        .filter_map(|s| {
                            if s.colors.iter().any(|&c| c == index) {
                                Some(quote::quote!(::sierra::Layout::ColorAttachmentOptimal))
                            } else if s.depth == Some(index) {
                                Some(quote::quote!(
                                    ::sierra::Layout::DepthStencilAttachmentOptimal
                                ))
                            } else {
                                None
                            }
                        })
                        .rev()
                        .next()
                        .unwrap_or_else(|| quote::quote!(::sierra::Layout::General))
                }
                Some(layout) => layout,
            };

            let store_op = match a.store_op {
                StoreOp::Store(_) => quote::format_ident!("Store"),
                StoreOp::DontCare => quote::format_ident!("DontCare"),
            };

            quote::quote!(
                attachments.push(::sierra::AttachmentInfo {
                    format: ::sierra::Attachment::format(&input.#member),
                    samples: ::sierra::Attachment::samples(&input.#member).unwrap_or_default(),
                    load_op: ::sierra::LoadOp::#load_op,
                    store_op: ::sierra::StoreOp::#store_op,
                    initial_layout: #initial_layout,
                    final_layout: #final_layout,
                });
            )
        })
        .collect::<TokenStream>();

    let push_subpass_infos = input
        .subpasses
        .iter()
        .map(|s| {
            let push_colors = s
                .colors
                .iter()
                .map(
                    |&c| quote::quote!(colors.push((#c, ::sierra::Layout::ColorAttachmentOptimal));),
                )
                .collect::<TokenStream>();

            let color_count = s.colors.len();

            match s.depth {
                Some(depth) => {
                    quote::quote!(
                        subpasses.push(::sierra::Subpass {
                            colors: {
                                let mut colors = ::std::vec::Vec::with_capacity(#color_count);
                                #push_colors
                                colors
                            },
                            depth: Some((#depth, ::sierra::Layout::DepthStencilAttachmentOptimal)),
                        });
                    )
                }
                None => {
                    quote::quote!(
                        subpasses.push(::sierra::Subpass {
                            colors: {
                                let mut colors = ::std::vec::Vec::with_capacity(#color_count);
                                #push_colors
                                colors
                            },
                            depth: None,
                        });
                    )
                }
            }
        })
        .collect::<TokenStream>();

    let attachment_count = input.attachments.len();
    let check_framebuffer_len = quote::quote!(
        debug_assert_eq!(fbinfo.attachments.len(), #attachment_count);
    );

    let check_framebuffer_attachments = input
        .attachments
        .iter()
        .enumerate()
        .map(|(index, a)| {
            let member = &a.member;
            quote::quote!(if !::sierra::Attachment::eq(&input.#member, &fbinfo.attachments[#index]) { return false; })
        })
        .collect::<TokenStream>();

    let find_fb_extent = input
        .attachments
        .iter()
        .map(|a| {
            let member = &a.member;
            quote::quote!(fb_extent = ::sierra::Extent2d::min(&fb_extent, &::sierra::Attachment::max_extent(&input.#member));)
        })
        .collect::<TokenStream>();

    let push_framebuffer_attachments = input
        .attachments
        .iter()
        .enumerate()
        .map(|(index, a)| {
            let index = u32::try_from(index).unwrap();

            let member = &a.member;
            let usages = input.subpasses.iter().filter_map(|s| {
                if s.colors.iter().any(|&c| c == index) {
                    Some(quote::quote!(::sierra::ImageUsage::COLOR_ATTACHMENT))
                } else if s.depth == Some(index) {
                    Some(quote::quote!(
                        ::sierra::ImageUsage::DEPTH_STENCIL_ATTACHMENT
                    ))
                } else {
                    None
                }
            });

            quote::quote!(
                attachments.push(::sierra::Attachment::get_view(&input.#member, device, ::sierra::ImageUsage::empty() #(|#usages)*, fb_extent)?);
            )
        })
        .collect::<TokenStream>();

    let subpass_count = input.subpasses.len();
    // let dependency_count = input.dependencies.len();
    let dependency_count = 0usize;

    let clear_values = input
        .attachments
        .iter()
        .filter_map(|a| match &a.load_op {
            LoadOp::Clear(ClearValue::Member(member)) => {
                Some(quote::quote!(::sierra::ClearValue::from(input.#member),))
            }
            LoadOp::Clear(ClearValue::Expr(expr)) => {
                Some(quote::quote!(::sierra::ClearValue::from(#expr),))
            }
            _ => None,
        })
        .collect::<TokenStream>();

    quote::quote!(
        #vis struct #instance {
            render_pass: Option<::sierra::RenderPass>,
            framebuffers: ::std::vec::Vec<::sierra::Framebuffer>,
        }

        impl #instance {
            pub fn new() -> Self {
                #instance {
                    render_pass: None,
                    framebuffers: ::std::vec::Vec::new(),
                }
            }

            pub fn update_framebuffer(&mut self, input: &#ident, device: &::sierra::Device)  -> ::std::result::Result<&::sierra::Framebuffer, ::sierra::FramebufferError> {
                let mut render_pass_compatible;
                if let Some(render_pass) = &self.render_pass {
                    render_pass_compatible = true;
                    #attachment_checks
                } else {
                    render_pass_compatible = false;
                }

                if !render_pass_compatible {
                    tracing::debug!("Render pass is not compatible with cached instance");

                    self.render_pass = None;

                    let mut attachments = ::std::vec::Vec::with_capacity(#attachment_count);
                    #push_attachment_infos

                    let mut subpasses = ::std::vec::Vec::with_capacity(#subpass_count);
                    #push_subpass_infos

                    let mut dependencies = ::std::vec::Vec::with_capacity(#dependency_count);

                    let render_pass = self.render_pass.get_or_insert(::sierra::Device::create_render_pass(
                        device,
                        ::sierra::RenderPassInfo {
                            attachments,
                            subpasses,
                            dependencies
                        }
                    )?);

                    let framebuffer_info = match self.framebuffers.iter().find(|fb| {
                        let fbinfo = fb.info();
                        #check_framebuffer_len
                        #check_framebuffer_attachments
                        true
                    }) {
                        Some(fb) => {
                            tracing::trace!("Found framebuffer with compatible attachments");

                            ::sierra::FramebufferInfo {
                                render_pass: ::std::clone::Clone::clone(render_pass),
                                attachments: ::std::clone::Clone::clone(&fb.info().attachments),
                                extent: fb.info().extent,
                            }
                        }
                        None => {
                            tracing::debug!("Framebuffer with compatible attachments not found");

                            let mut fb_extent = ::sierra::Extent2d { width: !0, height: !0 };
                            #find_fb_extent

                            let mut attachments = ::std::vec::Vec::with_capacity(#attachment_count);
                            #push_framebuffer_attachments

                            ::sierra::FramebufferInfo {
                                render_pass: ::std::clone::Clone::clone(render_pass),
                                attachments,
                                extent: fb_extent,
                            }
                        }
                    };

                    self.framebuffers.clear();
                    self.framebuffers.push(::sierra::Device::create_framebuffer(
                        device,
                        framebuffer_info,
                    )?);
                } else {
                    let render_pass = self.render_pass.as_ref().unwrap();
                    let framebuffer = match self.framebuffers.iter().position(|fb| {
                        let fbinfo = fb.info();
                        #check_framebuffer_len
                        #check_framebuffer_attachments
                        true
                    }) {
                        Some(fb_index) => {
                            tracing::trace!("Found framebuffer with compatible attachments");

                            let fb = self.framebuffers.remove(fb_index);
                            self.framebuffers.push(fb);
                        },
                        None => {
                            tracing::debug!("Framebuffer with compatible attachments not found");

                            let mut fb_extent = ::sierra::Extent2d { width: !0, height: !0 };
                            #find_fb_extent

                            let mut attachments = ::std::vec::Vec::with_capacity(#attachment_count);
                            #push_framebuffer_attachments

                            let framebuffer = ::sierra::Device::create_framebuffer(
                                device,
                                ::sierra::FramebufferInfo {
                                    render_pass: ::std::clone::Clone::clone(render_pass),
                                    attachments,
                                    extent: fb_extent,
                                },
                            )?;

                            self.framebuffers.push(framebuffer);

                            while self.framebuffers.len() > 3 {
                                self.framebuffers.remove(0);
                            }
                        }
                    };
                }

                Ok(self.framebuffers.last().unwrap())
            }
        }

        impl ::sierra::RenderPassInstance for #instance {
            type Input = #ident;

            fn begin_render_pass<'a, 'b>(&'a mut self, input: &#ident, device: &::sierra::Device, encoder: &'b mut ::sierra::Encoder<'a>) -> ::std::result::Result<::sierra::RenderPassEncoder<'b, 'a>, ::sierra::FramebufferError> {
                let fb = self.update_framebuffer(input, device)?;
                Ok(encoder.with_framebuffer(fb, &[#clear_values]))
            }
        }
    )
}

fn initial_layout(load_op: &LoadOp) -> TokenStream {
    match load_op {
        LoadOp::Clear(_) => quote::quote!(::std::option::Option::None),
        LoadOp::DontCare => quote::quote!(::std::option::Option::None),
        LoadOp::Load(Layout::Expr(expr)) => {
            quote::quote!(::std::option::Option::Some(#expr))
        }
        LoadOp::Load(Layout::Member(layout)) => {
            quote::quote!(::std::option::Option::Some(self.#layout))
        }
    }
}

fn final_layout(store_op: &StoreOp) -> Option<TokenStream> {
    match store_op {
        StoreOp::DontCare => None,
        StoreOp::Store(Layout::Expr(expr)) => Some(quote::quote!(#expr)),
        StoreOp::Store(Layout::Member(layout)) => Some(quote::quote!(self.#layout)),
    }
}
