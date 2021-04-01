mod acceleration_structure;
mod buffer;
mod combined_image_sampler;
mod input;
mod instance;
mod layout;
mod parse;
mod uniform;

pub fn descriptors(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro2::TokenStream {
    match parse::parse(attr, item) {
        Ok(input) => {
            let item_struct = &input.item_struct;
            std::iter::once(quote::quote!(#item_struct))
                .chain(Some(input::generate(&input)))
                .chain(Some(instance::generate(&input)))
                .chain(Some(layout::generate(&input)))
                // .chain(Some(generate_glsl_shader_input(&input)))
                .collect::<proc_macro2::TokenStream>()
        }
        Err(err) => err.into_compile_error(),
    }
}
