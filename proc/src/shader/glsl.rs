use {
    crate::descriptors::{
        buffer,
        parse::{FieldType, Input},
    },
    proc_macro2::TokenStream,
    sierra_types::Stage,
    std::fmt::Write as _,
    syn::spanned::Spanned as _,
};

pub fn generate_glsl_shader_input(shader_input: &Input) -> TokenStream {
    let vertex = code_for_stage(shader_input, Stage::Vertex);
    let tessellation_control = code_for_stage(shader_input, Stage::TessellationControl);
    let tessellation_evaluation = code_for_stage(shader_input, Stage::TessellationEvaluation);
    let geometry = code_for_stage(shader_input, Stage::Geometry);
    let fragment = code_for_stage(shader_input, Stage::Fragment);
    let compute = code_for_stage(shader_input, Stage::Compute);
    let raygen = code_for_stage(shader_input, Stage::Raygen);
    let any_hit = code_for_stage(shader_input, Stage::AnyHit);
    let closest_hit = code_for_stage(shader_input, Stage::ClosestHit);
    let miss = code_for_stage(shader_input, Stage::Miss);
    let intersection = code_for_stage(shader_input, Stage::Intersection);

    let shader_ident = &shader_input.ident;

    quote::quote!(
        impl ::sierra::glsl::ShaderInputDecl for #shader_ident {
            fn glsl(stage: ::sierra::Stage) -> String {
                match stage {
                    ::sierra::Stage::Vertex => #vertex,
                    ::sierra::Stage::TessellationControl => #tessellation_control,
                    ::sierra::Stage::TessellationEvaluation => #tessellation_evaluation,
                    ::sierra::Stage::Geometry => #geometry,
                    ::sierra::Stage::Fragment => #fragment,
                    ::sierra::Stage::Compute => #compute,
                    ::sierra::Stage::Raygen => #raygen,
                    ::sierra::Stage::AnyHit => #any_hit,
                    ::sierra::Stage::ClosestHit => #closest_hit,
                    ::sierra::Stage::Miss => #miss,
                    ::sierra::Stage::Intersection => #intersection,
                }
            }
        }
    )
}

fn code_for_stage(shader_input: &ShaderInput, stage: Stage) -> TokenStream {
    let mut deps = Vec::new();
    let mut args = Vec::new();

    let code = shader_input
        .fields
        .iter()
        .filter(|input| input.stages.contains(&stage))
        .map(|input| {
            let descriptor_field = quote::format_ident!("descriptor_{}", input.member);

            match &input.ty {
                InputFieldType::CombinedImageSampler(_) => format!(
                    "layout(binding = {}, set = 0) uniform sampler2D {};\n",
                    input.binding, descriptor_field
                ),
                InputFieldType::AccelerationStructure(_) => format!(
                    "layout(binding = {}, set = 0) uniform accelerationStructureEXT {};\n",
                    input.binding, descriptor_field
                ),
                InputFieldType::Buffer(buffer::Buffer {
                    kind: buffer::Kind::Uniform,
                    ty,
                }) => {
                    deps.push(quote::quote!(<#ty as ::sierra::glsl::GlslType>::deps(&mut ctx)));
                    args.push(quote::quote!(<#ty as ::sierra::glsl::GlslType>::def()));
                    format!(
                        "layout(binding = {}, set = 0) uniform {{}} {};\n",
                        input.binding, descriptor_field
                    )
                }
                InputFieldType::Buffer(buffer::Buffer {
                    kind: buffer::Kind::Storage,
                    ty,
                }) => {
                    deps.push(quote::quote!(<#ty as ::sierra::glsl::GlslType>::deps(&mut ctx)));
                    args.push(quote::quote!(<#ty as ::sierra::glsl::GlslType>::def()));
                    format!(
                        "layout(binding = {}, set = 0) buffer {{}} {};\n",
                        input.binding, descriptor_field
                    )
                }
            }
        })
        .collect::<String>();

    if code.is_empty() {
        assert!(args.is_empty());
        quote::quote!(String::new())
    } else {
        let lit = syn::LitStr::new(&code, shader_input.input_struct.span());
        if args.is_empty() {
            quote::quote!(#lit.to_owned())
        } else {
            quote::quote!(
                let mut ctx = ::sierra::glsl::GlslTypeContext::new();
                #(#deps;)*
                let mut code = ctx.code();
                std::write!(&mut code, #lit, #(#args),*).unwrap();
                code
            )
        }
    }
}

pub fn generate_glsl_type(input: &ShaderStructInput) -> TokenStream {
    let ident = &input.input_struct.ident;
    let ident_string = ident.to_string();
    let name = syn::LitStr::new(&ident_string, ident.span());

    let original_field_types = input.input_struct.fields.iter().map(|field| &field.ty);
    let field_types = input.fields.iter().map(|field| &field.as_type);
    let field_types2 = field_types.clone();
    let field_types3 = field_types.clone();
    let def_format_string = {
        let mut def = ident_string;
        std::write!(&mut def, "{{{{\n").unwrap();

        for field in &input.input_struct.fields {
            std::write!(
                &mut def,
                "    {{}} {}{{}};\n",
                *field
                    .ident
                    .as_ref()
                    .expect("Tuple struct are not supported")
            )
            .unwrap();
        }
        std::write!(&mut def, "}}}}").unwrap();
        def
    };

    quote::quote!(
        impl ::sierra::glsl::GlslType for #ident {
            fn name() -> &'static str {
                #name
            }

            fn deps(ctx: &mut ::sierra::glsl::GlslTypeContext) {
                use ::sierra::types::*;

                #(ctx.add::<#field_types, #original_field_types>();)*
            }

            fn def() -> String {
                use ::sierra::types::*;

                format!(
                    #def_format_string,
                    #(
                        <#field_types2 as ::sierra::glsl::GlslType>::name(),
                        <#field_types3 as ::sierra::glsl::GlslType>::suffix(),
                    )*
                )
            }
        }
    )
}
