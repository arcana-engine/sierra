use proc_macro2::TokenStream;
use syn::{parse::Parse, punctuated::Punctuated};

proc_easy::easy_flags! {
    pub ShaderStage(stage) |
    pub ShaderStages(stages) {
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

impl ShaderStage {
    pub fn bit(&self) -> u32 {
        match self {
            ShaderStage::Vertex(_) => 0b0000000000001,
            ShaderStage::TessellationControl(_) => 0b0000000000010,
            ShaderStage::TessellationEvaluation(_) => 0b0000000000100,
            ShaderStage::Geometry(_) => 0b0000000001000,
            ShaderStage::Fragment(_) => 0b0000000010000,
            ShaderStage::Compute(_) => 0b0000000100000,
            ShaderStage::Raygen(_) => 0b0000100000000,
            ShaderStage::AnyHit(_) => 0b0001000000000,
            ShaderStage::ClosestHit(_) => 0b0010000000000,
            ShaderStage::Miss(_) => 0b0100000000000,
            ShaderStage::Intersection(_) => 0b1000000000000,
        }
    }
}

pub fn combined_shader_stage_flags(stages: impl Iterator<Item = ShaderStage>) -> u32 {
    stages.fold(0, |flags, stage| flags | stage.bit())
}

impl ShaderStages {
    pub fn bits(&self) -> u32 {
        combined_shader_stage_flags(self.flags.iter().copied())
    }
}

pub fn parse_shader_stages(
    stream: syn::parse::ParseStream,
) -> syn::Result<Punctuated<ShaderStage, syn::Token![,]>> {
    stream.parse_terminated::<_, syn::Token![,]>(ShaderStage::parse)
}

pub fn shader_stages(tokens: proc_macro::TokenStream) -> TokenStream {
    let result = syn::parse::Parser::parse(parse_shader_stages, tokens);

    match result {
        Err(err) => err.into_compile_error(),
        Ok(stages) => {
            let flags = combined_shader_stage_flags(stages.iter().copied());
            quote::quote!(#flags)
        }
    }
}
