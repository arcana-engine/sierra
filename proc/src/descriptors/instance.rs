use {
    super::{
        buffer, image,
        layout::layout_type_name,
        parse::{DescriptorType, Input},
    },
    proc_macro2::TokenStream,
    syn::spanned::Spanned,
};

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
            quote::quote_spanned!(
                input.field.ty.span() => pub #descriptor_field: ::std::option::Option<<#ty as ::sierra::TypedDescriptorBinding>::Descriptors>,
            )
        })
        .collect();

    let update_descriptor_statements: TokenStream = input
        .descriptors
        .iter()
        .map(|input| {
            let field = &input.member;

            let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
            let write_descriptor = quote::format_ident!("write_{}_descriptor", input.member);

            let stream = quote::quote!(
                let #write_descriptor;
                match &elem.#descriptor_field {
                    Some(descriptors) if sierra::TypedDescriptorBinding::eq(&input.#field, descriptors) => {
                        #write_descriptor = false;
                    }
                    _ => {
                        elem.#descriptor_field = Some(::sierra::TypedDescriptorBinding::get_descriptors(&input.#field, device)?);
                        #write_descriptor = true;
                    }
                }
            );

            stream
        })
        .collect();

    let mut binding = 0u32;
    let write_updated_descriptor_statements: TokenStream = input
        .descriptors
        .iter()
        .filter_map(|input| {
            let span = input.field.ty.span();
            let descriptors = match input.desc_ty {
                DescriptorType::Sampler(_) => Some(quote::quote_spanned! {
                    span => <::sierra::SamplerDescriptor as ::sierra::TypedDescriptor>::descriptors(descriptors)
                }),
                DescriptorType::Image(image::Image { kind: image::Kind::Sampled}) => Some(quote::quote_spanned! {
                    span => <::sierra::SampledImageDescriptor as ::sierra::TypedDescriptor>::descriptors(descriptors)
                }),
                DescriptorType::Image(image::Image { kind: image::Kind::Storage }) => Some(quote::quote_spanned! {
                    span => <::sierra::StorageImageDescriptor as ::sierra::TypedDescriptor>::descriptors(descriptors)
                }),
                DescriptorType::AccelerationStructure(_) => Some(quote::quote_spanned! {
                    span => <::sierra::AccelerationStructureDescriptor as ::sierra::TypedDescriptor>::descriptors(descriptors)
                }),
                DescriptorType::Buffer(buffer::Buffer {
                    kind: buffer::Kind::Uniform,
                    texel: false,
                }) => Some(quote::quote_spanned! {
                    span=> <::sierra::UniformBufferDescriptor as ::sierra::TypedDescriptor>::descriptors(descriptors)
                }),
                DescriptorType::Buffer(buffer::Buffer {
                    kind: buffer::Kind::Storage,
                    texel: false,
                }) => Some(quote::quote_spanned! {
                    span=> <::sierra::StorageBufferDescriptor as ::sierra::TypedDescriptor>::descriptors(descriptors)
                }),
                DescriptorType::Buffer(buffer::Buffer {
                    kind: buffer::Kind::Uniform,
                    texel: true,
                }) => Some(quote::quote_spanned! {
                    span=> <::sierra::UniformTexelBufferDescriptor as ::sierra::TypedDescriptor>::descriptors(descriptors)
                }),
                DescriptorType::Buffer(buffer::Buffer {
                    kind: buffer::Kind::Storage,
                    texel: true,
                }) => Some(quote::quote_spanned! {
                    span=> <::sierra::StorageTexelBufferDescriptor as ::sierra::TypedDescriptor>::descriptors(descriptors)
                }),
            }?;

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
            Some(stream)
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
            let write_uniforms;
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
                    descriptors: ::sierra::Descriptors::UniformBuffer(::std::slice::from_ref(&elem.uniforms_buffer.as_ref().unwrap().1)),
                });
            }

            let (uniforms, buffer) = elem.uniforms_buffer.as_ref().unwrap();
            encoder.update_buffer(&buffer.buffer, 0, ::std::slice::from_ref(uniforms));
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

    quote::quote!(
        #doc_attr
        #vis struct #instance_ident {
            pub layout: ::sierra::DescriptorSetLayout,
            pub cycle: ::sierra::arrayvec::ArrayVec<#elem_ident, 5>,
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

            pub fn update<'a, 'b: 'a>(
                &'b mut self,
                input: &#ident,
                device: &::sierra::Device,
                encoder: &mut ::sierra::Encoder<'a>,
            ) -> ::std::result::Result<&'b #elem_ident, ::sierra::DescriptorsAllocationError> {
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
                    match self.cycle[self.cycle_next].set.is_writtable() {
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

                let elem = &mut self.cycle[self.cycle_next];
                #update_uniforms_statement
                #update_descriptor_statements

                {
                    let writable_set: &mut ::sierra::WritableDescriptorSet = unsafe {
                        // # Safety
                        // Loop above guaratees uniqueness.
                        elem.set.as_writtable()
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

            fn update<'a, 'b: 'a>(
                &'b mut self,
                input: &#ident,
                device: &::sierra::Device,
                encoder: &mut ::sierra::Encoder<'a>,
            ) -> ::std::result::Result<&'b #elem_ident, ::sierra::DescriptorsAllocationError> {
                self.update(input, device, encoder)
            }

            fn raw_layout(&self) -> &::sierra::DescriptorSetLayout {
                self.raw_layout()
            }
        }
    )
}
