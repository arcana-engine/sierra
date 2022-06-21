use proc_macro2::TokenStream;
use syn::{parse::Parse, punctuated::Punctuated};

proc_easy::easy_flags! {
    pub PipelineStage(stage) |
    pub PipelineStages(stages) {
        TopOfPipe(top_of_pipe),
        DrawIndirect(draw_indirect),
        VertexInput(vertex_input),
        VertexShader(vertex_shader),
        TessellationControlShader(tessellation_control_shader),
        TessellationEvaluationShader(tessellation_evaluation_shader),
        GeometryShader(geometry_shader),
        EarlyFragmentTests(early_fragment_tests),
        FragmentShader(fragment_shader),
        LateFragmentTests(late_fragment_tests),
        ColorAttachmentOutput(color_attachment_output),
        ComputeShader(compute_shader),
        Transfer(transfer),
        BottomOfPipe(bottom_of_pipe),
        Host(host),
        AllGraphics(all_graphics),
        AllCommands(all_commands),
        RayTracingShader(ray_tracing_shader),
        AccelerationStructureBuild(acceleration_structure_build),
    }
}

impl PipelineStage {
    pub fn bit(&self) -> u32 {
        match self {
            PipelineStage::TopOfPipe(_) => 0x00000001,
            PipelineStage::DrawIndirect(_) => 0x00000002,
            PipelineStage::VertexInput(_) => 0x00000004,
            PipelineStage::VertexShader(_) => 0x00000008,
            PipelineStage::TessellationControlShader(_) => 0x00000010,
            PipelineStage::TessellationEvaluationShader(_) => 0x00000020,
            PipelineStage::GeometryShader(_) => 0x00000040,
            PipelineStage::EarlyFragmentTests(_) => 0x00000100,
            PipelineStage::FragmentShader(_) => 0x00000080,
            PipelineStage::LateFragmentTests(_) => 0x00000200,
            PipelineStage::ColorAttachmentOutput(_) => 0x00000400,
            PipelineStage::ComputeShader(_) => 0x00000800,
            PipelineStage::Transfer(_) => 0x00001000,
            PipelineStage::BottomOfPipe(_) => 0x00002000,
            PipelineStage::Host(_) => 0x00004000,
            PipelineStage::AllGraphics(_) => 0x00008000,
            PipelineStage::AllCommands(_) => 0x00010000,
            PipelineStage::RayTracingShader(_) => 0x00200000,
            PipelineStage::AccelerationStructureBuild(_) => 0x02000000,
        }
    }
}

pub fn combined_pipeline_stage_flags(stages: impl Iterator<Item = PipelineStage>) -> u32 {
    stages.fold(0, |flags, stage| flags | stage.bit())
}

impl PipelineStages {
    pub fn bits(&self) -> u32 {
        combined_pipeline_stage_flags(self.flags.iter().copied())
    }
}

pub fn parse_pipeline_stages(
    stream: syn::parse::ParseStream,
) -> syn::Result<Punctuated<PipelineStage, syn::Token![,]>> {
    stream.parse_terminated::<_, syn::Token![,]>(PipelineStage::parse)
}

pub fn pipeline_stages(tokens: proc_macro::TokenStream) -> TokenStream {
    let result = syn::parse::Parser::parse(parse_pipeline_stages, tokens);

    match result {
        Err(err) => err.into_compile_error(),
        Ok(stages) => {
            let flags = combined_pipeline_stage_flags(stages.iter().copied());
            quote::quote!(#flags)
        }
    }
}
