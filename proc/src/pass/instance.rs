use std::convert::TryFrom;

use proc_easy::ReferenceExpr;
use proc_macro2::TokenStream;

use super::parse::{Clear, Input, Load, LoadOp, Store, StoreOp, SubpassDependency};

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
            let check_final_layout = a.store_op.as_ref().map(|store_op| {
                let final_layout = final_layout(store_op);
                quote::quote!(else if a.final_layout != ::sierra::Layout::from(#final_layout) {
                    ::sierra::debug!("Final layout is incompatible. Old {:?}, new {:?}", a.final_layout, #final_layout);
                    render_pass_compatible = false;
                } )
            });

            quote::quote!(
                let a = render_pass.info().attachments[#index];

                #[allow(clippy::cmp_owned)]
                {
                    if a.format != ::sierra::Attachment::format(&input.#member) {
                        ::sierra::debug!("Format is incompatible. Old {:?}, new {:?}", a.format, ::sierra::Attachment::format(&input.#member));
                        render_pass_compatible = false;
                    } else if let Some(samples) = ::sierra::Attachment::samples(&input.#member) {
                        if a.samples != samples {
                            ::sierra::debug!("Samples count is incompatible. Old {:?}, new {:?}", a.samples, ::sierra::Attachment::samples(&input.#member));
                            render_pass_compatible = false;
                        }
                    } else if a.initial_layout != ::std::option::Option::map(#initial_layout, ::sierra::Layout::from) {
                        ::sierra::debug!("Initial layout is incompatible. Old {:?}, new {:?}", a.initial_layout, ::std::option::Option::map(#initial_layout, ::sierra::Layout::from));
                        render_pass_compatible = false;
                    } #check_final_layout
                }
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
                Some(LoadOp::Clear(_)) => quote::format_ident!("Clear"),
                Some(LoadOp::Load(_)) => quote::format_ident!("Load"),
                None => quote::format_ident!("DontCare"),
            };

            let initial_layout = initial_layout(&a.load_op);
            let final_layout = match a.store_op.as_ref().map(final_layout) {
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
                Some(StoreOp::Store(_)) => quote::format_ident!("Store"),
                None => quote::format_ident!("DontCare"),
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
                .map(|&c| {
                    quote::quote! {
                        colors.push((#c, ::sierra::Layout::ColorAttachmentOptimal));
                    }
                })
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

    let push_subpass_dependencies = input
        .dependencies
        .iter()
        .map(|s| {
            let SubpassDependency {
                src,
                src_stages,
                dst,
                dst_stages,
            } = s;

            let src = match src {
                None => quote::quote!(None),
                Some(src) => quote::quote!(Some(#src)),
            };

            let dst = match dst {
                None => quote::quote!(None),
                Some(dst) => quote::quote!(Some(#dst)),
            };

            quote::quote!(
                dependencies.push(::sierra::SubpassDependency {
                    src: #src,
                    src_stages: ::sierra::PipelineStageFlags::from_bits_truncate(#src_stages),
                    dst: #dst,
                    dst_stages: ::sierra::PipelineStageFlags::from_bits_truncate(#dst_stages),
                });
            )
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
    let dependency_count = input.dependencies.len();

    let clear_values = input
        .attachments
        .iter()
        .filter_map(|a| match &a.load_op {
            Some(LoadOp::Clear(Clear(_, ReferenceExpr::Member { member }))) => {
                Some(quote::quote!(::sierra::ClearValue::from(input.#member),))
            }
            Some(LoadOp::Clear(Clear(_, ReferenceExpr::Expr { expr, .. }))) => {
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
                    if self.render_pass.is_some() {
                        ::sierra::debug!("Recreating render pass");
                    } else {
                        ::sierra::debug!("Creating render pass");
                    }

                    self.render_pass = None;

                    let mut attachments = ::std::vec::Vec::with_capacity(#attachment_count);
                    #push_attachment_infos

                    let mut subpasses = ::std::vec::Vec::with_capacity(#subpass_count);
                    #push_subpass_infos

                    let mut dependencies = ::std::vec::Vec::with_capacity(#dependency_count);
                    #push_subpass_dependencies

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
                            ::sierra::trace!("Found framebuffer with compatible attachments");

                            ::sierra::FramebufferInfo {
                                render_pass: ::std::clone::Clone::clone(render_pass),
                                attachments: ::std::clone::Clone::clone(&fb.info().attachments),
                                extent: fb.info().extent,
                            }
                        }
                        None => {
                            ::sierra::debug!("Framebuffer with compatible attachments not found");

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
                            ::sierra::trace!("Found framebuffer with compatible attachments");

                            let fb = self.framebuffers.remove(fb_index);
                            self.framebuffers.push(fb);
                        },
                        None => {
                            ::sierra::debug!("Framebuffer with compatible attachments not found");

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
                Ok(encoder.with_framebuffer(fb, encoder.scope().to_scope([#clear_values])))
            }
        }
    )
}

fn initial_layout(load_op: &Option<LoadOp>) -> TokenStream {
    match load_op {
        Some(LoadOp::Clear(_)) => quote::quote!(::std::option::Option::None),
        None => quote::quote!(::std::option::Option::None),
        Some(LoadOp::Load(Load(_, ReferenceExpr::Expr { expr, .. }))) => {
            quote::quote!(::std::option::Option::Some(#expr))
        }
        Some(LoadOp::Load(Load(_, ReferenceExpr::Member { member }))) => {
            quote::quote!(::std::option::Option::Some(self.#member))
        }
    }
}

fn final_layout(store_op: &StoreOp) -> TokenStream {
    match store_op {
        StoreOp::Store(Store(_, ReferenceExpr::Expr { expr, .. })) => quote::quote!(#expr),
        StoreOp::Store(Store(_, ReferenceExpr::Member { member })) => quote::quote!(self.#member),
    }
}
