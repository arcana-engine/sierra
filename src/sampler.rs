use std::hash::{Hash, Hasher};

pub use crate::backend::Sampler;

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

#[derive(Clone, Copy, Debug)]
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
    pub mip_lod_bias: f32,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub max_anisotropy: Option<f32>,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub compare_op: Option<CompareOp>,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub min_lod: f32,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub max_lod: f32,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub border_color: BorderColor,
    #[cfg_attr(feature = "serde-1", serde(default))]
    pub unnormalized_coordinates: bool,
}

impl PartialEq for SamplerInfo {
    fn eq(&self, other: &Self) -> bool {
        self.mag_filter == other.mag_filter
            && self.min_filter == other.min_filter
            && self.mipmap_mode == other.mipmap_mode
            && self.address_mode_u == other.address_mode_u
            && self.address_mode_v == other.address_mode_v
            && self.address_mode_w == other.address_mode_w
            && f32::to_bits(self.mip_lod_bias) == f32::to_bits(other.mip_lod_bias)
            && self.max_anisotropy.map(f32::to_bits) == other.max_anisotropy.map(f32::to_bits)
            && self.compare_op == other.compare_op
            && f32::to_bits(self.min_lod) == f32::to_bits(other.min_lod)
            && f32::to_bits(self.max_lod) == f32::to_bits(other.max_lod)
            && self.border_color == other.border_color
            && self.unnormalized_coordinates == other.unnormalized_coordinates
    }
}

impl Eq for SamplerInfo {}

impl Hash for SamplerInfo {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        Hash::hash(&self.mag_filter, hasher);
        Hash::hash(&self.min_filter, hasher);
        Hash::hash(&self.mipmap_mode, hasher);
        Hash::hash(&self.address_mode_u, hasher);
        Hash::hash(&self.address_mode_v, hasher);
        Hash::hash(&self.address_mode_w, hasher);
        Hash::hash(&f32::to_bits(self.mip_lod_bias), hasher);
        Hash::hash(&self.max_anisotropy.map(f32::to_bits), hasher);
        Hash::hash(&self.compare_op, hasher);
        Hash::hash(&f32::to_bits(self.min_lod), hasher);
        Hash::hash(&f32::to_bits(self.max_lod), hasher);
        Hash::hash(&self.border_color, hasher);
        Hash::hash(&self.unnormalized_coordinates, hasher);
    }
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
            mip_lod_bias: 0.0,
            max_anisotropy: None,
            compare_op: None,
            min_lod: 0.0,
            max_lod: 0.0,
            border_color: BorderColor::FloatTransparentBlack,
            unnormalized_coordinates: false,
        }
    }

    pub const fn linear() -> Self {
        SamplerInfo {
            mag_filter: Filter::Linear,
            min_filter: Filter::Linear,
            mipmap_mode: MipmapMode::Linear,
            address_mode_u: SamplerAddressMode::ClampToEdge,
            address_mode_v: SamplerAddressMode::ClampToEdge,
            address_mode_w: SamplerAddressMode::ClampToEdge,
            mip_lod_bias: 0.0,
            max_anisotropy: None,
            compare_op: None,
            min_lod: 0.0,
            max_lod: 0.0,
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
