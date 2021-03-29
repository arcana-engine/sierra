use {proc_macro2::TokenStream, quote::TokenStreamExt as _, std::collections::HashSet};

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

pub fn stage_flag_tokens(stage: Stage) -> TokenStream {
    match stage {
        Stage::Vertex => quote::quote!(::sierra::ShaderStageFlags::VERTEX),
        Stage::TessellationControl => {
            quote::quote!(::sierra::ShaderStageFlags::TESSELLATION_CONTROL)
        }
        Stage::TessellationEvaluation => {
            quote::quote!(::sierra::ShaderStageFlags::TESSELLATION_EVALUATION)
        }
        Stage::Geometry => {
            quote::quote!(::sierra::ShaderStageFlags::GEOMETRY)
        }
        Stage::Fragment => {
            quote::quote!(::sierra::ShaderStageFlags::FRAGMENT)
        }
        Stage::Compute => {
            quote::quote!(::sierra::ShaderStageFlags::COMPUTE)
        }
        Stage::Raygen => quote::quote!(::sierra::ShaderStageFlags::RAYGEN),
        Stage::AnyHit => quote::quote!(::sierra::ShaderStageFlags::ANY_HIT),
        Stage::ClosestHit => {
            quote::quote!(::sierra::ShaderStageFlags::CLOSEST_HIT)
        }
        Stage::Miss => quote::quote!(::sierra::ShaderStageFlags::MISS),
        Stage::Intersection => {
            quote::quote!(::sierra::ShaderStageFlags::INTERSECTION)
        }
    }
}

pub fn combined_stages_tokens(stages: impl IntoIterator<Item = Stage>) -> TokenStream {
    let mut stages = stages.into_iter();
    if let Some(head) = stages.next() {
        let or = proc_macro2::Punct::new('|', proc_macro2::Spacing::Alone);
        let mut stream = stage_flag_tokens(head);
        for stage in stages {
            stream.append(or.clone());
            stream.extend(stage_flag_tokens(stage));
        }
        stream
    } else {
        quote::quote!(::sierra::ShaderStageFlags::empty())
    }
}

pub fn combined_stages_tokens_dedup(stages: impl IntoIterator<Item = Stage>) -> TokenStream {
    let stages = stages.into_iter().collect::<HashSet<_>>();
    combined_stages_tokens(stages)
}
