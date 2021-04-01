mod input;
mod parse;

pub fn pass(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro2::TokenStream {
    match parse::parse(attr, item) {
        Ok(input) => {
            // input.item_struct.attrs.extend(
            //     syn::parse::Parser::parse2(
            //         syn::Attribute::parse_outer,
            //         quote::quote!(#[allow(dead_code)]),
            //     )
            //     .unwrap(),
            // );

            let item_struct = &input.item_struct;
            std::iter::once(quote::quote!(#item_struct))
                // .chain(Some(input::generate(&input)))
                // .chain(Some(instance::generate(&input)))
                // .chain(Some(layout::generate(&input)))
                .collect::<proc_macro2::TokenStream>()
        }
        Err(err) => err.into_compile_error(),
    }
}
