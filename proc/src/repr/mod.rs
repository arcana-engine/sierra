mod generate;
mod parse;

use proc_macro2::TokenStream;

pub fn shader_repr(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> TokenStream {
    match parse::parse(attr, item) {
        Ok(input) => {
            let struct_item = &input.item_struct;
            std::iter::once(quote::quote!(#struct_item))
                .chain(Some(generate::generate_repr(&input)))
                // .chain(Some(generate_glsl_type(&input)))
                .collect::<TokenStream>()
        }
        Err(err) => err.into_compile_error(),
    }
}
