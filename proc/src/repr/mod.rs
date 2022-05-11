mod generate;
mod parse;

use proc_macro2::TokenStream;

pub fn shader_repr(item: proc_macro::TokenStream) -> TokenStream {
    match parse::parse(item) {
        Ok(input) => {
            std::iter::once(generate::generate_repr(&input))
                // .chain(Some(generate_glsl_type(&input)))
                .collect::<TokenStream>()
        }
        Err(err) => err.into_compile_error(),
    }
}
