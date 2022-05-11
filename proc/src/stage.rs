use proc_macro2::TokenStream;
use syn::{parse::Parse, punctuated::Punctuated};

proc_easy::easy_flags! {
    pub Stage(stage) |
    pub Stages(stages) {
        Vertex(vertex),
        TessellationControl(tesselation_control),
        TessellationEvaluation(tessellation_evaluation),
        Geometry(geometry),
        Fragment(fragment),
        Compute(compute),
        Raygen(raygen),
        AnyHit(any_hit),
        ClosestHit(closest_hit),
        Miss(miss),
        Intersection(intersection),
    }
}

impl Stage {
    pub fn bit(&self) -> u32 {
        match self {
            Stage::Vertex(_) => 0b0000000000001,
            Stage::TessellationControl(_) => 0b0000000000010,
            Stage::TessellationEvaluation(_) => 0b0000000000100,
            Stage::Geometry(_) => 0b0000000001000,
            Stage::Fragment(_) => 0b0000000010000,
            Stage::Compute(_) => 0b0000000100000,
            Stage::Raygen(_) => 0b0000100000000,
            Stage::AnyHit(_) => 0b0001000000000,
            Stage::ClosestHit(_) => 0b0010000000000,
            Stage::Miss(_) => 0b0100000000000,
            Stage::Intersection(_) => 0b1000000000000,
        }
    }
}

pub fn combined_stages_flags<'a>(stages: impl Iterator<Item = Stage>) -> u32 {
    stages.fold(0, |flags, stage| flags | stage.bit())
}

impl Stages {
    pub fn bits(&self) -> u32 {
        combined_stages_flags(self.flags.iter().copied())
    }
}

pub fn parse_stages(
    stream: syn::parse::ParseStream,
) -> syn::Result<Punctuated<Stage, syn::Token![,]>> {
    stream.parse_terminated::<_, syn::Token![,]>(Stage::parse)
}

pub fn shader_stages(tokens: proc_macro::TokenStream) -> TokenStream {
    let result = syn::parse::Parser::parse(parse_stages, tokens);

    match result {
        Err(err) => err.into_compile_error(),
        Ok(stages) => {
            let flags = combined_stages_flags(stages.iter().copied());
            quote::quote!(#flags)
        }
    }
}
