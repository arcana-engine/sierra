use std::convert::TryFrom as _;

extern crate proc_macro;

macro_rules! on_first_ok {
    ($e:expr) => {
        if let Some(r) = $e {
            return Ok(Some(r));
        }
    };
}

mod descriptors;
mod graphics_pipeline;
mod pass;
mod pipeline;
mod repr;
mod stage;

#[proc_macro_attribute]
pub fn descriptors(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    descriptors::descriptors(attr, item).into()
}

#[proc_macro_attribute]
pub fn shader_repr(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    repr::shader_repr(attr, item).into()
}

#[proc_macro_attribute]
pub fn pipeline(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    pipeline::pipeline(attr, item).into()
}

#[proc_macro_attribute]
pub fn pass(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    pass::pass(attr, item).into()
}

#[proc_macro]
pub fn graphics_pipeline_desc(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    graphics_pipeline::graphics_pipeline_desc(item).into()
}

fn take_attributes<T>(
    attrs: &mut Vec<syn::Attribute>,
    mut f: impl FnMut(&syn::Attribute) -> syn::Result<Option<T>>,
) -> syn::Result<Vec<T>> {
    let len = attrs.len();
    let mut del = 0;
    let mut result = Vec::with_capacity(len);
    {
        let v = &mut **attrs;

        for i in 0..len {
            if let Some(value) = f(&v[i])? {
                result.push(value);
                del += 1;
            } else if del > 0 {
                v.swap(i - del, i);
            }
        }
    }
    if del > 0 {
        attrs.truncate(len - del);
    }

    Ok(result)
}

fn find_unique_attribute<T>(
    attrs: &mut Vec<syn::Attribute>,
    mut f: impl FnMut(&syn::Attribute) -> syn::Result<Option<T>>,
    msg: impl std::fmt::Display,
) -> syn::Result<Option<T>> {
    let mut found = None;

    for (index, attr) in attrs.iter().enumerate() {
        if let Some(v) = f(attr)? {
            if found.is_none() {
                found = Some((index, v));
            } else {
                return Err(syn::Error::new_spanned(attr, msg));
            }
        }
    }

    match found {
        Some((i, v)) => {
            attrs.remove(i);
            Ok(Some(v))
        }
        None => Ok(None),
    }
}

// fn get_unique_attribute<T>(
//     attrs: &mut Vec<syn::Attribute>,
//     mut f: impl FnMut(&syn::Attribute) -> syn::Result<Option<T>>,
//     spanned: &impl quote::ToTokens,
//     msg: impl std::fmt::Display,
// ) -> syn::Result<T> {
//     let mut found = None;
//     for (index, attr) in attrs.iter().enumerate() {
//         if let Some(v) = f(attr)? {
//             if found.is_none() {
//                 found = Some((index, v));
//             } else {
//                 return Err(syn::Error::new_spanned(attr, msg));
//             }
//         }
//     }

//     match found {
//         Some((i, v)) => {
//             attrs.remove(i);
//             Ok(v)
//         }
//         None => Err(syn::Error::new_spanned(spanned, msg)),
//     }
// }

fn find_unique<I>(
    iter: I,
    spanned: &impl quote::ToTokens,
    msg: impl std::fmt::Display,
) -> syn::Result<Option<I::Item>>
where
    I: IntoIterator,
{
    let mut iter = iter.into_iter();
    match iter.next() {
        None => Ok(None),
        Some(item) => {
            if iter.next().is_none() {
                Ok(Some(item))
            } else {
                Err(syn::Error::new_spanned(spanned, msg))
            }
        }
    }
}

fn get_unique<I>(
    iter: I,
    spanned: &impl quote::ToTokens,
    msg: impl std::fmt::Display,
) -> syn::Result<I::Item>
where
    I: IntoIterator,
{
    let mut iter = iter.into_iter();
    if let Some(item) = iter.next() {
        if iter.next().is_none() {
            return Ok(item);
        }
    }
    Err(syn::Error::new_spanned(spanned, msg))
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
            "Expected unnamed member for tuple-struct",
        )),
        (syn::Member::Unnamed(unnamed), syn::Fields::Named(_)) => Err(syn::Error::new_spanned(
            unnamed,
            "Expected named member for struct",
        )),
        (member, syn::Fields::Unit) => Err(syn::Error::new_spanned(
            member,
            "Unexpected member reference for unit-struct",
        )),
    }
}
