mod acceleration_structure;
mod buffer;
// mod combined_image_sampler;
mod image;
mod input;
mod instance;
mod layout;
mod parse;
mod sampler;
mod uniform;

use proc_macro2::TokenStream;

use crate::kw;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingFlag {
    UpdateAfterBind,
    PartiallyBound,
    UpdateUnused,
}

impl BindingFlag {
    pub fn flag(&self) -> u32 {
        match self {
            BindingFlag::UpdateAfterBind => 0x00000001,
            BindingFlag::PartiallyBound => 0x00000002,
            BindingFlag::UpdateUnused => 0x00000004,
        }
    }

    fn parse(stream: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead1 = stream.lookahead1();
        if lookahead1.peek(kw::UpdateAfterBind) {
            stream.parse::<kw::UpdateAfterBind>()?;
            Ok(BindingFlag::UpdateAfterBind)
        } else if lookahead1.peek(kw::PartiallyBound) {
            stream.parse::<kw::PartiallyBound>()?;
            Ok(BindingFlag::PartiallyBound)
        } else if lookahead1.peek(kw::UpdateUnused) {
            stream.parse::<kw::UpdateUnused>()?;
            Ok(BindingFlag::UpdateUnused)
        } else {
            Err(lookahead1.error())
        }
    }
}

fn combined_binding_flags(flags: impl IntoIterator<Item = BindingFlag>) -> u32 {
    flags
        .into_iter()
        .fold(0, |flags, binding| flags | binding.flag())
}

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

pub fn binding_flags(tokens: proc_macro::TokenStream) -> TokenStream {
    let result = syn::parse::Parser::parse(
        |stream: syn::parse::ParseStream| {
            stream.parse_terminated::<_, syn::Token![,]>(|stream: syn::parse::ParseStream| {
                BindingFlag::parse(stream)
            })
        },
        tokens,
    );

    match result {
        Err(err) => err.into_compile_error(),
        Ok(flags) => {
            let flags = combined_binding_flags(flags);
            quote::quote!(#flags)
        }
    }
}
