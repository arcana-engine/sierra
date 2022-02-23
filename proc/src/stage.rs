use crate::{kw, take_attributes};

use proc_macro2::TokenStream;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stage {
    Vertex,
    TessellationControl,
    TessellationEvaluation,
    Geometry,
    Fragment,
    Compute,
    Raygen,
    AnyHit,
    ClosestHit,
    Miss,
    Intersection,
}

impl Stage {
    pub fn flag(&self) -> u32 {
        match self {
            Stage::Vertex => 0b0000000000001,
            Stage::TessellationControl => 0b0000000000010,
            Stage::TessellationEvaluation => 0b0000000000100,
            Stage::Geometry => 0b0000000001000,
            Stage::Fragment => 0b0000000010000,
            Stage::Compute => 0b0000000100000,
            Stage::Raygen => 0b0000100000000,
            Stage::AnyHit => 0b0001000000000,
            Stage::ClosestHit => 0b0010000000000,
            Stage::Miss => 0b0100000000000,
            Stage::Intersection => 0b1000000000000,
        }
    }

    fn parse(stream: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead1 = stream.lookahead1();
        if lookahead1.peek(kw::Vertex) {
            stream.parse::<kw::Vertex>()?;
            return Ok(Stage::Vertex);
        }
        if lookahead1.peek(kw::TessellationControl) {
            stream.parse::<kw::TessellationControl>()?;
            return Ok(Stage::TessellationControl);
        }
        if lookahead1.peek(kw::TessellationEvaluation) {
            stream.parse::<kw::TessellationEvaluation>()?;
            return Ok(Stage::TessellationEvaluation);
        }
        if lookahead1.peek(kw::Geometry) {
            stream.parse::<kw::Geometry>()?;
            return Ok(Stage::Geometry);
        }
        if lookahead1.peek(kw::Fragment) {
            stream.parse::<kw::Fragment>()?;
            return Ok(Stage::Fragment);
        }
        if lookahead1.peek(kw::Compute) {
            stream.parse::<kw::Compute>()?;
            return Ok(Stage::Compute);
        }
        if lookahead1.peek(kw::Raygen) {
            stream.parse::<kw::Raygen>()?;
            return Ok(Stage::Raygen);
        }
        if lookahead1.peek(kw::AnyHit) {
            stream.parse::<kw::AnyHit>()?;
            return Ok(Stage::AnyHit);
        }
        if lookahead1.peek(kw::ClosestHit) {
            stream.parse::<kw::ClosestHit>()?;
            return Ok(Stage::ClosestHit);
        }
        if lookahead1.peek(kw::Miss) {
            stream.parse::<kw::Miss>()?;
            return Ok(Stage::Miss);
        }
        if lookahead1.peek(kw::Intersection) {
            stream.parse::<kw::Intersection>()?;
            return Ok(Stage::Intersection);
        }
        Err(lookahead1.error())
    }
}

pub fn combined_stages_flags(stages: impl IntoIterator<Item = Stage>) -> u32 {
    stages
        .into_iter()
        .fold(0, |flags, stage| flags | stage.flag())
}

pub fn take_stages(attributes: &mut Vec<syn::Attribute>) -> syn::Result<Vec<Stage>> {
    let stages = take_attributes(attributes, |attr| match attr.path.get_ident() {
        Some(ident) if ident == "stages" => attr
            .parse_args_with(|stream: syn::parse::ParseStream<'_>| {
                stream.parse_terminated::<_, syn::Token![,]>(Stage::parse)
            })
            .map(Some),
        _ => Ok(None),
    })?
    .into_iter()
    .flatten()
    .collect();

    Ok(stages)
}

pub fn shader_stages(tokens: proc_macro::TokenStream) -> TokenStream {
    let result = syn::parse::Parser::parse(
        |stream: syn::parse::ParseStream| {
            stream.parse_terminated::<_, syn::Token![,]>(|stream: syn::parse::ParseStream| {
                Stage::parse(stream)
            })
        },
        tokens,
    );

    match result {
        Err(err) => err.into_compile_error(),
        Ok(stages) => {
            let flags = combined_stages_flags(stages);
            quote::quote!(#flags)
        }
    }
}
