use {
    super::{
        instance::instance_type_name,
        layout::layout_type_name,
        parse::{Field, FieldType, Input},
    },
    proc_macro2::TokenStream,
};

pub(crate) fn generate(input: &Input) -> TokenStream {
    input
        .fields
        .iter()
        .map(|field| generate_for_input_field(&input.item_struct.ident, field))
        .chain(Some(generate_input_impl(input)))
        .collect::<TokenStream>()
}

fn generate_input_impl(input: &Input) -> TokenStream {
    let ident = &input.item_struct.ident;
    let layout_ident = layout_type_name(input);
    let instance_ident = instance_type_name(input);

    quote::quote! {
        impl ::sierra::DescriptorsInput for #ident {
            type Layout = #layout_ident;
            type Instance = #instance_ident;
        }
    }
}

fn generate_for_input_field(input_name: &syn::Ident, input_field: &Field) -> TokenStream {
    let field_member = &input_field.member;

    match &input_field.ty {
        FieldType::CombinedImageSampler(attr) => {
            let is_fresh_field = quote::format_ident!("is_fresh_{}", field_member);

            let get_field_descriptor = quote::format_ident!("get_{}_descriptor", field_member);

            match &attr.separate_sampler {
                Some(separate_sampler) => {
                    quote::quote!(
                        impl #input_name {
                            fn #is_fresh_field(&self, fresh: &::sierra::CombinedImageSampler) -> bool {
                                // combined_image_sampler
                                let current = ::sierra::CombinedImageSamplerEq {
                                    image: &self.#field_member,
                                    layout: ::sierra::Layout::ShaderReadOnlyOptimal,
                                    sampler: &self.#separate_sampler,
                                };
                                current == *fresh
                            }

                            fn #get_field_descriptor(
                                &self,
                                device: &::sierra::Device,
                            ) -> Result<::sierra::CombinedImageSampler, ::sierra::OutOfMemory> {
                                Ok(::sierra::CombinedImageSampler {
                                    view: ::sierra::MakeImageView::make_view(&self.#field_member, device)?,
                                    layout: ::sierra::Layout::ShaderReadOnlyOptimal,
                                    sampler: self.#separate_sampler.clone(),
                                })
                            }
                        }
                    )
                }
                None => quote::quote!(
                    impl #input_name {
                        fn #is_fresh_field(&self, fresh: &::sierra::CombinedImageSampler) -> bool {
                            self.#field_member == current
                        }

                        fn #get_field_descriptor(
                            &self,
                            device: &::sierra::Device,
                        ) -> Result<::sierra::CombinedImageSampler, ::sierra::OutOfMemory> {
                            Ok(self.#field_member.clone())
                        }
                    }
                ),
            }
        }
        FieldType::AccelerationStructure(_) => {
            let is_fresh_field = quote::format_ident!("is_fresh_{}", field_member);

            let get_field_descriptor = quote::format_ident!("get_{}_descriptor", field_member);

            quote::quote!(
                impl #input_name {
                    fn #is_fresh_field(&self, fresh: &::sierra::AccelerationStructure) -> bool {
                        // combined_image_sampler
                        self.#field_member == *fresh
                    }

                    fn #get_field_descriptor(
                        &self,
                        device: &::sierra::Device,
                    ) -> Result<::sierra::AccelerationStructure, ::sierra::OutOfMemory> {
                        Ok(self.#field_member.clone())
                    }
                }
            )
        }
        FieldType::Buffer(_) => {
            let is_fresh_field = quote::format_ident!("is_fresh_{}", field_member);

            let get_field_descriptor = quote::format_ident!("get_{}_descriptor", field_member);

            quote::quote!(
                impl #input_name {
                    fn #is_fresh_field(&self, fresh: &::sierra::BufferRegion) -> bool {
                        // combined_image_sampler
                        ::sierra::BufferRegionEq { buffer: & self.#field_member } == *fresh
                    }

                    fn #get_field_descriptor(
                        &self,
                        device: &::sierra::Device,
                    ) -> Result<::sierra::BufferRegion, ::sierra::OutOfMemory> {
                        Ok(self.#field_member.clone().into())
                    }
                }
            )
        }
    }
}
