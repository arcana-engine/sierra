use proc_macro2::TokenStream;
use syn::spanned::Spanned;

use super::{layout::layout_type_name, parse::Input};

pub(super) fn instance_type_name(input: &Input) -> syn::Ident {
    quote::format_ident!("{}Instance", input.item_struct.ident)
}

pub(super) fn generate(input: &Input) -> TokenStream {
    let ident = &input.item_struct.ident;
    let layout_ident = layout_type_name(input);
    let instance_ident = instance_type_name(input);
    let elem_ident = quote::format_ident!("{}Elem", instance_ident);

    let descriptors: TokenStream = input
        .descriptors
        .iter()
        .map(|input| {
            let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
            let ty = &input.field.ty;

            let descriptor_kind = input.desc_ty.descriptor_kind();

            quote::quote_spanned!(
                input.field.ty.span() => pub #descriptor_field: ::std::option::Option<<#ty as ::sierra::DescriptorBindingArray<#descriptor_kind>>::DescriptorArray>,
            )
        })
        .collect();

    let update_descriptor_statements: TokenStream = input
        .descriptors
        .iter()
        .map(|input| {
            let field = &input.member;

            let descriptor_kind = input.desc_ty.descriptor_kind();
            let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
            let write_descriptor = quote::format_ident!("write_{}_descriptor", input.member);
            quote::quote!(
                let #write_descriptor;
                match &elem.#descriptor_field {
                    Some(descriptors) if sierra::DescriptorBindingArray::<#descriptor_kind>::is_compatible(&input.#field, descriptors) => {
                        #write_descriptor = false;
                    }
                    _ => {
                        elem.#descriptor_field = Some(::sierra::DescriptorBindingArray::<#descriptor_kind>::get_descriptors(&input.#field, device)?);
                        write_descriptor_any = true;
                        #write_descriptor = true;
                    }
                }
            )
        })
        .collect();

    let mut binding = 0u32;
    let write_updated_descriptor_statements: TokenStream = input
        .descriptors
        .iter()
        .map(|input| {
            let span = input.field.ty.span();
            let descriptor_kind = input.desc_ty.descriptor_kind();
            let descriptors = quote::quote_spanned!(span => <#descriptor_kind as ::sierra::DescriptorKind>::descriptors(descriptors));
            let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
            let write_descriptor = quote::format_ident!("write_{}_descriptor", input.member);

            let stream = quote::quote!(
                if #write_descriptor {
                    let descriptors: &_ = elem.#descriptor_field.as_ref().unwrap();
                    writes.push(::sierra::DescriptorSetWrite {
                        binding: #binding,
                        element: 0,
                        descriptors: #descriptors,
                    });
                }
            );

            binding += 1;
            stream
        })
        .collect();

    let updated_descriptor_assertions: TokenStream = input
        .descriptors
        .iter()
        .map(|input| {
            let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
            quote::quote!(
                debug_assert!(elem.#descriptor_field.is_some());
            )
        })
        .collect();

    let new_cycle_elem_descriptors: TokenStream = input
        .descriptors
        .iter()
        .map(|input| {
            let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
            quote::quote!(
                #descriptor_field: ::std::option::Option::None,
            )
        })
        .collect();

    let vis = &input.item_struct.vis;
    let uniforms_ident = quote::format_ident!("{}Uniforms", input.item_struct.ident);

    let uniforms_field = if input.uniforms.is_empty() {
        TokenStream::new()
    } else {
        quote::quote!(pub uniforms_buffer: ::std::option::Option<(#uniforms_ident, ::sierra::BufferRange)>,)
    };

    let new_cycle_elem_uniforms_buffer = if input.uniforms.is_empty() {
        TokenStream::new()
    } else {
        quote::quote!(uniforms_buffer: ::std::option::Option::None,)
    };

    let update_uniforms_statement = if input.uniforms.is_empty() {
        TokenStream::new()
    } else {
        quote::quote!(
            if elem.uniforms_buffer.is_none() {
                let mut uniforms: #uniforms_ident = ::sierra::bytemuck::Zeroable::zeroed();
                uniforms.copy_from_input(input);
                let buffer = device.create_buffer(::sierra::BufferInfo {
                    align: 255,
                    size: ::std::convert::TryFrom::try_from(::std::mem::size_of::<#uniforms_ident>() as u64).map_err(|_| ::sierra::OutOfMemory)?,
                    usage: ::sierra::BufferUsage::UNIFORM | ::sierra::BufferUsage::TRANSFER_DST,
                })?;

                elem.uniforms_buffer = Some((uniforms, buffer.into()));
                write_uniforms = true;
            } else {
                write_uniforms = false;
                elem.uniforms_buffer.as_mut().unwrap().0.copy_from_input(input);
            }
        )
    };

    let write_uniforms_statement = if input.uniforms.is_empty() {
        TokenStream::new()
    } else {
        quote::quote!(
            if write_uniforms {
                writes.push(::sierra::DescriptorSetWrite {
                    binding: #binding,
                    element: 0,
                    descriptors: ::sierra::DescriptorSlice::UniformBuffer(::std::slice::from_ref(&elem.uniforms_buffer.as_ref().unwrap().1)),
                });
            }
        )
    };

    let update_uniforms_buffer_statement = if input.uniforms.is_empty() {
        TokenStream::new()
    } else {
        quote::quote!(
            let (uniforms, buffer) = elem.uniforms_buffer.as_ref().unwrap();
            encoder.update_buffer(scope.to_scope(buffer.buffer.clone()), 0, scope.to_scope([*uniforms]));
        )
    };

    let doc_attr = if cfg!(feature = "verbose-docs") {
        format!(
            "#[doc = \"[`sierra::DescriptorsInstance`] implementation for [`{}`]\"]",
            ident
        )
        .parse()
        .unwrap()
    } else {
        quote::quote!(#[doc(hidden)])
    };

    let max_writes = input.descriptors.len() + (!input.uniforms.is_empty()) as usize;

    let cycle_capacity = input.cycle_capacity;

    quote::quote!(
        #doc_attr
        #vis struct #instance_ident {
            pub layout: ::sierra::DescriptorSetLayout,
            pub cycle: ::sierra::arrayvec::ArrayVec<#elem_ident, #cycle_capacity>,
            pub cycle_next: usize,
        }

        #doc_attr
        #vis struct #elem_ident {
            pub set: ::sierra::DescriptorSet,
            #descriptors
            #uniforms_field
        }

        impl ::sierra::UpdatedDescriptors for #elem_ident {
            fn raw(&self) -> &::sierra::DescriptorSet {
                &self.set
            }
        }

        impl #instance_ident {
            pub fn new(layout: &#layout_ident) -> Self {
                #instance_ident {
                    layout: layout.layout.clone(),
                    cycle: ::sierra::arrayvec::ArrayVec::new(),
                    cycle_next: 0,
                }
            }

            pub fn clear(&mut self) {
                self.cycle.clear();
            }

            pub fn update(
                &mut self,
                input: &#ident,
                device: &::sierra::Device,
                encoder: &mut ::sierra::Encoder,
            ) -> ::std::result::Result<&#elem_ident, ::sierra::DescriptorsAllocationError> {
                if self.cycle.is_empty() {
                    self.cycle.push(#elem_ident {
                        set: device.create_descriptor_set(::sierra::DescriptorSetInfo {
                            layout: self.layout.clone(),
                        })?.share(),
                        #new_cycle_elem_descriptors
                        #new_cycle_elem_uniforms_buffer
                    });
                }

                if self.cycle_next >= self.cycle.len() {
                    self.cycle_next = 0;
                }

                let start = self.cycle_next;

                loop {
                    match self.cycle[self.cycle_next].set.is_unused() {
                        false => {
                            let next = (self.cycle_next + 1) % self.cycle.len();
                            if next == start {
                                let new_elem = #elem_ident {
                                    set: device.create_descriptor_set(::sierra::DescriptorSetInfo {
                                        layout: self.layout.clone(),
                                    })?.share(),
                                    #new_cycle_elem_descriptors
                                    #new_cycle_elem_uniforms_buffer
                                };

                                // No sets available yet.
                                if self.cycle.len() < self.cycle.capacity() {
                                    self.cycle.insert(start + 1, new_elem);
                                    self.cycle_next = start + 1;
                                } else {
                                    self.cycle[start] = new_elem;
                                    self.cycle_next = start;
                                }

                                break;
                            }
                            self.cycle_next = next;
                        }
                        true => break,
                    }
                }

                let scope = encoder.scope();

                let elem = &mut self.cycle[self.cycle_next];

                #[allow(unused)]
                let mut write_uniforms = false;
                #[allow(unused)]
                let mut write_descriptor_any = false;

                #update_uniforms_statement
                #update_descriptor_statements

                if write_descriptor_any || write_uniforms {
                    let writable_set: &mut ::sierra::WritableDescriptorSet = unsafe {
                        // # Safety
                        // Loop above guarantees uniqueness.
                        elem.set.as_writable()
                    };

                    let mut writes = ::sierra::arrayvec::ArrayVec::<_, #max_writes>::new();
                    #write_uniforms_statement
                    #write_updated_descriptor_statements

                    device.update_descriptor_sets(&mut [::sierra::UpdateDescriptorSet {
                        set: writable_set,
                        writes: &writes,
                        copies: &[],
                    }]);
                }

                #update_uniforms_buffer_statement
                #updated_descriptor_assertions

                self.cycle_next += 1;
                ::std::result::Result::Ok(&*elem)
            }

            pub fn raw_layout(&self) -> &::sierra::DescriptorSetLayout {
                &self.layout
            }
        }

        impl ::sierra::DescriptorsInstance<#ident> for #instance_ident {
            type Updated = #elem_ident;

            fn update(
                &mut self,
                input: &#ident,
                device: &::sierra::Device,
                encoder: &mut ::sierra::Encoder,
            ) -> ::std::result::Result<&#elem_ident, ::sierra::DescriptorsAllocationError> {
                self.update(input, device, encoder)
            }

            fn raw_layout(&self) -> &::sierra::DescriptorSetLayout {
                self.raw_layout()
            }
        }
    )
}
