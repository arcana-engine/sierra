extern crate proc_macro;

macro_rules! on_first_ok {
    ($e:expr) => {
        if let Some(r) = $e {
            return Ok(Some(r));
        }
    };
}

mod descriptors;
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

fn get_unique_attribute<T>(
    attrs: &mut Vec<syn::Attribute>,
    mut f: impl FnMut(&syn::Attribute) -> syn::Result<Option<T>>,
    msg: impl std::fmt::Display,
) -> syn::Result<T> {
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
            Ok(v)
        }
        None => Err(syn::Error::new(proc_macro2::Span::call_site(), msg)),
    }
}

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
