mod input;
mod instance;
mod layout;
mod parse;

pub fn pipeline(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro2::TokenStream {
    let input = parse::parse(attr, item);

    let item_struct = &input.item_struct;
    std::iter::once(quote::quote!(#item_struct))
        .chain(Some(input::generate(&input)))
        .chain(Some(instance::generate(&input)))
        .chain(Some(layout::generate(&input)))
        .collect::<proc_macro2::TokenStream>()
}
