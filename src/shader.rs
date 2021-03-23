pub use crate::backend::ShaderModule;
use {
    crate::{assert_error, OutOfMemory},
    std::{
        convert::TryFrom,
        fmt::{self, Debug, Display},
    },
};

bitflags::bitflags! {
    /// Flags for each of graphics shaders.
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct ShaderStageFlags: u32 {
        const VERTEX                    = 0b0000000000001;
        const TESSELLATION_CONTROL      = 0b0000000000010;
        const TESSELLATION_EVALUATION   = 0b0000000000100;
        const GEOMETRY                  = 0b0000000001000;
        const FRAGMENT                  = 0b0000000010000;
        const COMPUTE                   = 0b0000000100000;
        const RAYGEN                    = 0b0000100000000;
        const ANY_HIT                   = 0b0001000000000;
        const CLOSEST_HIT               = 0b0010000000000;
        const MISS                      = 0b0100000000000;
        const INTERSECTION              = 0b1000000000000;

        const ALL_GRAPHICS              = 0b011111;
        const ALL                       = !0;
    }
}

/// Shader language.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ShaderLanguage {
    /// OpengGL Shading Language.
    GLSL,

    /// High Level Shading Language.
    HLSL,

    /// Standard Portable Intermediate Representation - V.
    SPIRV,
}

impl Display for ShaderLanguage {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GLSL => fmt.write_str("GLSL"),
            Self::HLSL => fmt.write_str("HLSL"),
            Self::SPIRV => fmt.write_str("SPIRV"),
        }
    }
}

/// Defines layout for descriptor sets.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct ShaderModuleInfo {
    /// Source code of the shader.
    #[cfg_attr(feature = "serde-1", serde(with = "serde_bytes"))]
    pub code: Box<[u8]>,

    /// Source language.
    pub language: ShaderLanguage,
}

impl Debug for ShaderModuleInfo {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let alternate = fmt.alternate();
        let mut ds = fmt.debug_struct("ShaderModuleInfo");
        ds.field("language", &self.language);
        if alternate {
            match std::str::from_utf8(&self.code) {
                Ok(code) => ds.field("code", &code),
                Err(_) => ds.field("code", &"<binary>"),
            }
        } else {
            ds.field("code", &"..")
        };
        ds.finish()
    }
}

impl ShaderModuleInfo {
    /// Creates GLSL shader module info.
    pub fn glsl(bytes: impl Into<Box<[u8]>>) -> Self {
        ShaderModuleInfo {
            code: bytes.into(),
            language: ShaderLanguage::GLSL,
        }
    }

    /// Creates HLSL shader module info.
    pub fn hlsl(bytes: impl Into<Box<[u8]>>) -> Self {
        ShaderModuleInfo {
            code: bytes.into(),
            language: ShaderLanguage::HLSL,
        }
    }

    /// Creates SPIR-V shader module info.
    pub fn spirv(bytes: impl Into<Box<[u8]>>) -> Self {
        ShaderModuleInfo {
            code: bytes.into(),
            language: ShaderLanguage::SPIRV,
        }
    }
}

/// Valid SPIR-V shader code.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
// FIXME: After implementing check
// produce unique key for shaders and use for comparison.
// FIXME: `Debug` must print human-readable version.
pub struct Spirv {
    #[cfg_attr(feature = "serde-1", serde(with = "serde_bytes"))]
    code: Box<[u8]>,
}

impl Spirv {
    /// Wraps raw bytes that must contain valid SPIR-V shader code.
    ///
    /// FIXME: Actually check validity.
    pub fn new(bytes: impl Into<Box<[u8]>>) -> Self {
        Spirv { code: bytes.into() }
    }
}

impl From<Spirv> for ShaderModuleInfo {
    fn from(shader: Spirv) -> Self {
        ShaderModuleInfo {
            code: shader.code.into(),
            language: ShaderLanguage::SPIRV,
        }
    }
}

/// Valid GLSL shader code.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
// FIXME: After implementing check
// produce unique key for shaders and use for comparison.
pub struct Glsl {
    code: Box<str>,
}

impl Glsl {
    /// Wraps string that must contain valid GLSL shader code.
    ///
    /// FIXME: Actually check validity.
    pub fn new(string: impl Into<Box<str>>) -> Self {
        Glsl {
            code: string.into(),
        }
    }
}

impl From<Glsl> for ShaderModuleInfo {
    fn from(shader: Glsl) -> Self {
        ShaderModuleInfo {
            code: shader.code.into(),
            language: ShaderLanguage::GLSL,
        }
    }
}

/// Valid HLSL shader code.
// FIXME: After implementing check
// produce unique key for shaders and use for comparison.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct Hlsl {
    code: Box<str>,
}

impl Hlsl {
    /// Wraps string that must contain valid HLSL shader code.
    ///
    /// FIXME: Actually check validity.
    pub fn new(string: impl Into<Box<str>>) -> Self {
        Hlsl {
            code: string.into(),
        }
    }
}

impl From<Hlsl> for ShaderModuleInfo {
    fn from(shader: Hlsl) -> Self {
        ShaderModuleInfo {
            code: shader.code.into(),
            language: ShaderLanguage::HLSL,
        }
    }
}

/// Shader module and entry point.
/// Uniquely identifies shader for pipeline.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Shader {
    /// Shader module created by `Device` from source.
    pub module: ShaderModule,

    /// Name of entry point.
    pub entry: Box<str>,

    /// Stage of this shader.
    pub stage: ShaderStage,
}

impl Shader {
    /// Creates new shader from module using "main" entry point.
    pub fn with_main(module: ShaderModule, stage: ShaderStage) -> Self {
        Shader {
            module,
            entry: "main".into(),
            stage,
        }
    }

    pub fn module(&self) -> &ShaderModule {
        &self.module
    }

    pub fn entry(&self) -> &str {
        &*self.entry
    }

    pub fn stage(&self) -> ShaderStage {
        self.stage
    }
}

#[derive(Clone, Copy, Debug, thiserror::Error)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum InvalidShader {
    #[error("Source is empty")]
    EmptySource,

    #[error("Source size is not multiple of 4 bytes")]
    SizeIsNotMultipleOfFour,

    #[error("Wrong spir-v magic. Expected 0x07230203, found 0x{found:x}")]
    WrongMagic { found: u32 },
}

#[derive(Debug, thiserror::Error)]
pub enum CreateShaderModuleError {
    #[error(transparent)]
    OutOfMemoryError {
        #[from]
        source: OutOfMemory,
    },

    #[error("Shader is invalid {source}")]
    InvalidShader {
        #[from]
        source: InvalidShader,
    },

    #[error("Shader language {language:?} is unsupported")]
    UnsupportedShaderLanguage { language: ShaderLanguage },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum ShaderStage {
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

impl Display for ShaderStage {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vertex => fmt.write_str("Vertex"),
            Self::TessellationControl => fmt.write_str("TessellationControl"),
            Self::TessellationEvaluation => {
                fmt.write_str("TessellationEvaluation")
            }
            Self::Geometry => fmt.write_str("Geometry"),
            Self::Fragment => fmt.write_str("Fragment"),
            Self::Compute => fmt.write_str("Compute"),
            Self::Raygen => fmt.write_str("Raygen"),
            Self::AnyHit => fmt.write_str("AnyHit"),
            Self::ClosestHit => fmt.write_str("ClosestHit"),
            Self::Miss => fmt.write_str("Miss"),
            Self::Intersection => fmt.write_str("Intersection"),
        }
    }
}

impl From<ShaderStage> for ShaderStageFlags {
    fn from(stage: ShaderStage) -> Self {
        match stage {
            ShaderStage::Vertex => ShaderStageFlags::VERTEX,
            ShaderStage::TessellationControl => {
                ShaderStageFlags::TESSELLATION_CONTROL
            }
            ShaderStage::TessellationEvaluation => {
                ShaderStageFlags::TESSELLATION_EVALUATION
            }
            ShaderStage::Geometry => ShaderStageFlags::GEOMETRY,
            ShaderStage::Fragment => ShaderStageFlags::FRAGMENT,
            ShaderStage::Compute => ShaderStageFlags::COMPUTE,
            ShaderStage::Raygen => ShaderStageFlags::RAYGEN,
            ShaderStage::AnyHit => ShaderStageFlags::ANY_HIT,
            ShaderStage::ClosestHit => ShaderStageFlags::CLOSEST_HIT,
            ShaderStage::Miss => ShaderStageFlags::MISS,
            ShaderStage::Intersection => ShaderStageFlags::INTERSECTION,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, thiserror::Error)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[error("Wrong shader stage. Expected {expected}, actual: {actual}")]
pub struct WrongShaderStage {
    expected: ShaderStage,
    actual: ShaderStage,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VertexShader {
    module: ShaderModule,
    entry: Box<str>,
}

impl VertexShader {
    pub fn new(module: ShaderModule, entry: impl Into<Box<str>>) -> Self {
        VertexShader {
            module,
            entry: entry.into(),
        }
    }

    pub fn with_main(module: ShaderModule) -> Self {
        VertexShader {
            module,
            entry: "main".into(),
        }
    }

    pub fn module(&self) -> &ShaderModule {
        &self.module
    }

    pub fn entry(&self) -> &str {
        &*self.entry
    }
}

impl TryFrom<Shader> for VertexShader {
    type Error = WrongShaderStage;

    fn try_from(shader: Shader) -> Result<Self, WrongShaderStage> {
        if shader.stage != ShaderStage::Vertex {
            Err(WrongShaderStage {
                actual: shader.stage,
                expected: ShaderStage::Vertex,
            })
        } else {
            Ok(VertexShader {
                module: shader.module,
                entry: shader.entry,
            })
        }
    }
}

impl From<VertexShader> for Shader {
    fn from(shader: VertexShader) -> Shader {
        Shader {
            module: shader.module,
            entry: shader.entry,
            stage: ShaderStage::Vertex,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TessellationControlShader {
    module: ShaderModule,
    entry: Box<str>,
}

impl TessellationControlShader {
    pub fn new(module: ShaderModule, entry: impl Into<Box<str>>) -> Self {
        TessellationControlShader {
            module,
            entry: entry.into(),
        }
    }

    pub fn with_main(module: ShaderModule) -> Self {
        TessellationControlShader {
            module,
            entry: "main".into(),
        }
    }

    pub fn module(&self) -> &ShaderModule {
        &self.module
    }

    pub fn entry(&self) -> &str {
        &*self.entry
    }
}

impl TryFrom<Shader> for TessellationControlShader {
    type Error = WrongShaderStage;

    fn try_from(shader: Shader) -> Result<Self, WrongShaderStage> {
        if shader.stage != ShaderStage::TessellationControl {
            Err(WrongShaderStage {
                actual: shader.stage,
                expected: ShaderStage::TessellationControl,
            })
        } else {
            Ok(TessellationControlShader {
                module: shader.module,
                entry: shader.entry,
            })
        }
    }
}

impl From<TessellationControlShader> for Shader {
    fn from(shader: TessellationControlShader) -> Shader {
        Shader {
            module: shader.module,
            entry: shader.entry,
            stage: ShaderStage::TessellationControl,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TessellationEvaluationShader {
    module: ShaderModule,
    entry: Box<str>,
}

impl TessellationEvaluationShader {
    pub fn new(module: ShaderModule, entry: impl Into<Box<str>>) -> Self {
        TessellationEvaluationShader {
            module,
            entry: entry.into(),
        }
    }

    pub fn with_main(module: ShaderModule) -> Self {
        TessellationEvaluationShader {
            module,
            entry: "main".into(),
        }
    }

    pub fn module(&self) -> &ShaderModule {
        &self.module
    }

    pub fn entry(&self) -> &str {
        &*self.entry
    }
}

impl TryFrom<Shader> for TessellationEvaluationShader {
    type Error = WrongShaderStage;

    fn try_from(shader: Shader) -> Result<Self, WrongShaderStage> {
        if shader.stage != ShaderStage::TessellationEvaluation {
            Err(WrongShaderStage {
                actual: shader.stage,
                expected: ShaderStage::TessellationEvaluation,
            })
        } else {
            Ok(TessellationEvaluationShader {
                module: shader.module,
                entry: shader.entry,
            })
        }
    }
}

impl From<TessellationEvaluationShader> for Shader {
    fn from(shader: TessellationEvaluationShader) -> Shader {
        Shader {
            module: shader.module,
            entry: shader.entry,
            stage: ShaderStage::TessellationEvaluation,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GeometryShader {
    module: ShaderModule,
    entry: Box<str>,
}

impl GeometryShader {
    pub fn new(module: ShaderModule, entry: impl Into<Box<str>>) -> Self {
        GeometryShader {
            module,
            entry: entry.into(),
        }
    }

    pub fn with_main(module: ShaderModule) -> Self {
        GeometryShader {
            module,
            entry: "main".into(),
        }
    }

    pub fn module(&self) -> &ShaderModule {
        &self.module
    }

    pub fn entry(&self) -> &str {
        &*self.entry
    }
}

impl TryFrom<Shader> for GeometryShader {
    type Error = WrongShaderStage;

    fn try_from(shader: Shader) -> Result<Self, WrongShaderStage> {
        if shader.stage != ShaderStage::Geometry {
            Err(WrongShaderStage {
                actual: shader.stage,
                expected: ShaderStage::Geometry,
            })
        } else {
            Ok(GeometryShader {
                module: shader.module,
                entry: shader.entry,
            })
        }
    }
}

impl From<GeometryShader> for Shader {
    fn from(shader: GeometryShader) -> Shader {
        Shader {
            module: shader.module,
            entry: shader.entry,
            stage: ShaderStage::Geometry,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FragmentShader {
    module: ShaderModule,
    entry: Box<str>,
}

impl FragmentShader {
    pub fn new(module: ShaderModule, entry: impl Into<Box<str>>) -> Self {
        FragmentShader {
            module,
            entry: entry.into(),
        }
    }

    pub fn with_main(module: ShaderModule) -> Self {
        FragmentShader {
            module,
            entry: "main".into(),
        }
    }

    pub fn module(&self) -> &ShaderModule {
        &self.module
    }

    pub fn entry(&self) -> &str {
        &*self.entry
    }
}

impl TryFrom<Shader> for FragmentShader {
    type Error = WrongShaderStage;

    fn try_from(shader: Shader) -> Result<Self, WrongShaderStage> {
        if shader.stage != ShaderStage::Fragment {
            Err(WrongShaderStage {
                actual: shader.stage,
                expected: ShaderStage::Fragment,
            })
        } else {
            Ok(FragmentShader {
                module: shader.module,
                entry: shader.entry,
            })
        }
    }
}

impl From<FragmentShader> for Shader {
    fn from(shader: FragmentShader) -> Shader {
        Shader {
            module: shader.module,
            entry: shader.entry,
            stage: ShaderStage::Fragment,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComputeShader {
    module: ShaderModule,
    entry: Box<str>,
}

impl ComputeShader {
    pub fn new(module: ShaderModule, entry: impl Into<Box<str>>) -> Self {
        ComputeShader {
            module,
            entry: entry.into(),
        }
    }

    pub fn with_main(module: ShaderModule) -> Self {
        ComputeShader {
            module,
            entry: "main".into(),
        }
    }

    pub fn module(&self) -> &ShaderModule {
        &self.module
    }

    pub fn entry(&self) -> &str {
        &*self.entry
    }
}

impl TryFrom<Shader> for ComputeShader {
    type Error = WrongShaderStage;

    fn try_from(shader: Shader) -> Result<Self, WrongShaderStage> {
        if shader.stage != ShaderStage::Compute {
            Err(WrongShaderStage {
                actual: shader.stage,
                expected: ShaderStage::Compute,
            })
        } else {
            Ok(ComputeShader {
                module: shader.module,
                entry: shader.entry,
            })
        }
    }
}

impl From<ComputeShader> for Shader {
    fn from(shader: ComputeShader) -> Shader {
        Shader {
            module: shader.module,
            entry: shader.entry,
            stage: ShaderStage::Compute,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RaygenShader {
    module: ShaderModule,
    entry: Box<str>,
}

impl RaygenShader {
    pub fn new(module: ShaderModule, entry: impl Into<Box<str>>) -> Self {
        RaygenShader {
            module,
            entry: entry.into(),
        }
    }

    pub fn with_main(module: ShaderModule) -> Self {
        RaygenShader {
            module,
            entry: "main".into(),
        }
    }

    pub fn module(&self) -> &ShaderModule {
        &self.module
    }

    pub fn entry(&self) -> &str {
        &*self.entry
    }
}

impl TryFrom<Shader> for RaygenShader {
    type Error = WrongShaderStage;

    fn try_from(shader: Shader) -> Result<Self, WrongShaderStage> {
        if shader.stage != ShaderStage::Raygen {
            Err(WrongShaderStage {
                actual: shader.stage,
                expected: ShaderStage::Raygen,
            })
        } else {
            Ok(RaygenShader {
                module: shader.module,
                entry: shader.entry,
            })
        }
    }
}

impl From<RaygenShader> for Shader {
    fn from(shader: RaygenShader) -> Shader {
        Shader {
            module: shader.module,
            entry: shader.entry,
            stage: ShaderStage::Raygen,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AnyHitShader {
    module: ShaderModule,
    entry: Box<str>,
}

impl AnyHitShader {
    pub fn new(module: ShaderModule, entry: impl Into<Box<str>>) -> Self {
        AnyHitShader {
            module,
            entry: entry.into(),
        }
    }

    pub fn with_main(module: ShaderModule) -> Self {
        AnyHitShader {
            module,
            entry: "main".into(),
        }
    }

    pub fn module(&self) -> &ShaderModule {
        &self.module
    }

    pub fn entry(&self) -> &str {
        &*self.entry
    }
}

impl TryFrom<Shader> for AnyHitShader {
    type Error = WrongShaderStage;

    fn try_from(shader: Shader) -> Result<Self, WrongShaderStage> {
        if shader.stage != ShaderStage::AnyHit {
            Err(WrongShaderStage {
                actual: shader.stage,
                expected: ShaderStage::AnyHit,
            })
        } else {
            Ok(AnyHitShader {
                module: shader.module,
                entry: shader.entry,
            })
        }
    }
}

impl From<AnyHitShader> for Shader {
    fn from(shader: AnyHitShader) -> Shader {
        Shader {
            module: shader.module,
            entry: shader.entry,
            stage: ShaderStage::AnyHit,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ClosestHitShader {
    module: ShaderModule,
    entry: Box<str>,
}

impl ClosestHitShader {
    pub fn new(module: ShaderModule, entry: impl Into<Box<str>>) -> Self {
        ClosestHitShader {
            module,
            entry: entry.into(),
        }
    }

    pub fn with_main(module: ShaderModule) -> Self {
        ClosestHitShader {
            module,
            entry: "main".into(),
        }
    }

    pub fn module(&self) -> &ShaderModule {
        &self.module
    }

    pub fn entry(&self) -> &str {
        &*self.entry
    }
}

impl TryFrom<Shader> for ClosestHitShader {
    type Error = WrongShaderStage;

    fn try_from(shader: Shader) -> Result<Self, WrongShaderStage> {
        if shader.stage != ShaderStage::ClosestHit {
            Err(WrongShaderStage {
                actual: shader.stage,
                expected: ShaderStage::ClosestHit,
            })
        } else {
            Ok(ClosestHitShader {
                module: shader.module,
                entry: shader.entry,
            })
        }
    }
}

impl From<ClosestHitShader> for Shader {
    fn from(shader: ClosestHitShader) -> Shader {
        Shader {
            module: shader.module,
            entry: shader.entry,
            stage: ShaderStage::ClosestHit,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MissShader {
    module: ShaderModule,
    entry: Box<str>,
}

impl MissShader {
    pub fn new(module: ShaderModule, entry: impl Into<Box<str>>) -> Self {
        MissShader {
            module,
            entry: entry.into(),
        }
    }

    pub fn with_main(module: ShaderModule) -> Self {
        MissShader {
            module,
            entry: "main".into(),
        }
    }

    pub fn module(&self) -> &ShaderModule {
        &self.module
    }

    pub fn entry(&self) -> &str {
        &*self.entry
    }
}

impl TryFrom<Shader> for MissShader {
    type Error = WrongShaderStage;

    fn try_from(shader: Shader) -> Result<Self, WrongShaderStage> {
        if shader.stage != ShaderStage::Miss {
            Err(WrongShaderStage {
                actual: shader.stage,
                expected: ShaderStage::Miss,
            })
        } else {
            Ok(MissShader {
                module: shader.module,
                entry: shader.entry,
            })
        }
    }
}

impl From<MissShader> for Shader {
    fn from(shader: MissShader) -> Shader {
        Shader {
            module: shader.module,
            entry: shader.entry,
            stage: ShaderStage::Miss,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IntersectionShader {
    module: ShaderModule,
    entry: Box<str>,
}

impl IntersectionShader {
    pub fn new(module: ShaderModule, entry: impl Into<Box<str>>) -> Self {
        IntersectionShader {
            module,
            entry: entry.into(),
        }
    }

    pub fn with_main(module: ShaderModule) -> Self {
        IntersectionShader {
            module,
            entry: "main".into(),
        }
    }

    pub fn module(&self) -> &ShaderModule {
        &self.module
    }

    pub fn entry(&self) -> &str {
        &*self.entry
    }
}

impl TryFrom<Shader> for IntersectionShader {
    type Error = WrongShaderStage;

    fn try_from(shader: Shader) -> Result<Self, WrongShaderStage> {
        if shader.stage != ShaderStage::Intersection {
            Err(WrongShaderStage {
                actual: shader.stage,
                expected: ShaderStage::Intersection,
            })
        } else {
            Ok(IntersectionShader {
                module: shader.module,
                entry: shader.entry,
            })
        }
    }
}

impl From<IntersectionShader> for Shader {
    fn from(shader: IntersectionShader) -> Shader {
        Shader {
            module: shader.module,
            entry: shader.entry,
            stage: ShaderStage::Intersection,
        }
    }
}

#[allow(dead_code)]
fn check_create_shader_module_error() {
    assert_error::<InvalidShader>();
    assert_error::<CreateShaderModuleError>();
}

#[cfg(feature = "shader-compiler")]
pub mod shader_compiler {
    use super::*;

    #[derive(Debug, thiserror::Error)]
    pub enum ShaderCompileFailed {
        #[error("Failed to compile shader. UTF-8 shader source code expected: {source}")]
        NonUTF8 {
            #[from]
            source: std::str::Utf8Error,
        },

        #[error("Shaderc failed to compile shader source code: {source}")]
        Shaderc {
            #[from]
            source: shaderc::Error,
        },
    }

    pub fn compile_shader(
        code: &[u8],
        entry: &str,
        language: ShaderLanguage,
        source_name: &str,
        include: impl Fn(&str, shaderc::IncludeType) -> Option<String>,
    ) -> Result<Box<[u8]>, ShaderCompileFailed> {
        let mut options = shaderc::CompileOptions::new().unwrap();

        options.set_source_language(match language {
            ShaderLanguage::GLSL => shaderc::SourceLanguage::GLSL,
            ShaderLanguage::HLSL => shaderc::SourceLanguage::HLSL,
            ShaderLanguage::SPIRV => return Ok(code.into()),
            // _ => return Err(ShaderCompileFailed::Unsupported { language }),
        });

        options.set_include_callback(|path, ty, _, _| {
            let content = include(path, ty).ok_or_else(|| {
                format!("Failed to load shader file {}", path)
            })?;

            Ok(shaderc::ResolvedInclude {
                resolved_name: path.to_owned(),
                content,
            })
        });

        let mut compiler = shaderc::Compiler::new().unwrap();

        let binary_result = compiler.compile_into_spirv(
            std::str::from_utf8(code)?,
            shaderc::ShaderKind::InferFromSource,
            source_name,
            entry,
            Some(&options),
        )?;

        if !binary_result.get_warning_messages().is_empty() {
            tracing::warn!("{}", binary_result.get_warning_messages());
        }

        Ok(binary_result.as_binary_u8().into())
    }
}
