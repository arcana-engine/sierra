pub use crate::backend::Sampler;
use ordered_float::OrderedFloat;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum CompareOp {
    /// Never passes.
    Never,

    /// Passes if fragment's depth is less than stored.
    Less,

    /// Passes if fragment's depth is equal to stored.
    Equal,

    /// Passes if fragment's depth is less than or equal to stored.
    LessOrEqual,

    /// Passes if fragment's depth is greater than stored.
    Greater,

    /// Passes if fragment's depth is not equal to stored.
    NotEqual,

    /// Passes if fragment's depth is greater than or equal to stored.
    GreaterOrEqual,

    /// Always passes.
    Always,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum Filter {
    Nearest,
    Linear,
    // Cubic,
}

impl Default for Filter {
    fn default() -> Self {
        Filter::Nearest
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum MipmapMode {
    Nearest,
    Linear,
}

impl Default for MipmapMode {
    fn default() -> Self {
        MipmapMode::Nearest
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum SamplerAddressMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
    MirrorClampToEdge,
}

impl Default for SamplerAddressMode {
    fn default() -> Self {
        SamplerAddressMode::ClampToEdge
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum BorderColor {
    FloatTransparentBlack,
    IntTransparentBlack,
    FloatOpaqueBlack,
    IntOpaqueBlack,
    FloatOpaqueWhite,
    IntOpaqueWhite,
}

impl Default for BorderColor {
    fn default() -> Self {
        BorderColor::FloatTransparentBlack
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct SamplerInfo {
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub mag_filter: Filter,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub min_filter: Filter,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub mipmap_mode: MipmapMode,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub address_mode_u: SamplerAddressMode,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub address_mode_v: SamplerAddressMode,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub address_mode_w: SamplerAddressMode,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub mip_lod_bias: OrderedFloat<f32>,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub max_anisotropy: Option<OrderedFloat<f32>>,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub compare_op: Option<CompareOp>,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub min_lod: OrderedFloat<f32>,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub max_lod: OrderedFloat<f32>,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub border_color: BorderColor,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub unnormalized_coordinates: bool,
}

impl SamplerInfo {
    pub const fn new() -> Self {
        SamplerInfo {
            mag_filter: Filter::Nearest,
            min_filter: Filter::Nearest,
            mipmap_mode: MipmapMode::Nearest,
            address_mode_u: SamplerAddressMode::ClampToEdge,
            address_mode_v: SamplerAddressMode::ClampToEdge,
            address_mode_w: SamplerAddressMode::ClampToEdge,
            mip_lod_bias: OrderedFloat(0.0),
            max_anisotropy: None,
            compare_op: None,
            min_lod: OrderedFloat(0.0),
            max_lod: OrderedFloat(0.0),
            border_color: BorderColor::FloatTransparentBlack,
            unnormalized_coordinates: false,
        }
    }
}

impl Default for SamplerInfo {
    fn default() -> Self {
        SamplerInfo::new()
    }
}

#[cfg(feature = "serde-1")]
mod defaults {
    use ordered_float::OrderedFloat;

    pub const fn max_lod() -> OrderedFloat<f32> {
        OrderedFloat(1000.0)
    }
}
