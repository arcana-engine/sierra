use {super::parse::Input, proc_macro2::TokenStream};

pub(super) fn generate(input: &Input) -> TokenStream {
    let ident = &input.item_struct.ident;

    quote::quote!(
        impl #ident {}
    )
}
