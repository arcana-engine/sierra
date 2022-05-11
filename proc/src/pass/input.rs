use {super::parse::Input, proc_macro2::TokenStream};

pub(super) fn generate(input: &Input) -> TokenStream {
    let ident = &input.item_struct.ident;
    let instance = quote::format_ident!("{}Instance", input.item_struct.ident);

    quote::quote!(
        impl #ident {
            pub fn instance() -> #instance {
                #instance::new()
            }
        }

        impl ::sierra::TypedRenderPass for #ident {
            type Instance = #instance;

            fn instance() -> #instance {
                #instance::new()
            }
        }
    )
}
