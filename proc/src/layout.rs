use syn::spanned::Spanned;

proc_easy::easy_flags! {
    pub StructLayout(layout) |
    pub StructLayouts(layouts) {
        Std140(std140),
        Std430(std430),
    }
}

impl Default for StructLayout {
    fn default() -> Self {
        StructLayout::Std140(Default::default())
    }
}

impl StructLayout {
    pub fn name(&self) -> &'static str {
        match self {
            StructLayout::Std140(_) => "Std140",
            StructLayout::Std430(_) => "Std430",
        }
    }

    pub fn default_sierra_type() -> proc_macro2::TokenStream {
        quote::quote!(::sierra::Std140)
    }

    pub fn sierra_type(&self) -> proc_macro2::TokenStream {
        match self {
            StructLayout::Std140(kw) => quote::quote_spanned!(kw.span() => ::sierra::Std140),
            StructLayout::Std430(kw) => quote::quote_spanned!(kw.span() => ::sierra::Std430),
        }
    }
}
