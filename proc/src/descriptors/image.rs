use proc_macro2::TokenStream;
use syn::spanned::Spanned;

use crate::kw;

impl Image {
    #[inline]
    pub fn validate(&self, _item_struct: &syn::ItemStruct) -> syn::Result<()> {
        Ok(())
    }
}

proc_easy::easy_argument_group! {
    #[derive(Clone, Copy)]
    pub enum Kind {
        Sampled(kw::sampled),
        Storage(kw::storage),
    }
}

proc_easy::easy_parse! {
    #[derive(Clone)]
    pub struct ConstLayout {
        pub kw: syn::Token![const],
        pub ty: syn::Type,
    }
}

proc_easy::easy_parse! {
    #[derive(Clone)]
    pub enum LayoutValue {
        Const(ConstLayout),
        Dynamic(syn::Token![dyn]),
    }
}

proc_easy::easy_argument_value! {
    #[derive(Clone)]
    pub struct Layout {
        pub kw: kw::layout,
        pub value: LayoutValue,
    }
}

impl Layout {
    pub fn to_tokens(&self) -> TokenStream {
        match &self.value {
            LayoutValue::Const(layout) => {
                let ty = &layout.ty;
                quote::quote!(#ty)
            }
            LayoutValue::Dynamic(token) => {
                quote::quote_spanned!(token.span() => ::sierra::DynamicLayout)
            }
        }
    }

    pub fn to_tokens_opt(opt: Option<&Self>, default: impl FnOnce() -> TokenStream) -> TokenStream {
        match opt {
            None => default(),
            Some(layout) => layout.to_tokens(),
        }
    }
}

proc_easy::easy_argument_tuple! {
    #[derive(Clone)]
    pub struct Image {
        pub kw: kw::image,
        pub kind: Option<Kind>,
        pub layout: Option<Layout>,
    }
}
