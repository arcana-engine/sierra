mod acceleration_structure;
mod buffer;
mod image;
mod input;
mod instance;
mod layout;
mod parse;
mod sampler;
mod uniform;

pub fn descriptors(item: proc_macro::TokenStream) -> proc_macro2::TokenStream {
    match parse::parse(item) {
        Ok(input) => {
            std::iter::once(input::generate(&input))
                .chain(Some(instance::generate(&input)))
                .chain(Some(layout::generate(&input)))
                // .chain(Some(generate_glsl_shader_input(&input)))
                .collect::<proc_macro2::TokenStream>()
        }
        Err(err) => err.into_compile_error(),
    }
}
