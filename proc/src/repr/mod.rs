mod parse;
mod repr;

use proc_macro2::TokenStream;

pub fn shader_repr(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> TokenStream {
    let input = parse::parse(attr, item);

    let struct_item = &input.item_struct;
    std::iter::once(quote::quote!(#struct_item))
        .chain(Some(repr::generate_repr(&input)))
        // .chain(Some(generate_glsl_type(&input)))
        .collect::<TokenStream>()
}
