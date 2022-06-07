/// Texel format.
/// Images can have different texel formats.
/// Some of which are color or depth and/or stencil.
/// Format defines components, number of bits, layout and representation of
/// texels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum Format {
    R8Unorm,
    R8Snorm,
    R8Uscaled,
    R8Sscaled,
    R8Uint,
    R8Sint,
    R8Srgb,
    RG8Unorm,
    RG8Snorm,
    RG8Uscaled,
    RG8Sscaled,
    RG8Uint,
    RG8Sint,
    RG8Srgb,
    RGB8Unorm,
    RGB8Snorm,
    RGB8Uscaled,
    RGB8Sscaled,
    RGB8Uint,
    RGB8Sint,
    RGB8Srgb,
    BGR8Unorm,
    BGR8Snorm,
    BGR8Uscaled,
    BGR8Sscaled,
    BGR8Uint,
    BGR8Sint,
    BGR8Srgb,
    RGBA8Unorm,
    RGBA8Snorm,
    RGBA8Uscaled,
    RGBA8Sscaled,
    RGBA8Uint,
    RGBA8Sint,
    RGBA8Srgb,
    BGRA8Unorm,
    BGRA8Snorm,
    BGRA8Uscaled,
    BGRA8Sscaled,
    BGRA8Uint,
    BGRA8Sint,
    BGRA8Srgb,
    R16Unorm,
    R16Snorm,
    R16Uscaled,
    R16Sscaled,
    R16Uint,
    R16Sint,
    R16Sfloat,
    RG16Unorm,
    RG16Snorm,
    RG16Uscaled,
    RG16Sscaled,
    RG16Uint,
    RG16Sint,
    RG16Sfloat,
    RGB16Unorm,
    RGB16Snorm,
    RGB16Uscaled,
    RGB16Sscaled,
    RGB16Uint,
    RGB16Sint,
    RGB16Sfloat,
    RGBA16Unorm,
    RGBA16Snorm,
    RGBA16Uscaled,
    RGBA16Sscaled,
    RGBA16Uint,
    RGBA16Sint,
    RGBA16Sfloat,
    R32Uint,
    R32Sint,
    R32Sfloat,
    RG32Uint,
    RG32Sint,
    RG32Sfloat,
    RGB32Uint,
    RGB32Sint,
    RGB32Sfloat,
    RGBA32Uint,
    RGBA32Sint,
    RGBA32Sfloat,
    R64Uint,
    R64Sint,
    R64Sfloat,
    RG64Uint,
    RG64Sint,
    RG64Sfloat,
    RGB64Uint,
    RGB64Sint,
    RGB64Sfloat,
    RGBA64Uint,
    RGBA64Sint,
    RGBA64Sfloat,
    D16Unorm,
    D32Sfloat,
    S8Uint,
    D16UnormS8Uint,
    D24UnormS8Uint,
    D32SfloatS8Uint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum Type {
    Uint,
    Sint,
    Srgb,
    Unorm,
    Snorm,
    Uscaled,
    Sscaled,
    Sfloat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum Channels {
    R,
    RG,
    RGB,
    BGR,
    RGBA,
    BGRA,
    D,
    S,
    DS,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct FormatDescription<C, B, T> {
    pub channels: C,
    pub bits: B,
    pub ty: T,
}

bitflags::bitflags! {
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct AspectFlags: u8 {
        const COLOR = 0x1;
        const DEPTH = 0x2;
        const STENCIL = 0x4;
    }
}

impl Format {
    pub fn aspect_flags(&self) -> AspectFlags {
        let mut flags = AspectFlags::empty();

        if self.is_color() {
            flags |= AspectFlags::COLOR;
        }

        if self.is_depth() {
            flags |= AspectFlags::DEPTH;
        }

        if self.is_stencil() {
            flags |= AspectFlags::STENCIL;
        }

        flags
    }

    pub fn is_color(&self) -> bool {
        !matches!(
            self,
            Self::D16Unorm
                | Self::D32Sfloat
                | Self::S8Uint
                | Self::D16UnormS8Uint
                | Self::D24UnormS8Uint
                | Self::D32SfloatS8Uint
        )
    }

    pub fn is_depth(&self) -> bool {
        matches!(
            self,
            Self::D16Unorm
                | Self::D32Sfloat
                | Self::D16UnormS8Uint
                | Self::D24UnormS8Uint
                | Self::D32SfloatS8Uint
        )
    }

    pub fn is_stencil(&self) -> bool {
        matches!(
            self,
            Self::S8Uint | Self::D16UnormS8Uint | Self::D24UnormS8Uint | Self::D32SfloatS8Uint
        )
    }

    pub fn description(&self) -> FormatDescription<Channels, u32, Type> {
        match self {
            Self::R8Unorm => FormatDescription {
                channels: Channels::R,
                ty: Type::Unorm,
                bits: 8,
            },
            Self::R8Snorm => FormatDescription {
                channels: Channels::R,
                ty: Type::Snorm,
                bits: 8,
            },
            Self::R8Uscaled => FormatDescription {
                channels: Channels::R,
                ty: Type::Uscaled,
                bits: 8,
            },
            Self::R8Sscaled => FormatDescription {
                channels: Channels::R,
                ty: Type::Sscaled,
                bits: 8,
            },
            Self::R8Uint => FormatDescription {
                channels: Channels::R,
                ty: Type::Uint,
                bits: 8,
            },
            Self::R8Sint => FormatDescription {
                channels: Channels::R,
                ty: Type::Sint,
                bits: 8,
            },
            Self::R8Srgb => FormatDescription {
                channels: Channels::R,
                ty: Type::Srgb,
                bits: 8,
            },
            Self::RG8Unorm => FormatDescription {
                channels: Channels::RG,
                ty: Type::Unorm,
                bits: 8,
            },
            Self::RG8Snorm => FormatDescription {
                channels: Channels::RG,
                ty: Type::Snorm,
                bits: 8,
            },
            Self::RG8Uscaled => FormatDescription {
                channels: Channels::RG,
                ty: Type::Uscaled,
                bits: 8,
            },
            Self::RG8Sscaled => FormatDescription {
                channels: Channels::RG,
                ty: Type::Sscaled,
                bits: 8,
            },
            Self::RG8Uint => FormatDescription {
                channels: Channels::RG,
                ty: Type::Uint,
                bits: 8,
            },
            Self::RG8Sint => FormatDescription {
                channels: Channels::RG,
                ty: Type::Sint,
                bits: 8,
            },
            Self::RG8Srgb => FormatDescription {
                channels: Channels::RG,
                ty: Type::Srgb,
                bits: 8,
            },
            Self::RGB8Unorm => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Unorm,
                bits: 8,
            },
            Self::RGB8Snorm => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Snorm,
                bits: 8,
            },
            Self::RGB8Uscaled => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Uscaled,
                bits: 8,
            },
            Self::RGB8Sscaled => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Sscaled,
                bits: 8,
            },
            Self::RGB8Uint => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Uint,
                bits: 8,
            },
            Self::RGB8Sint => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Sint,
                bits: 8,
            },
            Self::RGB8Srgb => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Srgb,
                bits: 8,
            },
            Self::BGR8Unorm => FormatDescription {
                channels: Channels::BGR,
                ty: Type::Unorm,
                bits: 8,
            },
            Self::BGR8Snorm => FormatDescription {
                channels: Channels::BGR,
                ty: Type::Snorm,
                bits: 8,
            },
            Self::BGR8Uscaled => FormatDescription {
                channels: Channels::BGR,
                ty: Type::Uscaled,
                bits: 8,
            },
            Self::BGR8Sscaled => FormatDescription {
                channels: Channels::BGR,
                ty: Type::Sscaled,
                bits: 8,
            },
            Self::BGR8Uint => FormatDescription {
                channels: Channels::BGR,
                ty: Type::Uint,
                bits: 8,
            },
            Self::BGR8Sint => FormatDescription {
                channels: Channels::BGR,
                ty: Type::Sint,
                bits: 8,
            },
            Self::BGR8Srgb => FormatDescription {
                channels: Channels::BGR,
                ty: Type::Srgb,
                bits: 8,
            },
            Self::RGBA8Unorm => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Unorm,
                bits: 8,
            },
            Self::RGBA8Snorm => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Snorm,
                bits: 8,
            },
            Self::RGBA8Uscaled => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Uscaled,
                bits: 8,
            },
            Self::RGBA8Sscaled => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Sscaled,
                bits: 8,
            },
            Self::RGBA8Uint => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Uint,
                bits: 8,
            },
            Self::RGBA8Sint => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Sint,
                bits: 8,
            },
            Self::RGBA8Srgb => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Srgb,
                bits: 8,
            },
            Self::BGRA8Unorm => FormatDescription {
                channels: Channels::BGRA,
                ty: Type::Unorm,
                bits: 8,
            },
            Self::BGRA8Snorm => FormatDescription {
                channels: Channels::BGRA,
                ty: Type::Snorm,
                bits: 8,
            },
            Self::BGRA8Uscaled => FormatDescription {
                channels: Channels::BGRA,
                ty: Type::Uscaled,
                bits: 8,
            },
            Self::BGRA8Sscaled => FormatDescription {
                channels: Channels::BGRA,
                ty: Type::Sscaled,
                bits: 8,
            },
            Self::BGRA8Uint => FormatDescription {
                channels: Channels::BGRA,
                ty: Type::Uint,
                bits: 8,
            },
            Self::BGRA8Sint => FormatDescription {
                channels: Channels::BGRA,
                ty: Type::Sint,
                bits: 8,
            },
            Self::BGRA8Srgb => FormatDescription {
                channels: Channels::BGRA,
                ty: Type::Srgb,
                bits: 8,
            },
            Self::R16Unorm => FormatDescription {
                channels: Channels::R,
                ty: Type::Unorm,
                bits: 16,
            },
            Self::R16Snorm => FormatDescription {
                channels: Channels::R,
                ty: Type::Snorm,
                bits: 16,
            },
            Self::R16Uscaled => FormatDescription {
                channels: Channels::R,
                ty: Type::Uscaled,
                bits: 16,
            },
            Self::R16Sscaled => FormatDescription {
                channels: Channels::R,
                ty: Type::Sscaled,
                bits: 16,
            },
            Self::R16Uint => FormatDescription {
                channels: Channels::R,
                ty: Type::Uint,
                bits: 16,
            },
            Self::R16Sint => FormatDescription {
                channels: Channels::R,
                ty: Type::Sint,
                bits: 16,
            },
            Self::R16Sfloat => FormatDescription {
                channels: Channels::R,
                ty: Type::Sfloat,
                bits: 16,
            },
            Self::RG16Unorm => FormatDescription {
                channels: Channels::RG,
                ty: Type::Unorm,
                bits: 16,
            },
            Self::RG16Snorm => FormatDescription {
                channels: Channels::RG,
                ty: Type::Snorm,
                bits: 16,
            },
            Self::RG16Uscaled => FormatDescription {
                channels: Channels::RG,
                ty: Type::Uscaled,
                bits: 16,
            },
            Self::RG16Sscaled => FormatDescription {
                channels: Channels::RG,
                ty: Type::Sscaled,
                bits: 16,
            },
            Self::RG16Uint => FormatDescription {
                channels: Channels::RG,
                ty: Type::Uint,
                bits: 16,
            },
            Self::RG16Sint => FormatDescription {
                channels: Channels::RG,
                ty: Type::Sint,
                bits: 16,
            },
            Self::RG16Sfloat => FormatDescription {
                channels: Channels::RG,
                ty: Type::Sfloat,
                bits: 16,
            },
            Self::RGB16Unorm => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Unorm,
                bits: 16,
            },
            Self::RGB16Snorm => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Snorm,
                bits: 16,
            },
            Self::RGB16Uscaled => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Uscaled,
                bits: 16,
            },
            Self::RGB16Sscaled => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Sscaled,
                bits: 16,
            },
            Self::RGB16Uint => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Uint,
                bits: 16,
            },
            Self::RGB16Sint => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Sint,
                bits: 16,
            },
            Self::RGB16Sfloat => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Sfloat,
                bits: 16,
            },
            Self::RGBA16Unorm => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Unorm,
                bits: 16,
            },
            Self::RGBA16Snorm => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Snorm,
                bits: 16,
            },
            Self::RGBA16Uscaled => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Uscaled,
                bits: 16,
            },
            Self::RGBA16Sscaled => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Sscaled,
                bits: 16,
            },
            Self::RGBA16Uint => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Uint,
                bits: 16,
            },
            Self::RGBA16Sint => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Sint,
                bits: 16,
            },
            Self::RGBA16Sfloat => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Sfloat,
                bits: 16,
            },
            Self::R32Uint => FormatDescription {
                channels: Channels::R,
                ty: Type::Uint,
                bits: 32,
            },
            Self::R32Sint => FormatDescription {
                channels: Channels::R,
                ty: Type::Sint,
                bits: 32,
            },
            Self::R32Sfloat => FormatDescription {
                channels: Channels::R,
                ty: Type::Sfloat,
                bits: 32,
            },
            Self::RG32Uint => FormatDescription {
                channels: Channels::RG,
                ty: Type::Uint,
                bits: 32,
            },
            Self::RG32Sint => FormatDescription {
                channels: Channels::RG,
                ty: Type::Sint,
                bits: 32,
            },
            Self::RG32Sfloat => FormatDescription {
                channels: Channels::RG,
                ty: Type::Sfloat,
                bits: 32,
            },
            Self::RGB32Uint => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Uint,
                bits: 32,
            },
            Self::RGB32Sint => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Sint,
                bits: 32,
            },
            Self::RGB32Sfloat => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Sfloat,
                bits: 32,
            },
            Self::RGBA32Uint => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Uint,
                bits: 32,
            },
            Self::RGBA32Sint => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Sint,
                bits: 32,
            },
            Self::RGBA32Sfloat => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Sfloat,
                bits: 32,
            },
            Self::R64Uint => FormatDescription {
                channels: Channels::R,
                ty: Type::Uint,
                bits: 64,
            },
            Self::R64Sint => FormatDescription {
                channels: Channels::R,
                ty: Type::Sint,
                bits: 64,
            },
            Self::R64Sfloat => FormatDescription {
                channels: Channels::R,
                ty: Type::Sfloat,
                bits: 64,
            },
            Self::RG64Uint => FormatDescription {
                channels: Channels::RG,
                ty: Type::Uint,
                bits: 64,
            },
            Self::RG64Sint => FormatDescription {
                channels: Channels::RG,
                ty: Type::Sint,
                bits: 64,
            },
            Self::RG64Sfloat => FormatDescription {
                channels: Channels::RG,
                ty: Type::Sfloat,
                bits: 64,
            },
            Self::RGB64Uint => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Uint,
                bits: 64,
            },
            Self::RGB64Sint => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Sint,
                bits: 64,
            },
            Self::RGB64Sfloat => FormatDescription {
                channels: Channels::RGB,
                ty: Type::Sfloat,
                bits: 64,
            },
            Self::RGBA64Uint => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Uint,
                bits: 64,
            },
            Self::RGBA64Sint => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Sint,
                bits: 64,
            },
            Self::RGBA64Sfloat => FormatDescription {
                channels: Channels::RGBA,
                ty: Type::Sfloat,
                bits: 64,
            },
            Self::D16Unorm => FormatDescription {
                channels: Channels::D,
                ty: Type::Unorm,
                bits: 16,
            },
            Self::D32Sfloat => FormatDescription {
                channels: Channels::D,
                ty: Type::Sfloat,
                bits: 32,
            },
            Self::S8Uint => FormatDescription {
                channels: Channels::S,
                ty: Type::Uint,
                bits: 8,
            },
            Self::D16UnormS8Uint => FormatDescription {
                channels: Channels::DS,
                ty: Type::Unorm,
                bits: 16,
            },
            Self::D24UnormS8Uint => FormatDescription {
                channels: Channels::DS,
                bits: 24,
                ty: Type::Unorm,
            },
            Self::D32SfloatS8Uint => FormatDescription {
                channels: Channels::DS,
                bits: 32,
                ty: Type::Sfloat,
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum R {}

#[derive(Clone, Copy, Debug)]
pub enum RG {}

#[derive(Clone, Copy, Debug)]
pub enum RGB {}

#[derive(Clone, Copy, Debug)]
pub enum BGR {}

#[derive(Clone, Copy, Debug)]
pub enum RGBA {}

#[derive(Clone, Copy, Debug)]
pub enum BGRA {}

#[derive(Clone, Copy, Debug)]
pub enum D {}

#[derive(Clone, Copy, Debug)]
pub enum S {}

#[derive(Clone, Copy, Debug)]
pub enum DS {}

#[derive(Clone, Copy, Debug)]
pub enum Uint {}

#[derive(Clone, Copy, Debug)]
pub enum Sint {}

#[derive(Clone, Copy, Debug)]
pub enum Srgb {}

#[derive(Clone, Copy, Debug)]
pub enum Unorm {}

#[derive(Clone, Copy, Debug)]
pub enum Snorm {}

#[derive(Clone, Copy, Debug)]
pub enum Uscaled {}

#[derive(Clone, Copy, Debug)]
pub enum Sscaled {}

#[derive(Clone, Copy, Debug)]
pub enum Sfloat {}

#[derive(Clone, Copy, Debug)]
pub enum ConstBits<const BITS: u32> {}

pub trait StaticFormat {
    const FORMAT: Format;
}

impl StaticFormat for FormatDescription<R, ConstBits<8>, Unorm> {
    const FORMAT: Format = Format::R8Unorm;
}
impl StaticFormat for FormatDescription<R, ConstBits<8>, Snorm> {
    const FORMAT: Format = Format::R8Snorm;
}
impl StaticFormat for FormatDescription<R, ConstBits<8>, Uscaled> {
    const FORMAT: Format = Format::R8Uscaled;
}
impl StaticFormat for FormatDescription<R, ConstBits<8>, Sscaled> {
    const FORMAT: Format = Format::R8Sscaled;
}
impl StaticFormat for FormatDescription<R, ConstBits<8>, Uint> {
    const FORMAT: Format = Format::R8Uint;
}
impl StaticFormat for FormatDescription<R, ConstBits<8>, Sint> {
    const FORMAT: Format = Format::R8Sint;
}
impl StaticFormat for FormatDescription<R, ConstBits<8>, Srgb> {
    const FORMAT: Format = Format::R8Srgb;
}
impl StaticFormat for FormatDescription<RG, ConstBits<8>, Unorm> {
    const FORMAT: Format = Format::RG8Unorm;
}
impl StaticFormat for FormatDescription<RG, ConstBits<8>, Snorm> {
    const FORMAT: Format = Format::RG8Snorm;
}
impl StaticFormat for FormatDescription<RG, ConstBits<8>, Uscaled> {
    const FORMAT: Format = Format::RG8Uscaled;
}
impl StaticFormat for FormatDescription<RG, ConstBits<8>, Sscaled> {
    const FORMAT: Format = Format::RG8Sscaled;
}
impl StaticFormat for FormatDescription<RG, ConstBits<8>, Uint> {
    const FORMAT: Format = Format::RG8Uint;
}
impl StaticFormat for FormatDescription<RG, ConstBits<8>, Sint> {
    const FORMAT: Format = Format::RG8Sint;
}
impl StaticFormat for FormatDescription<RG, ConstBits<8>, Srgb> {
    const FORMAT: Format = Format::RG8Srgb;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<8>, Unorm> {
    const FORMAT: Format = Format::RGB8Unorm;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<8>, Snorm> {
    const FORMAT: Format = Format::RGB8Snorm;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<8>, Uscaled> {
    const FORMAT: Format = Format::RGB8Uscaled;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<8>, Sscaled> {
    const FORMAT: Format = Format::RGB8Sscaled;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<8>, Uint> {
    const FORMAT: Format = Format::RGB8Uint;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<8>, Sint> {
    const FORMAT: Format = Format::RGB8Sint;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<8>, Srgb> {
    const FORMAT: Format = Format::RGB8Srgb;
}
impl StaticFormat for FormatDescription<BGR, ConstBits<8>, Unorm> {
    const FORMAT: Format = Format::BGR8Unorm;
}
impl StaticFormat for FormatDescription<BGR, ConstBits<8>, Snorm> {
    const FORMAT: Format = Format::BGR8Snorm;
}
impl StaticFormat for FormatDescription<BGR, ConstBits<8>, Uscaled> {
    const FORMAT: Format = Format::BGR8Uscaled;
}
impl StaticFormat for FormatDescription<BGR, ConstBits<8>, Sscaled> {
    const FORMAT: Format = Format::BGR8Sscaled;
}
impl StaticFormat for FormatDescription<BGR, ConstBits<8>, Uint> {
    const FORMAT: Format = Format::BGR8Uint;
}
impl StaticFormat for FormatDescription<BGR, ConstBits<8>, Sint> {
    const FORMAT: Format = Format::BGR8Sint;
}
impl StaticFormat for FormatDescription<BGR, ConstBits<8>, Srgb> {
    const FORMAT: Format = Format::BGR8Srgb;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<8>, Unorm> {
    const FORMAT: Format = Format::RGBA8Unorm;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<8>, Snorm> {
    const FORMAT: Format = Format::RGBA8Snorm;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<8>, Uscaled> {
    const FORMAT: Format = Format::RGBA8Uscaled;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<8>, Sscaled> {
    const FORMAT: Format = Format::RGBA8Sscaled;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<8>, Uint> {
    const FORMAT: Format = Format::RGBA8Uint;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<8>, Sint> {
    const FORMAT: Format = Format::RGBA8Sint;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<8>, Srgb> {
    const FORMAT: Format = Format::RGBA8Srgb;
}
impl StaticFormat for FormatDescription<BGRA, ConstBits<8>, Unorm> {
    const FORMAT: Format = Format::BGRA8Unorm;
}
impl StaticFormat for FormatDescription<BGRA, ConstBits<8>, Snorm> {
    const FORMAT: Format = Format::BGRA8Snorm;
}
impl StaticFormat for FormatDescription<BGRA, ConstBits<8>, Uscaled> {
    const FORMAT: Format = Format::BGRA8Uscaled;
}
impl StaticFormat for FormatDescription<BGRA, ConstBits<8>, Sscaled> {
    const FORMAT: Format = Format::BGRA8Sscaled;
}
impl StaticFormat for FormatDescription<BGRA, ConstBits<8>, Uint> {
    const FORMAT: Format = Format::BGRA8Uint;
}
impl StaticFormat for FormatDescription<BGRA, ConstBits<8>, Sint> {
    const FORMAT: Format = Format::BGRA8Sint;
}
impl StaticFormat for FormatDescription<BGRA, ConstBits<8>, Srgb> {
    const FORMAT: Format = Format::BGRA8Srgb;
}
impl StaticFormat for FormatDescription<R, ConstBits<16>, Unorm> {
    const FORMAT: Format = Format::R16Unorm;
}
impl StaticFormat for FormatDescription<R, ConstBits<16>, Snorm> {
    const FORMAT: Format = Format::R16Snorm;
}
impl StaticFormat for FormatDescription<R, ConstBits<16>, Uscaled> {
    const FORMAT: Format = Format::R16Uscaled;
}
impl StaticFormat for FormatDescription<R, ConstBits<16>, Sscaled> {
    const FORMAT: Format = Format::R16Sscaled;
}
impl StaticFormat for FormatDescription<R, ConstBits<16>, Uint> {
    const FORMAT: Format = Format::R16Uint;
}
impl StaticFormat for FormatDescription<R, ConstBits<16>, Sint> {
    const FORMAT: Format = Format::R16Sint;
}
impl StaticFormat for FormatDescription<R, ConstBits<16>, Sfloat> {
    const FORMAT: Format = Format::R16Sfloat;
}
impl StaticFormat for FormatDescription<RG, ConstBits<16>, Unorm> {
    const FORMAT: Format = Format::RG16Unorm;
}
impl StaticFormat for FormatDescription<RG, ConstBits<16>, Snorm> {
    const FORMAT: Format = Format::RG16Snorm;
}
impl StaticFormat for FormatDescription<RG, ConstBits<16>, Uscaled> {
    const FORMAT: Format = Format::RG16Uscaled;
}
impl StaticFormat for FormatDescription<RG, ConstBits<16>, Sscaled> {
    const FORMAT: Format = Format::RG16Sscaled;
}
impl StaticFormat for FormatDescription<RG, ConstBits<16>, Uint> {
    const FORMAT: Format = Format::RG16Uint;
}
impl StaticFormat for FormatDescription<RG, ConstBits<16>, Sint> {
    const FORMAT: Format = Format::RG16Sint;
}
impl StaticFormat for FormatDescription<RG, ConstBits<16>, Sfloat> {
    const FORMAT: Format = Format::RG16Sfloat;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<16>, Unorm> {
    const FORMAT: Format = Format::RGB16Unorm;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<16>, Snorm> {
    const FORMAT: Format = Format::RGB16Snorm;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<16>, Uscaled> {
    const FORMAT: Format = Format::RGB16Uscaled;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<16>, Sscaled> {
    const FORMAT: Format = Format::RGB16Sscaled;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<16>, Uint> {
    const FORMAT: Format = Format::RGB16Uint;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<16>, Sint> {
    const FORMAT: Format = Format::RGB16Sint;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<16>, Sfloat> {
    const FORMAT: Format = Format::RGB16Sfloat;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<16>, Unorm> {
    const FORMAT: Format = Format::RGBA16Unorm;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<16>, Snorm> {
    const FORMAT: Format = Format::RGBA16Snorm;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<16>, Uscaled> {
    const FORMAT: Format = Format::RGBA16Uscaled;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<16>, Sscaled> {
    const FORMAT: Format = Format::RGBA16Sscaled;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<16>, Uint> {
    const FORMAT: Format = Format::RGBA16Uint;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<16>, Sint> {
    const FORMAT: Format = Format::RGBA16Sint;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<16>, Sfloat> {
    const FORMAT: Format = Format::RGBA16Sfloat;
}
impl StaticFormat for FormatDescription<R, ConstBits<32>, Uint> {
    const FORMAT: Format = Format::R32Uint;
}
impl StaticFormat for FormatDescription<R, ConstBits<32>, Sint> {
    const FORMAT: Format = Format::R32Sint;
}
impl StaticFormat for FormatDescription<R, ConstBits<32>, Sfloat> {
    const FORMAT: Format = Format::R32Sfloat;
}
impl StaticFormat for FormatDescription<RG, ConstBits<32>, Uint> {
    const FORMAT: Format = Format::RG32Uint;
}
impl StaticFormat for FormatDescription<RG, ConstBits<32>, Sint> {
    const FORMAT: Format = Format::RG32Sint;
}
impl StaticFormat for FormatDescription<RG, ConstBits<32>, Sfloat> {
    const FORMAT: Format = Format::RG32Sfloat;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<32>, Uint> {
    const FORMAT: Format = Format::RGB32Uint;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<32>, Sint> {
    const FORMAT: Format = Format::RGB32Sint;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<32>, Sfloat> {
    const FORMAT: Format = Format::RGB32Sfloat;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<32>, Uint> {
    const FORMAT: Format = Format::RGBA32Uint;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<32>, Sint> {
    const FORMAT: Format = Format::RGBA32Sint;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<32>, Sfloat> {
    const FORMAT: Format = Format::RGBA32Sfloat;
}
impl StaticFormat for FormatDescription<R, ConstBits<64>, Uint> {
    const FORMAT: Format = Format::R64Uint;
}
impl StaticFormat for FormatDescription<R, ConstBits<64>, Sint> {
    const FORMAT: Format = Format::R64Sint;
}
impl StaticFormat for FormatDescription<R, ConstBits<64>, Sfloat> {
    const FORMAT: Format = Format::R64Sfloat;
}
impl StaticFormat for FormatDescription<RG, ConstBits<64>, Uint> {
    const FORMAT: Format = Format::RG64Uint;
}
impl StaticFormat for FormatDescription<RG, ConstBits<64>, Sint> {
    const FORMAT: Format = Format::RG64Sint;
}
impl StaticFormat for FormatDescription<RG, ConstBits<64>, Sfloat> {
    const FORMAT: Format = Format::RG64Sfloat;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<64>, Uint> {
    const FORMAT: Format = Format::RGB64Uint;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<64>, Sint> {
    const FORMAT: Format = Format::RGB64Sint;
}
impl StaticFormat for FormatDescription<RGB, ConstBits<64>, Sfloat> {
    const FORMAT: Format = Format::RGB64Sfloat;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<64>, Uint> {
    const FORMAT: Format = Format::RGBA64Uint;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<64>, Sint> {
    const FORMAT: Format = Format::RGBA64Sint;
}
impl StaticFormat for FormatDescription<RGBA, ConstBits<64>, Sfloat> {
    const FORMAT: Format = Format::RGBA64Sfloat;
}
impl StaticFormat for FormatDescription<D, ConstBits<16>, Unorm> {
    const FORMAT: Format = Format::D16Unorm;
}
impl StaticFormat for FormatDescription<D, ConstBits<32>, Sfloat> {
    const FORMAT: Format = Format::D32Sfloat;
}
impl StaticFormat for FormatDescription<S, ConstBits<8>, Uint> {
    const FORMAT: Format = Format::S8Uint;
}
impl StaticFormat for FormatDescription<DS, ConstBits<16>, Unorm> {
    const FORMAT: Format = Format::D16UnormS8Uint;
}
impl StaticFormat for FormatDescription<DS, ConstBits<24>, Unorm> {
    const FORMAT: Format = Format::D24UnormS8Uint;
}
impl StaticFormat for FormatDescription<DS, ConstBits<32>, Sfloat> {
    const FORMAT: Format = Format::D32SfloatS8Uint;
}
