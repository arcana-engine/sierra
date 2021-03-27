use {
    super::{
        buffer,
        layout::layout_type_name,
        parse::{FieldType, Input},
    },
    proc_macro2::TokenStream,
};

pub(crate) fn instance_type_name(input: &Input) -> syn::Ident {
    quote::format_ident!("{}Instance", input.item_struct.ident)
}

pub(crate) fn generate(input: &Input) -> TokenStream {
    let ident = &input.item_struct.ident;
    let layout_ident = layout_type_name(input);
    let instance_ident = instance_type_name(input);
    let elem_ident = quote::format_ident!("{}Elem", instance_ident);

    let fields: TokenStream = input
        .fields
        .iter()
        .map(|input| match &input.ty {
            FieldType::CombinedImageSampler(_) => {
                let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
                quote::quote!(
                    #descriptor_field: ::std::option::Option<::sierra::CombinedImageSampler>,
                )
            }
            FieldType::AccelerationStructure(_) => {
                let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
                quote::quote!(
                    #descriptor_field: ::std::option::Option<::sierra::AccelerationStructure>,
                )
            }
            FieldType::Buffer(_) => {
                let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
                quote::quote!(
                    #descriptor_field: ::std::option::Option<::sierra::BufferRegion>,
                )
            }
        })
        .collect();

    let update_field_statements: TokenStream = input
        .fields
        .iter()
        .map(|input| match &input.ty {
            ty if ty.is_descriptor() => {
                let descriptor_field =
                    quote::format_ident!("descriptor_{}", input.member);

                let is_fresh_field =
                    quote::format_ident!("is_fresh_{}", input.member);

                let get_field_descriptor =
                    quote::format_ident!("get_{}_descriptor", input.member);

                let binding = input.binding;

                let descriptors = match ty {
                    FieldType::CombinedImageSampler(_) => {
                        quote::quote!(::sierra::Descriptors::CombinedImageSampler(std::slice::from_ref(descriptor)))
                    }
                    FieldType::AccelerationStructure(_) => {
                        quote::quote!(::sierra::Descriptors::AccelerationStructure(std::slice::from_ref(descriptor)))
                    }
                    FieldType::Buffer(buffer::Buffer { kind: buffer::Kind::Uniform,.. }) => {
                        quote::quote!(::sierra::Descriptors::UniformBuffer(std::slice::from_ref(descriptor)))
                    }
                    FieldType::Buffer(buffer::Buffer { kind: buffer::Kind::Storage,.. }) => {
                        quote::quote!(::sierra::Descriptors::StorageBuffer(std::slice::from_ref(descriptor)))
                    }
                };

                quote::quote!(
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
                )
            }
            _ => unreachable!(),
        })
        .collect();

    let get_update_field_assertions: TokenStream = input
        .fields
        .iter()
        .map(|input| match &input.ty {
            ty if ty.is_descriptor() => {
                let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
                quote::quote!(
                    assert!(elem.#descriptor_field.is_some());
                )
            }
            _ => unreachable!(),
        })
        .collect();

    let new_cycle_elem_fields: TokenStream = input
        .fields
        .iter()
        .map(|input| match &input.ty {
            ty if ty.is_descriptor() => {
                let descriptor_field = quote::format_ident!("descriptor_{}", input.member);
                quote::quote!(
                    #descriptor_field: ::std::option::Option::None,
                )
            }
            _ => unreachable!(),
        })
        .collect();

    let vis = &input.item_struct.vis;

    quote::quote!(
        #vis struct #instance_ident {
            layout: ::sierra::DescriptorSetLayout,
            cycle: ::std::vec::Vec<#elem_ident>,
        }

        struct #elem_ident {
            set: ::sierra::DescriptorSet,
            #fields
        }

        impl #instance_ident {
            pub fn new(layout: &#layout_ident) -> Self {
                #instance_ident {
                    layout: layout.0.clone(),
                    cycle: ::std::vec::Vec::new(),
                }
            }

            fn new_cycle_elem(&self, device: &::sierra::Device) -> ::std::result::Result<#elem_ident, ::sierra::OutOfMemory> {
                ::std::result::Result::Ok(#elem_ident {
                    set: device.create_descriptor_set(::sierra::DescriptorSetInfo {
                        layout: self.layout.clone(),
                    })?,
                    #new_cycle_elem_fields
                })
            }

            fn get_updated(&self, fence: usize) -> &::sierra::DescriptorSet {
                let elem = self.cycle.get(fence).expect("`fence` is out of bounds. call `update` with this `fence` value first");

                #get_update_field_assertions

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
            ) -> ::std::result::Result<(), ::sierra::OutOfMemory> {
                while self.cycle.len() <= fence {
                    let new_elem = self.new_cycle_elem(device)?;
                    self.cycle.push(new_elem);
                }

                let elem = self.cycle.get_mut(fence).unwrap();

                #update_field_statements

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
