use std::convert::TryFrom as _;

extern crate proc_macro;

mod binding_flags;
mod descriptors;
mod format;
mod graphics_pipeline;
mod layout;
mod pass;
mod pipeline;
mod pipeline_stages;
mod repr;
mod shader_stage;
mod swizzle;

#[proc_macro_derive(Descriptors, attributes(sierra))]
pub fn descriptors(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    descriptors::descriptors(item).into()
}

#[proc_macro_derive(ShaderRepr, attributes(sierra))]
pub fn shader_repr(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    repr::shader_repr(item).into()
}

#[proc_macro_derive(PipelineInput, attributes(sierra))]
pub fn pipeline_input(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    pipeline::pipeline_input(item).into()
}

#[proc_macro_derive(Pass, attributes(sierra))]
pub fn render_pass(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    pass::pass(item).into()
}

#[proc_macro]
pub fn graphics_pipeline_desc(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    graphics_pipeline::graphics_pipeline_desc(item).into()
}

#[proc_macro]
pub fn shader_stages(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    shader_stage::shader_stages(tokens).into()
}

#[proc_macro]
pub fn binding_flags(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    binding_flags::binding_flags(tokens).into()
}

#[proc_macro]
pub fn pipeline_stages(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    pipeline_stages::pipeline_stages(tokens).into()
}

#[proc_macro]
pub fn swizzle(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    swizzle::swizzle(item).into()
}

#[proc_macro]
pub fn format(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    format::format(item).into()
}

fn validate_member(member: &syn::Member, item_struct: &syn::ItemStruct) -> syn::Result<u32> {
    match (member, &item_struct.fields) {
        (syn::Member::Named(member_ident), syn::Fields::Named(fields)) => {
            for (index, field) in fields.named.iter().enumerate() {
                let field_ident = field.ident.as_ref().unwrap();
                if *field_ident == *member_ident {
                    return u32::try_from(index)
                        .map_err(|_| syn::Error::new_spanned(member, "Too many fields"));
                }
            }
            Err(syn::Error::new_spanned(
                member,
                "Member not found in structure",
            ))
        }
        (syn::Member::Unnamed(unnamed), syn::Fields::Unnamed(fields)) => {
            let valid =
                usize::try_from(unnamed.index).map_or(false, |index| index < fields.unnamed.len());
            if !valid {
                Err(syn::Error::new_spanned(
                    member,
                    "Member index is out of bounds",
                ))
            } else {
                Ok(unnamed.index)
            }
        }
        (syn::Member::Named(named), syn::Fields::Unnamed(_)) => Err(syn::Error::new_spanned(
            named,
            "Unexpected unnamed member for tuple-struct",
        )),
        (syn::Member::Unnamed(unnamed), syn::Fields::Named(_)) => Err(syn::Error::new_spanned(
            unnamed,
            "Unexpected named member for struct",
        )),
        (member, syn::Fields::Unit) => Err(syn::Error::new_spanned(
            member,
            "Unexpected member reference for unit-struct",
        )),
    }
}

// fn parse_attrs_with<T>(
//     attrs: &[syn::Attribute],
//     mut f: fn(syn::parse::ParseStream) -> syn::Result<T>,
// ) -> syn::Result<Punctuated<T, syn::Token![,]>> {
//     let mut result = Punctuated::new();

//     for attr in attrs {
//         if attr.path.is_ident("sierra") {
//             let array = attr.parse_args_with(|stream: syn::parse::ParseStream| {
//                 stream.parse_terminated::<_, syn::Token![,]>(f)
//             })?;
//             result.extend(array.into_pairs());
//         }
//     }

//     Ok(result)
// }

mod kw {
    proc_easy::easy_token!(acceleration_structure);
    proc_easy::easy_token!(buffer);
    proc_easy::easy_token!(image);
    proc_easy::easy_token!(sampled);
    proc_easy::easy_token!(sampler);
    proc_easy::easy_token!(uniform);
    proc_easy::easy_token!(storage);
    proc_easy::easy_token!(texel);
    proc_easy::easy_token!(subpass);
    proc_easy::easy_token!(color);
    proc_easy::easy_token!(depth);
    proc_easy::easy_token!(clear);
    proc_easy::easy_token!(load);
    proc_easy::easy_token!(store);
    proc_easy::easy_token!(capacity);
    proc_easy::easy_token!(set);
    proc_easy::easy_token!(push);
    proc_easy::easy_token!(layout);
    proc_easy::easy_token!(attachment);
    proc_easy::easy_token!(top_of_pipe);
    proc_easy::easy_token!(draw_indirect);
    proc_easy::easy_token!(vertex_input);
    proc_easy::easy_token!(vertex_shader);
    proc_easy::easy_token!(tessellation_control_shader);
    proc_easy::easy_token!(tessellation_evaluation_shader);
    proc_easy::easy_token!(geometry_shader);
    proc_easy::easy_token!(early_fragment_tests);
    proc_easy::easy_token!(fragment_shader);
    proc_easy::easy_token!(late_fragment_tests);
    proc_easy::easy_token!(color_attachment_output);
    proc_easy::easy_token!(compute_shader);
    proc_easy::easy_token!(transfer);
    proc_easy::easy_token!(bottom_of_pipe);
    proc_easy::easy_token!(host);
    proc_easy::easy_token!(all_graphics);
    proc_easy::easy_token!(all_commands);
    proc_easy::easy_token!(ray_tracing_shader);
    proc_easy::easy_token!(acceleration_structure_build);
    proc_easy::easy_token!(dependency);
    proc_easy::easy_token!(external);
}
