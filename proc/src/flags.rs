use proc_easy::private::Punctuated;
use proc_macro2::TokenStream;
use syn::parse::Parse;

proc_easy::easy_flags! {
    pub BindingFlag(flag) | pub BindingFlags(flags) {
        UpdateAfterBind(update_after_bind),
        PartiallyBound(partially_bound),
        UpdateUnused(update_unused),
    }
}

impl BindingFlag {
    pub fn bit(&self) -> u32 {
        match self {
            BindingFlag::UpdateAfterBind(_) => 0x00000001,
            BindingFlag::PartiallyBound(_) => 0x00000002,
            BindingFlag::UpdateUnused(_) => 0x00000004,
        }
    }
}

impl BindingFlags {
    pub fn bits(&self) -> u32 {
        combined_binding_flags(self.flags.iter().copied())
    }
}

pub fn combined_binding_flags(flags: impl Iterator<Item = BindingFlag>) -> u32 {
    flags.fold(0, |flags, binding| flags | binding.bit())
}

pub fn parse_binding_flags(
    stream: syn::parse::ParseStream,
) -> syn::Result<Punctuated<BindingFlag, syn::Token![,]>> {
    stream.parse_terminated::<_, syn::Token![,]>(BindingFlag::parse)
}

pub fn binding_flags(tokens: proc_macro::TokenStream) -> TokenStream {
    let result = syn::parse::Parser::parse(parse_binding_flags, tokens);

    match result {
        Err(err) => err.into_compile_error(),
        Ok(flags) => {
            let flags = combined_binding_flags(flags.into_iter());
            quote::quote!(#flags)
        }
    }
}
