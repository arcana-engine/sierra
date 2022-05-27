mod input;
// mod instance;
mod layout;
mod parse;

pub fn pipeline_input(item: proc_macro::TokenStream) -> proc_macro2::TokenStream {
    match parse::parse(item) {
        Ok(input) => {
            std::iter::once(input::generate(&input))
                // .chain(Some(instance::generate(&input)))
                .chain(Some(layout::generate(&input)))
                .collect::<proc_macro2::TokenStream>()
        }
        Err(err) => err.into_compile_error(),
    }
}
