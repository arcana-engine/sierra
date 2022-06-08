use proc_macro2::TokenStream;
use syn::spanned::Spanned;

use crate::{format::parse_format, kw};

impl Buffer {
    #[inline]
    pub fn validate(&self, _item_struct: &syn::ItemStruct) -> syn::Result<()> {
        Ok(())
    }
}

proc_easy::easy_argument_group! {
    #[derive(Clone, Copy)]
    pub enum Kind {
        Uniform(kw::uniform),
        Storage(kw::storage),
    }
}

proc_easy::easy_parse! {
    #[derive(Clone)]
    pub struct ConstFormat {
        pub kw: syn::Token![const],
        pub ty: syn::Type,
    }
}

proc_easy::easy_parse! {
    #[derive(Clone)]
    pub enum FormatValue {
        ! Const(syn::Ident),
        Dynamic(syn::Token![dyn]),
    }
}

impl Default for FormatValue {
    fn default() -> Self {
        FormatValue::Dynamic(syn::Token![dyn](proc_macro2::Span::call_site()))
    }
}

proc_easy::easy_argument_value! {
    #[derive(Clone)]
    pub struct Texel {
        pub kw: kw::texel,
        ? pub format: FormatValue,
    }
}

impl FormatValue {
    pub fn to_tokens(&self) -> Result<TokenStream, syn::Error> {
        match self {
            FormatValue::Dynamic(token) => {
                Ok(quote::quote_spanned!(token.span() => ::sierra::DynamicFormat))
            }
            FormatValue::Const(format) => parse_format(&*format.to_string()),
        }
    }
}

proc_easy::easy_argument_tuple! {
    #[derive(Clone)]
    pub struct Buffer {
        pub kw: kw::buffer,
        pub kind: Option<Kind>,
        pub texel: Option<Texel>,
    }
}
