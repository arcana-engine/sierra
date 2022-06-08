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
    match try_descriptors(item) {
        Ok(output) => output,
        Err(err) => err.into_compile_error(),
    }
}

fn try_descriptors(item: proc_macro::TokenStream) -> Result<proc_macro2::TokenStream, syn::Error> {
    let input = parse::parse(item)?;
    let tokens = std::iter::once(input::generate(&input))
        .chain(Some(instance::generate(&input)?))
        .chain(Some(layout::generate(&input)?))
        // .chain(Some(generate_glsl_shader_input(&input)))
        .collect::<proc_macro2::TokenStream>();
    Ok(tokens)
}
