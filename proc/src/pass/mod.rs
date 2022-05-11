mod input;
mod instance;
mod parse;

pub fn pass(item: proc_macro::TokenStream) -> proc_macro2::TokenStream {
    match parse::parse(item) {
        Ok(input) => std::iter::once(input::generate(&input))
            .chain(Some(instance::generate(&input)))
            .collect::<proc_macro2::TokenStream>(),
        Err(err) => err.into_compile_error(),
    }
}
