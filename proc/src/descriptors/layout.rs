use {
    super::{
        buffer,
        instance::instance_type_name,
        parse::{Field, FieldType, Input},
    },
    crate::stage::Stage,
    proc_macro2::TokenStream,
};

pub(crate) fn layout_type_name(input: &Input) -> syn::Ident {
    quote::format_ident!("{}Layout", input.item_struct.ident)
}

pub(crate) fn generate(input: &Input) -> TokenStream {
    let layout_ident = layout_type_name(input);
    let instance_ident = instance_type_name(input);

    let bindings = input
        .fields
        .iter()
        .map(|input_field| generate_layout_binding(input_field))
        .collect::<Vec<_>>();

    let vis = &input.item_struct.vis;

    quote::quote!(
        #[derive(Clone, Debug)]
        #[repr(transparent)]
        #vis struct #layout_ident(::sierra::DescriptorSetLayout);

        impl ::sierra::DescriptorsLayout for #layout_ident {
            type Instance = #instance_ident;

            fn new(device: &::sierra::Device) -> Result<Self, ::sierra::OutOfMemory> {
                let layout =
                    device.create_descriptor_set_layout(::sierra::DescriptorSetLayoutInfo {
                        bindings: vec![#(#bindings),*],
                        flags: ::sierra::DescriptorSetLayoutFlags::empty(),
                    })?;

                Ok(#layout_ident(layout))
            }


            fn instantiate(&self) -> #instance_ident {
                #instance_ident::new(self)
            }
        }
    )
    .into()
}

fn generate_layout_binding(input_field: &Field) -> TokenStream {
    let ty = match input_field.ty {
        FieldType::CombinedImageSampler(_) => {
            quote::format_ident!("CombinedImageSampler")
        }
        FieldType::AccelerationStructure(_) => {
            quote::format_ident!("AccelerationStructure")
        }
        FieldType::Buffer(buffer::Buffer {
            kind: buffer::Kind::Uniform,
            ..
        }) => {
            quote::format_ident!("UniformBuffer")
        }
        FieldType::Buffer(buffer::Buffer {
            kind: buffer::Kind::Storage,
            ..
        }) => {
            quote::format_ident!("StorageBuffer")
        }
    };

    let binding = input_field.binding;

    let stages = input_field
        .stages
        .iter()
        .map(|stage| match stage {
            Stage::Vertex => quote::quote!(::sierra::ShaderStageFlags::VERTEX),
            Stage::TessellationControl => {
                quote::quote!(::sierra::ShaderStageFlags::TESSELLATION_CONTROL)
            }
            Stage::TessellationEvaluation => {
                quote::quote!(::sierra::ShaderStageFlags::TESSELLATION_EVALUATION)
            }
            Stage::Geometry => {
                quote::quote!(::sierra::ShaderStageFlags::GEOMETRY)
            }
            Stage::Fragment => {
                quote::quote!(::sierra::ShaderStageFlags::FRAGMENT)
            }
            Stage::Compute => {
                quote::quote!(::sierra::ShaderStageFlags::COMPUTE)
            }
            Stage::Raygen => quote::quote!(::sierra::ShaderStageFlags::RAYGEN),
            Stage::AnyHit => quote::quote!(::sierra::ShaderStageFlags::ANY_HIT),
            Stage::ClosestHit => {
                quote::quote!(::sierra::ShaderStageFlags::CLOSEST_HIT)
            }
            Stage::Miss => quote::quote!(::sierra::ShaderStageFlags::MISS),
            Stage::Intersection => {
                quote::quote!(::sierra::ShaderStageFlags::INTERSECTION)
            }
        })
        .collect::<Vec<_>>();

    quote::quote!(
        ::sierra::DescriptorSetLayoutBinding {
            binding: #binding,
            ty: ::sierra::DescriptorType::#ty,
            count: 1,
            stages: ::sierra::ShaderStageFlags::empty() #(|#stages)*,
            flags: ::sierra::DescriptorBindingFlags::empty(),
        }
    )
}
