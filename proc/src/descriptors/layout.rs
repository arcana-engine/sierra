use {
    super::{
        buffer, combined_binding_flags, image,
        instance::instance_type_name,
        parse::{Descriptor, DescriptorType, Input},
    },
    crate::stage::combined_stages_flags,
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
        let stages =
            combined_stages_flags(input.uniforms.iter().flat_map(|u| u.stages.iter().copied()));

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
    )
}

fn generate_layout_binding(descriptor: &Descriptor, binding: u32) -> TokenStream {
    let desc_ty = match descriptor.desc_ty {
        DescriptorType::Sampler(_) => {
            quote::format_ident!("Sampler")
        }
        DescriptorType::Image(image::Image {
            kind: image::Kind::Sampled,
        }) => {
            quote::format_ident!("SampledImage")
        }
        DescriptorType::Image(image::Image {
            kind: image::Kind::Storage,
        }) => {
            quote::format_ident!("StorageImage")
        }
        DescriptorType::Buffer(buffer::Buffer {
            kind: buffer::Kind::Uniform,
            texel: false,
        }) => {
            quote::format_ident!("UniformBuffer")
        }
        DescriptorType::Buffer(buffer::Buffer {
            kind: buffer::Kind::Storage,
            texel: false,
        }) => {
            quote::format_ident!("StorageTexelBuffer")
        }
        DescriptorType::Buffer(buffer::Buffer {
            kind: buffer::Kind::Uniform,
            texel: true,
        }) => {
            quote::format_ident!("UniformTexelBuffer")
        }
        DescriptorType::Buffer(buffer::Buffer {
            kind: buffer::Kind::Storage,
            texel: true,
        }) => {
            quote::format_ident!("StorageBuffer")
        }
        DescriptorType::AccelerationStructure(_) => {
            quote::format_ident!("AccelerationStructure")
        }
    };

    let stages = combined_stages_flags(descriptor.stages.iter().copied());
    let flags = combined_binding_flags(descriptor.flags.iter().copied());

    let ty = &descriptor.field.ty;

    quote::quote!(
        ::sierra::DescriptorSetLayoutBinding {
            binding: #binding,
            ty: ::sierra::DescriptorType::#desc_ty,
            count: <#ty as ::sierra::TypedDescriptorBinding>::COUNT,
            stages: ::sierra::ShaderStageFlags::from_bits_truncate(#stages),
            flags: ::sierra::DescriptorBindingFlags::from_bits_truncate(#flags),
        }
    )
}
