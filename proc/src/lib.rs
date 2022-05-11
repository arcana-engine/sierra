use std::convert::TryFrom as _;

extern crate proc_macro;

mod descriptors;
mod flags;
mod graphics_pipeline;
mod layout;
mod pass;
mod pipeline;
mod repr;
mod stage;
mod swizzle;

#[proc_macro_derive(TypedDescriptors, attributes(sierra))]
pub fn typed_descriptors(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    descriptors::descriptors(item).into()
}

#[proc_macro_derive(ShaderRepr, attributes(sierra))]
pub fn shader_repr(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    repr::shader_repr(item).into()
}

#[proc_macro_derive(TypedPipeline, attributes(sierra))]
pub fn typed_pipeline(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    pipeline::pipeline(item).into()
}

#[proc_macro_derive(TypedRenderPass, attributes(sierra))]
pub fn typed_render_pass(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    pass::pass(item).into()
}

#[proc_macro]
pub fn graphics_pipeline_desc(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    graphics_pipeline::graphics_pipeline_desc(item).into()
}

#[proc_macro]
pub fn shader_stages(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    stage::shader_stages(tokens).into()
}

#[proc_macro]
pub fn binding_flags(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    flags::binding_flags(tokens).into()
}

#[proc_macro]
pub fn swizzle(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    swizzle::swizzle(item).into()
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
}
