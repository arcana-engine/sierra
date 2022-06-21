use std::convert::TryFrom;

use proc_macro2::TokenStream;
use syn::spanned::Spanned;

use super::{
    acceleration_structure::AccelerationStructure,
    buffer, image,
    instance::instance_type_name,
    parse::{Descriptor, DescriptorType, Input},
    sampler::Sampler,
};

use crate::shader_stage::combined_shader_stage_flags;

pub(super) fn layout_type_name(input: &Input) -> syn::Ident {
    quote::format_ident!("{}Layout", input.item_struct.ident)
}

pub(super) fn generate(input: &Input) -> Result<TokenStream, syn::Error> {
    let layout_ident = layout_type_name(input);
    let instance_ident = instance_type_name(input);

    let mut bindings = input
        .descriptors
        .iter()
        .enumerate()
        .map(|(binding, descriptor)| {
            generate_layout_binding(
                descriptor,
                u32::try_from(binding).expect("Too many descriptors"),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    if !input.uniforms.is_empty() {
        let stages = combined_shader_stage_flags(
            input
                .uniforms
                .iter()
                .flat_map(|u| u.stages.flags.iter().copied()),
        );

        let binding = u32::try_from(bindings.len()).expect("Too many descriptors");
        bindings.push(quote::quote!(
            ::sierra::DescriptorSetLayoutBinding {
                binding: #binding,
                ty: ::sierra::DescriptorType::UniformBuffer,
                count: 1,
                stages: ::sierra::ShaderStageFlags::from_bits_truncate(#stages),
                flags: ::sierra::DescriptorBindingFlags::empty(),
            }
        ));
    }

    let vis = &input.item_struct.vis;
    let ident = &input.item_struct.ident;

    let doc_attr = if cfg!(feature = "verbose-docs") {
        format!(
            "#[doc = \"[`sierra::DescriptorsLayout`] implementation for [`{}`]\"]",
            ident
        )
        .parse()
        .unwrap()
    } else {
        quote::quote!(#[doc(hidden)])
    };

    let tokens = quote::quote!(
        #[derive(Clone, Debug)]
        #[repr(transparent)]
        #doc_attr
        #vis struct #layout_ident {
            pub layout: ::sierra::DescriptorSetLayout
        }

        impl #layout_ident {
            pub fn new(device: &::sierra::Device) -> ::std::result::Result<Self, ::sierra::OutOfMemory> {
                let layout =
                    device.create_descriptor_set_layout(::sierra::DescriptorSetLayoutInfo {
                        bindings: ::std::vec![#(#bindings),*],
                        flags: ::sierra::DescriptorSetLayoutFlags::empty(),
                    })?;

                ::std::result::Result::Ok(#layout_ident { layout })
            }

            pub fn raw(&self) -> &::sierra::DescriptorSetLayout {
                &self.layout
            }

            pub fn instance(&self) -> #instance_ident {
                #instance_ident::new(self)
            }
        }


        impl ::sierra::DescriptorsLayout for #layout_ident {
            type Instance = #instance_ident;

            fn raw(&self) -> &::sierra::DescriptorSetLayout {
                self.raw()
            }

            fn instance(&self) -> #instance_ident {
                self.instance()
            }
        }
    );
    Ok(tokens)
}

fn generate_layout_binding(
    descriptor: &Descriptor,
    binding: u32,
) -> Result<TokenStream, syn::Error> {
    let desc_ty = match descriptor.desc_ty {
        DescriptorType::Sampler(Sampler { kw }) => {
            quote::quote_spanned!(kw.span() => ::sierra::DescriptorType::Sampler)
        }
        DescriptorType::Image(image::Image {
            kw,
            kind: None | Some(image::Kind::Sampled(_)),
            ..
        }) => {
            quote::quote_spanned!(kw.span() => ::sierra::DescriptorType::SampledImage)
        }
        DescriptorType::Image(image::Image {
            kw,
            kind: Some(image::Kind::Storage(_)),
            ..
        }) => {
            quote::quote_spanned!(kw.span() => ::sierra::DescriptorType::StorageImage)
        }
        DescriptorType::Buffer(buffer::Buffer {
            kw,
            kind: None | Some(buffer::Kind::Uniform(_)),
            texel: None,
        }) => {
            quote::quote_spanned!(kw.span() => ::sierra::DescriptorType::UniformBuffer)
        }
        DescriptorType::Buffer(buffer::Buffer {
            kw,
            kind: Some(buffer::Kind::Storage(_)),
            texel: None,
        }) => {
            quote::quote_spanned!(kw.span() => ::sierra::DescriptorType::StorageTexelBuffer)
        }
        DescriptorType::Buffer(buffer::Buffer {
            kw,
            kind: None | Some(buffer::Kind::Uniform(_)),
            texel: Some(_),
        }) => {
            quote::quote_spanned!(kw.span() => ::sierra::DescriptorType::UniformTexelBuffer)
        }
        DescriptorType::Buffer(buffer::Buffer {
            kw,
            kind: Some(buffer::Kind::Storage(_)),
            texel: Some(_),
        }) => {
            quote::quote_spanned!(kw.span() => ::sierra::DescriptorType::StorageTexelBuffer)
        }
        DescriptorType::AccelerationStructure(AccelerationStructure { kw }) => {
            quote::quote_spanned!(kw.span() => ::sierra::DescriptorType::AccelerationStructure)
        }
    };

    let stages = descriptor.stages.bits();
    let flags = descriptor.flags.bits();

    let descriptor_kind = descriptor.desc_ty.descriptor_kind()?;

    let ty = &descriptor.field.ty;
    Ok(quote::quote_spanned!(
        descriptor.field.span() =>
        ::sierra::DescriptorSetLayoutBinding {
            binding: #binding,
            ty: #desc_ty,
            count: <#ty as ::sierra::DescriptorBindingArray<#descriptor_kind>>::COUNT,
            stages: ::sierra::ShaderStageFlags::from_bits_truncate(#stages),
            flags: ::sierra::DescriptorBindingFlags::from_bits_truncate(#flags | <#ty as ::sierra::DescriptorBindingArray<#descriptor_kind>>::FLAGS.bits()),
        }
    ))
}
