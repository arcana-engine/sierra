use {
    super::{
        buffer,
        instance::instance_type_name,
        parse::{Descriptor, DescriptorType, Input},
    },
    crate::stage::{combined_stages_tokens, combined_stages_tokens_dedup},
    proc_macro2::TokenStream,
    std::convert::TryFrom as _,
};

pub(super) fn layout_type_name(input: &Input) -> syn::Ident {
    quote::format_ident!("{}Layout", input.item_struct.ident)
}

pub(super) fn generate(input: &Input) -> TokenStream {
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
        .collect::<Vec<_>>();

    if !input.uniforms.is_empty() {
        let stages = combined_stages_tokens_dedup(
            input.uniforms.iter().flat_map(|u| u.stages.iter().copied()),
        );

        let binding = u32::try_from(bindings.len()).expect("Too many descriptors");
        bindings.push(quote::quote!(
            ::sierra::DescriptorSetLayoutBinding {
                binding: #binding,
                ty: ::sierra::DescriptorType::UniformBuffer,
                count: 1,
                stages: #stages,
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

    quote::quote!(
        #[derive(Clone, Debug)]
        #[repr(transparent)]
        #doc_attr
        #vis struct #layout_ident {
            pub layout: ::sierra::DescriptorSetLayout
        }

        impl ::sierra::DescriptorsLayout for #layout_ident {
            type Instance = #instance_ident;

            fn new(device: &::sierra::Device) -> ::std::result::Result<Self, ::sierra::OutOfMemory> {
                let layout =
                    device.create_descriptor_set_layout(::sierra::DescriptorSetLayoutInfo {
                        bindings: vec![#(#bindings),*],
                        flags: ::sierra::DescriptorSetLayoutFlags::empty(),
                    })?;

                ::std::result::Result::Ok(#layout_ident { layout })
            }

            fn instantiate(&self) -> #instance_ident {
                #instance_ident::new(self)
            }
        }
    )
    .into()
}

fn generate_layout_binding(descriptor: &Descriptor, binding: u32) -> TokenStream {
    let ty = match descriptor.ty {
        DescriptorType::CombinedImageSampler(_) => {
            quote::format_ident!("CombinedImageSampler")
        }
        DescriptorType::AccelerationStructure(_) => {
            quote::format_ident!("AccelerationStructure")
        }
        DescriptorType::Buffer(buffer::Buffer {
            kind: buffer::Kind::Uniform,
            ..
        }) => {
            quote::format_ident!("UniformBuffer")
        }
        DescriptorType::Buffer(buffer::Buffer {
            kind: buffer::Kind::Storage,
            ..
        }) => {
            quote::format_ident!("StorageBuffer")
        }
    };

    let stages = combined_stages_tokens(descriptor.stages.iter().copied());

    quote::quote!(
        ::sierra::DescriptorSetLayoutBinding {
            binding: #binding,
            ty: ::sierra::DescriptorType::#ty,
            count: 1,
            stages: #stages,
            flags: ::sierra::DescriptorBindingFlags::empty(),
        }
    )
}
