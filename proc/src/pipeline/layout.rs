use std::convert::TryFrom;

use proc_macro2::TokenStream;

use super::parse::Input;

pub(super) fn layout_type_name(input: &Input) -> syn::Ident {
    quote::format_ident!("{}Layout", input.item_struct.ident)
}

pub(super) fn generate(input: &Input) -> TokenStream {
    let layout_ident = layout_type_name(input);

    let layout_sets = input
        .sets
        .iter()
        .map(|set| {
            let vis = &set.field.vis;
            let ident = &set.field.ident;
            let ty = &set.field.ty;
            quote::quote!(
                #vis #ident: <#ty as ::sierra::Descriptors>::Layout,
            )
        })
        .collect::<TokenStream>();

    let layout_sets_new = input
        .sets
        .iter()
        .map(|set| {
            let ident = &set.field.ident;
            let ty = &set.field.ty;
            quote::quote!(
                let #ident = <#ty as ::sierra::Descriptors>::layout(device)?;
            )
        })
        .collect::<TokenStream>();

    let layout_sets_init = input
        .sets
        .iter()
        .map(|set| {
            let ident = &set.field.ident;
            quote::quote!(
                #ident,
            )
        })
        .collect::<TokenStream>();

    let raw_set_layouts = input
        .sets
        .iter()
        .map(|set| {
            let ident = &set.field.ident;
            quote::quote!(
                ::std::clone::Clone::clone(::sierra::DescriptorsLayout::raw(&#ident))
            )
        })
        .collect::<Vec<_>>();

    let pipeline_descriptors = input
        .sets
        .iter()
        .enumerate()
        .map(|(index, set)| {
            let ty = &set.field.ty;
            let index = u32::try_from(index).expect("Too many sets");
            quote::quote!(
                impl ::sierra::UpdatedPipelineDescriptors<#layout_ident, #index> for <<#ty as ::sierra::Descriptors>::Instance as ::sierra::DescriptorsInstance<#ty>>::Updated {}
            )
        })
        .collect::<TokenStream>();

    let (_, push_constants_descs, push_constants_impls) = input
        .push_constants
        .iter()
        .fold((quote::quote!(0), quote::quote!{}, quote::quote!{}), |(mut offset, mut desc, mut impls), push_constants| {
            let field_type = &push_constants.field.ty;
            let sierra_layout = push_constants.layout.sierra_type();
            let stages = push_constants.stages.bits();

            let field_align_mask = quote::quote!(<#field_type as ::sierra::ShaderRepr<#sierra_layout>>::ALIGN_MASK);
            let this_offset = quote::quote!(::sierra::align_offset(#field_align_mask, #offset));
            let field_repr = quote::quote!(<#field_type as ::sierra::ShaderRepr<#sierra_layout>>::Type);

            offset = quote::quote!(::sierra::next_offset(#field_align_mask, #offset, ::std::mem::size_of::<#field_repr>()));

            desc.extend(quote::quote!(
                ::sierra::PushConstant {
                    stages: ::sierra::ShaderStageFlags::from_bits_truncate(#stages),
                    offset: #this_offset as u32,
                    size: ::std::mem::size_of::<#field_repr>() as u32,
                },
            ));

            impls.extend(quote::quote!(
                impl ::sierra::PipelinePushConstants<#layout_ident> for #field_type {
                    const STAGES: ::sierra::ShaderStageFlags = ::sierra::ShaderStageFlags::from_bits_truncate(#stages);
                    const OFFSET: u32 = #this_offset as u32;

                    type Repr = #field_repr;

                    fn to_repr(&self) -> #field_repr {
                        <#field_type as ::sierra::ShaderRepr<#sierra_layout>>::to_repr(self)
                    }
                }
            ));
            (offset, desc, impls)
        });

    let vis = &input.item_struct.vis;
    let ident = &input.item_struct.ident;

    let doc_attr = if cfg!(feature = "verbose-docs") {
        format!(
            "#[doc = \"[`sierra::PipelineInputLayout`] implementation for [`{}`]\"]",
            ident
        )
        .parse()
        .unwrap()
    } else {
        quote::quote!(#[doc(hidden)])
    };

    quote::quote!(
        #[derive(Clone, Debug)]
        #doc_attr
        #vis struct #layout_ident {
            pub pipeline_layout: ::sierra::PipelineLayout,
            #layout_sets
        }

        impl #layout_ident {
            pub fn new(device: &::sierra::Device) -> ::std::result::Result<Self, ::sierra::OutOfMemory> {
                #layout_sets_new

                let pipeline_layout = device.create_pipeline_layout(::sierra::PipelineLayoutInfo {
                    sets: ::std::vec![#(#raw_set_layouts),*],
                    push_constants: ::std::vec![#push_constants_descs],
                })?;

                Ok(#layout_ident {
                    pipeline_layout,
                    #layout_sets_init
                })
            }

            pub fn raw(&self) -> &::sierra::PipelineLayout {
                &self.pipeline_layout
            }

            pub fn bind_graphics<D, const N: u32>(&self, updated_descriptors: &D, encoder: &mut ::sierra::EncoderCommon)
            where
                D: ::sierra::UpdatedPipelineDescriptors<Self, N>,
            {
                encoder.bind_graphics_descriptor_sets(
                    &self.pipeline_layout,
                    N,
                    &[::sierra::UpdatedDescriptors::raw(updated_descriptors)],
                    &[],
                )
            }

            pub fn bind_compute<D, const N: u32>(&self, updated_descriptors: &D, encoder: &mut ::sierra::EncoderCommon)
            where
                D: ::sierra::UpdatedPipelineDescriptors<Self, N>,
            {
                encoder.bind_compute_descriptor_sets(
                    &self.pipeline_layout,
                    N,
                    &[::sierra::UpdatedDescriptors::raw(updated_descriptors)],
                    &[],
                )
            }

            pub fn bind_ray_tracing<D, const N: u32>(&self, updated_descriptors: &D, encoder: &mut ::sierra::EncoderCommon)
            where
                D: ::sierra::UpdatedPipelineDescriptors<Self, N>,
            {
                encoder.bind_ray_tracing_descriptor_sets(
                    &self.pipeline_layout,
                    N,
                    &[::sierra::UpdatedDescriptors::raw(updated_descriptors)],
                    &[],
                )
            }

            fn push_constants<P>(&self, push_constants: &P, encoder: &mut ::sierra::EncoderCommon)
            where
                P: ::sierra::PipelinePushConstants<Self>,
            {
                encoder.push_constants_pod(
                    &self.pipeline_layout,
                    P::STAGES,
                    P::OFFSET,
                    &[P::to_repr(push_constants)]
                );
            }
        }

        impl ::sierra::PipelineInputLayout for #layout_ident {
            fn new(device: &::sierra::Device) -> ::std::result::Result<Self, ::sierra::OutOfMemory> {
                Self::new(device)
            }

            fn raw(&self) -> &::sierra::PipelineLayout {
                self.raw()
            }

            fn bind_graphics<D, const N: u32>(&self, updated_descriptors: &D, encoder: &mut ::sierra::EncoderCommon)
            where
                D: ::sierra::UpdatedPipelineDescriptors<Self, N>,
            {
                self.bind_graphics(updated_descriptors, encoder);
            }

            fn bind_compute<D, const N: u32>(&self, updated_descriptors: &D, encoder: &mut ::sierra::EncoderCommon)
            where
                D: ::sierra::UpdatedPipelineDescriptors<Self, N>,
            {
                self.bind_compute(updated_descriptors, encoder);
            }

            fn bind_ray_tracing<D, const N: u32>(&self, updated_descriptors: &D, encoder: &mut ::sierra::EncoderCommon)
            where
                D: ::sierra::UpdatedPipelineDescriptors<Self, N>,
            {
                self.bind_ray_tracing(updated_descriptors, encoder);
            }

            fn push_constants<P>(&self, push_constants: &P, encoder: &mut ::sierra::EncoderCommon)
            where
                P: ::sierra::PipelinePushConstants<Self>,
            {
                self.push_constants(push_constants, encoder);
            }
        }

        #pipeline_descriptors

        #push_constants_impls
    )
}
