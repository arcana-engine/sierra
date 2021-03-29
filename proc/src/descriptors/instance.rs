use {
    super::{
        buffer,
        layout::layout_type_name,
        parse::{DescriptorType, Input},
    },
    proc_macro2::TokenStream,
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
        .filter_map(|input| match &input.ty {
            DescriptorType::CombinedImageSampler(_) => {
                let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
                Some(quote::quote!(
                    pub #descriptor_field: ::std::option::Option<::sierra::CombinedImageSampler>,
                ))
            }
            DescriptorType::AccelerationStructure(_) => {
                let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
                Some(quote::quote!(
                    pub #descriptor_field: ::std::option::Option<::sierra::AccelerationStructure>,
                ))
            }
            DescriptorType::Buffer(_) => {
                let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
                Some(quote::quote!(
                    pub #descriptor_field: ::std::option::Option<::sierra::BufferRegion>,
                ))
            }
        })
        .collect();

    let mut binding = 0u32;

    let update_descriptor_statements: TokenStream = input
        .descriptors
        .iter()
        .filter_map(|input| {
            let descriptors = match input.ty {
                DescriptorType::CombinedImageSampler(_) => {
                    Some(quote::quote!(::sierra::Descriptors::CombinedImageSampler(std::slice::from_ref(descriptor))))
                }
                DescriptorType::AccelerationStructure(_) => {
                    Some(quote::quote!(::sierra::Descriptors::AccelerationStructure(std::slice::from_ref(descriptor))))
                }
                DescriptorType::Buffer(buffer::Buffer { kind: buffer::Kind::Uniform,.. }) => {
                    Some(quote::quote!(::sierra::Descriptors::UniformBuffer(std::slice::from_ref(descriptor))))
                }
                DescriptorType::Buffer(buffer::Buffer { kind: buffer::Kind::Storage,.. }) => {
                    Some(quote::quote!(::sierra::Descriptors::StorageBuffer(std::slice::from_ref(descriptor))))
                }
            }?;

                let descriptor_field =
                    quote::format_ident!("descriptor_{}", input.member);

                let is_fresh_field =
                    quote::format_ident!("is_fresh_{}", input.member);

                let get_field_descriptor =
                    quote::format_ident!("get_{}_descriptor", input.member);

                let stream = quote::quote!(
                    match &mut elem.#descriptor_field {
                        Some(descriptor) if input.#is_fresh_field(descriptor) => {}
                        _ => {
                            elem.#descriptor_field = None;
                            let descriptor: &_ = elem.#descriptor_field.get_or_insert(input.#get_field_descriptor(device)?);
                            writes.extend(Some(::sierra::WriteDescriptorSet {
                                set: &elem.set,
                                binding: #binding,
                                element: 0,
                                descriptors: #descriptors,
                            }));
                        }
                    }
                );

                binding += 1;
                Some(stream)
            
        })
        .collect();

    let get_update_descriptor_assertions: TokenStream = input
        .descriptors
        .iter()
        .map(|input| {
            let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
            quote::quote!(
                assert!(elem.#descriptor_field.is_some());
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
        quote::quote!(pub uniforms_buffer: ::std::option::Option<(#uniforms_ident, ::sierra::BufferRegion)>,)
    };

    let new_cycle_elem_uniforms_buffer = if input.uniforms.is_empty() {
        TokenStream::new()
    } else {
        quote::quote!(
            uniforms_buffer: ::std::option::Option::None,
        )
    };

    let update_uniforms_statements = if input.uniforms.is_empty() {
        TokenStream::new()
    } else {
        quote::quote!(
            if elem.uniforms_buffer.is_none() {
                let mut uniforms: #uniforms_ident = ::sierra::Zeroable::zeroed();
                uniforms.copy_from_input(input);
                let buffer = device.create_buffer(::sierra::BufferInfo {
                    align: 255,
                    size: ::std::convert::TryFrom::try_from(::std::mem::size_of::<#uniforms_ident>() as u64).map_err(|_| ::sierra::OutOfMemory)?,
                    usage: ::sierra::BufferUsage::UNIFORM | ::sierra::BufferUsage::TRANSFER_DST,
                })?;

                elem.uniforms_buffer = Some((uniforms, buffer.into()));

                writes.extend(Some(::sierra::WriteDescriptorSet {
                    set: &elem.set,
                    binding: #binding,
                    element: 0,
                    descriptors: ::sierra::Descriptors::UniformBuffer(::std::slice::from_ref(&elem.uniforms_buffer.as_ref().unwrap().1)),
                }));
            } else {
                elem.uniforms_buffer.as_mut().unwrap().0.copy_from_input(input);
            }

            let (uniforms, buffer) = elem.uniforms_buffer.as_ref().unwrap();
            encoder.update_buffer(&buffer.buffer, 0, ::std::slice::from_ref(uniforms));
        )
    };

    let doc_attr = if cfg!(feature = "verbose-docs") {
        format!("#[doc = \"[`sierra::DescriptorsInstance`] implementation for [`{}`]\"]", ident).parse().unwrap()
    } else {
        quote::quote!(#[doc(hidden)])
    };

    quote::quote!(
        #doc_attr
        #vis struct #instance_ident {
            pub layout: ::sierra::DescriptorSetLayout,
            pub cycle: ::std::vec::Vec<#elem_ident>,
        }

        #doc_attr
        #vis struct #elem_ident {
            pub set: ::sierra::DescriptorSet,
            #descriptors
            #uniforms_field
        }

        impl #instance_ident {
            pub fn new(layout: &#layout_ident) -> Self {
                #instance_ident {
                    layout: layout.layout.clone(),
                    cycle: ::std::vec::Vec::new(),
                }
            }

            fn new_cycle_elem(&self, device: &::sierra::Device) -> ::std::result::Result<#elem_ident, ::sierra::OutOfMemory> {
                ::std::result::Result::Ok(#elem_ident {
                    set: device.create_descriptor_set(::sierra::DescriptorSetInfo {
                        layout: self.layout.clone(),
                    })?,
                    #new_cycle_elem_descriptors
                    #new_cycle_elem_uniforms_buffer
                })
            }

            fn get_updated(&self, fence: usize) -> &::sierra::DescriptorSet {
                let elem = self.cycle.get(fence).expect("`fence` is out of bounds. call `update` with this `fence` value first");

                #get_update_descriptor_assertions

                &elem.set
            }
        }

        impl ::sierra::DescriptorsInstance for #instance_ident {
            type Input = #ident;

            fn update<'a>(
                &'a mut self,
                input: &#ident,
                fence: usize,
                device: &::sierra::Device,
                writes: &mut impl ::std::iter::Extend<::sierra::WriteDescriptorSet<'a>>,
                encoder: &mut ::sierra::Encoder<'a>,
            ) -> ::std::result::Result<(), ::sierra::OutOfMemory> {
                while self.cycle.len() <= fence {
                        let new_elem = self.new_cycle_elem(device)?;
                        self.cycle.push(new_elem);
                }
                let elem = self.cycle.get_mut(fence).unwrap();
                #update_uniforms_statements
                #update_descriptor_statements

                ::std::result::Result::Ok(())
            }

            fn bind_graphics<'a>(
                &'a self,
                fence: usize,
                layout: &'a ::sierra::PipelineLayout,
                index: u32,
                encoder: &mut ::sierra::EncoderCommon<'a>,
            ) {
                debug_assert_eq!(<usize as ::std::convert::TryFrom<u32>>::try_from(index).map(|index| &layout.info().sets[index]), Ok(&self.layout));

                let set = self.get_updated(fence);

                encoder.bind_graphics_descriptor_sets(
                    layout,
                    index,
                    ::std::slice::from_ref(set),
                    &[],
                );
            }

            fn bind_compute<'a>(
                &'a self,
                fence: usize,
                layout: &'a ::sierra::PipelineLayout,
                index: u32,
                encoder: &mut ::sierra::EncoderCommon<'a>,
            ) {
                debug_assert_eq!(<usize as ::std::convert::TryFrom<u32>>::try_from(index).map(|index| &layout.info().sets[index]), Ok(&self.layout));

                let set = self.get_updated(fence);

                encoder.bind_compute_descriptor_sets(
                    layout,
                    index,
                    ::std::slice::from_ref(set),
                    &[],
                );
            }

            fn bind_ray_tracing<'a>(
                &'a self,
                fence: usize,
                layout: &'a ::sierra::PipelineLayout,
                index: u32,
                encoder: &mut ::sierra::EncoderCommon<'a>,
            ) {
                debug_assert_eq!(<usize as ::std::convert::TryFrom<u32>>::try_from(index).map(|index| &layout.info().sets[index]), Ok(&self.layout));

                let set = self.get_updated(fence);

                encoder.bind_ray_tracing_descriptor_sets(
                    layout,
                    index,
                    ::std::slice::from_ref(set),
                    &[],
                );
            }
        }
    )
}
