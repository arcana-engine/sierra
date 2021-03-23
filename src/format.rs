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
pub enum FormatType {
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
pub struct Repr {
    pub ty: FormatType,
    pub bits: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum FormatDescription {
    R(Repr),
    RG(Repr),
    RGB(Repr),
    BGR(Repr),
    RGBA(Repr),
    BGRA(Repr),
    Depth(Repr),
    Stencil(Repr),
    DepthStencil { depth: Repr, stencil: Repr },
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
        match self {
            Self::D16Unorm
            | Self::D32Sfloat
            | Self::S8Uint
            | Self::D16UnormS8Uint
            | Self::D24UnormS8Uint
            | Self::D32SfloatS8Uint => false,
            _ => true,
        }
    }

    pub fn color_type(&self) -> Option<FormatType> {
        match self.description() {
            FormatDescription::R(repr) => Some(repr.ty),
            FormatDescription::RG(repr) => Some(repr.ty),
            FormatDescription::RGB(repr) => Some(repr.ty),
            FormatDescription::BGR(repr) => Some(repr.ty),
            FormatDescription::RGBA(repr) => Some(repr.ty),
            FormatDescription::BGRA(repr) => Some(repr.ty),
            _ => None,
        }
    }

    pub fn is_depth(&self) -> bool {
        match self {
            Self::D16Unorm
            | Self::D32Sfloat
            | Self::D16UnormS8Uint
            | Self::D24UnormS8Uint
            | Self::D32SfloatS8Uint => true,
            _ => false,
        }
    }

    pub fn is_stencil(&self) -> bool {
        match self {
            Self::S8Uint
            | Self::D16UnormS8Uint
            | Self::D24UnormS8Uint
            | Self::D32SfloatS8Uint => true,
            _ => false,
        }
    }

    pub fn description(&self) -> FormatDescription {
        match self {
            Self::R8Unorm => FormatDescription::R(Repr {
                bits: 8,
                ty: FormatType::Unorm,
            }),
            Self::R8Snorm => FormatDescription::R(Repr {
                bits: 8,
                ty: FormatType::Snorm,
            }),
            Self::R8Uscaled => FormatDescription::R(Repr {
                bits: 8,
                ty: FormatType::Uscaled,
            }),
            Self::R8Sscaled => FormatDescription::R(Repr {
                bits: 8,
                ty: FormatType::Sscaled,
            }),
            Self::R8Uint => FormatDescription::R(Repr {
                bits: 8,
                ty: FormatType::Uint,
            }),
            Self::R8Sint => FormatDescription::R(Repr {
                bits: 8,
                ty: FormatType::Sint,
            }),
            Self::R8Srgb => FormatDescription::R(Repr {
                bits: 8,
                ty: FormatType::Srgb,
            }),
            Self::RG8Unorm => FormatDescription::RG(Repr {
                bits: 8,
                ty: FormatType::Unorm,
            }),
            Self::RG8Snorm => FormatDescription::RG(Repr {
                bits: 8,
                ty: FormatType::Snorm,
            }),
            Self::RG8Uscaled => FormatDescription::RG(Repr {
                bits: 8,
                ty: FormatType::Uscaled,
            }),
            Self::RG8Sscaled => FormatDescription::RG(Repr {
                bits: 8,
                ty: FormatType::Sscaled,
            }),
            Self::RG8Uint => FormatDescription::RG(Repr {
                bits: 8,
                ty: FormatType::Uint,
            }),
            Self::RG8Sint => FormatDescription::RG(Repr {
                bits: 8,
                ty: FormatType::Sint,
            }),
            Self::RG8Srgb => FormatDescription::RG(Repr {
                bits: 8,
                ty: FormatType::Srgb,
            }),
            Self::RGB8Unorm => FormatDescription::RGB(Repr {
                bits: 8,
                ty: FormatType::Unorm,
            }),
            Self::RGB8Snorm => FormatDescription::RGB(Repr {
                bits: 8,
                ty: FormatType::Snorm,
            }),
            Self::RGB8Uscaled => FormatDescription::RGB(Repr {
                bits: 8,
                ty: FormatType::Uscaled,
            }),
            Self::RGB8Sscaled => FormatDescription::RGB(Repr {
                bits: 8,
                ty: FormatType::Sscaled,
            }),
            Self::RGB8Uint => FormatDescription::RGB(Repr {
                bits: 8,
                ty: FormatType::Uint,
            }),
            Self::RGB8Sint => FormatDescription::RGB(Repr {
                bits: 8,
                ty: FormatType::Sint,
            }),
            Self::RGB8Srgb => FormatDescription::RGB(Repr {
                bits: 8,
                ty: FormatType::Srgb,
            }),
            Self::BGR8Unorm => FormatDescription::BGR(Repr {
                bits: 8,
                ty: FormatType::Unorm,
            }),
            Self::BGR8Snorm => FormatDescription::BGR(Repr {
                bits: 8,
                ty: FormatType::Snorm,
            }),
            Self::BGR8Uscaled => FormatDescription::BGR(Repr {
                bits: 8,
                ty: FormatType::Uscaled,
            }),
            Self::BGR8Sscaled => FormatDescription::BGR(Repr {
                bits: 8,
                ty: FormatType::Sscaled,
            }),
            Self::BGR8Uint => FormatDescription::BGR(Repr {
                bits: 8,
                ty: FormatType::Uint,
            }),
            Self::BGR8Sint => FormatDescription::BGR(Repr {
                bits: 8,
                ty: FormatType::Sint,
            }),
            Self::BGR8Srgb => FormatDescription::BGR(Repr {
                bits: 8,
                ty: FormatType::Srgb,
            }),
            Self::RGBA8Unorm => FormatDescription::RGBA(Repr {
                bits: 8,
                ty: FormatType::Unorm,
            }),
            Self::RGBA8Snorm => FormatDescription::RGBA(Repr {
                bits: 8,
                ty: FormatType::Snorm,
            }),
            Self::RGBA8Uscaled => FormatDescription::RGBA(Repr {
                bits: 8,
                ty: FormatType::Uscaled,
            }),
            Self::RGBA8Sscaled => FormatDescription::RGBA(Repr {
                bits: 8,
                ty: FormatType::Sscaled,
            }),
            Self::RGBA8Uint => FormatDescription::RGBA(Repr {
                bits: 8,
                ty: FormatType::Uint,
            }),
            Self::RGBA8Sint => FormatDescription::RGBA(Repr {
                bits: 8,
                ty: FormatType::Sint,
            }),
            Self::RGBA8Srgb => FormatDescription::RGBA(Repr {
                bits: 8,
                ty: FormatType::Srgb,
            }),
            Self::BGRA8Unorm => FormatDescription::BGRA(Repr {
                bits: 8,
                ty: FormatType::Unorm,
            }),
            Self::BGRA8Snorm => FormatDescription::BGRA(Repr {
                bits: 8,
                ty: FormatType::Snorm,
            }),
            Self::BGRA8Uscaled => FormatDescription::BGRA(Repr {
                bits: 8,
                ty: FormatType::Uscaled,
            }),
            Self::BGRA8Sscaled => FormatDescription::BGRA(Repr {
                bits: 8,
                ty: FormatType::Sscaled,
            }),
            Self::BGRA8Uint => FormatDescription::BGRA(Repr {
                bits: 8,
                ty: FormatType::Uint,
            }),
            Self::BGRA8Sint => FormatDescription::BGRA(Repr {
                bits: 8,
                ty: FormatType::Sint,
            }),
            Self::BGRA8Srgb => FormatDescription::BGRA(Repr {
                bits: 8,
                ty: FormatType::Srgb,
            }),
            Self::R16Unorm => FormatDescription::R(Repr {
                bits: 16,
                ty: FormatType::Unorm,
            }),
            Self::R16Snorm => FormatDescription::R(Repr {
                bits: 16,
                ty: FormatType::Snorm,
            }),
            Self::R16Uscaled => FormatDescription::R(Repr {
                bits: 16,
                ty: FormatType::Uscaled,
            }),
            Self::R16Sscaled => FormatDescription::R(Repr {
                bits: 16,
                ty: FormatType::Sscaled,
            }),
            Self::R16Uint => FormatDescription::R(Repr {
                bits: 16,
                ty: FormatType::Uint,
            }),
            Self::R16Sint => FormatDescription::R(Repr {
                bits: 16,
                ty: FormatType::Sint,
            }),
            Self::R16Sfloat => FormatDescription::R(Repr {
                bits: 16,
                ty: FormatType::Sfloat,
            }),
            Self::RG16Unorm => FormatDescription::RG(Repr {
                bits: 16,
                ty: FormatType::Unorm,
            }),
            Self::RG16Snorm => FormatDescription::RG(Repr {
                bits: 16,
                ty: FormatType::Snorm,
            }),
            Self::RG16Uscaled => FormatDescription::RG(Repr {
                bits: 16,
                ty: FormatType::Uscaled,
            }),
            Self::RG16Sscaled => FormatDescription::RG(Repr {
                bits: 16,
                ty: FormatType::Sscaled,
            }),
            Self::RG16Uint => FormatDescription::RG(Repr {
                bits: 16,
                ty: FormatType::Uint,
            }),
            Self::RG16Sint => FormatDescription::RG(Repr {
                bits: 16,
                ty: FormatType::Sint,
            }),
            Self::RG16Sfloat => FormatDescription::RG(Repr {
                bits: 16,
                ty: FormatType::Sfloat,
            }),
            Self::RGB16Unorm => FormatDescription::RGB(Repr {
                bits: 16,
                ty: FormatType::Unorm,
            }),
            Self::RGB16Snorm => FormatDescription::RGB(Repr {
                bits: 16,
                ty: FormatType::Snorm,
            }),
            Self::RGB16Uscaled => FormatDescription::RGB(Repr {
                bits: 16,
                ty: FormatType::Uscaled,
            }),
            Self::RGB16Sscaled => FormatDescription::RGB(Repr {
                bits: 16,
                ty: FormatType::Sscaled,
            }),
            Self::RGB16Uint => FormatDescription::RGB(Repr {
                bits: 16,
                ty: FormatType::Uint,
            }),
            Self::RGB16Sint => FormatDescription::RGB(Repr {
                bits: 16,
                ty: FormatType::Sint,
            }),
            Self::RGB16Sfloat => FormatDescription::RGB(Repr {
                bits: 16,
                ty: FormatType::Sfloat,
            }),
            Self::RGBA16Unorm => FormatDescription::RGBA(Repr {
                bits: 16,
                ty: FormatType::Unorm,
            }),
            Self::RGBA16Snorm => FormatDescription::RGBA(Repr {
                bits: 16,
                ty: FormatType::Snorm,
            }),
            Self::RGBA16Uscaled => FormatDescription::RGBA(Repr {
                bits: 16,
                ty: FormatType::Uscaled,
            }),
            Self::RGBA16Sscaled => FormatDescription::RGBA(Repr {
                bits: 16,
                ty: FormatType::Sscaled,
            }),
            Self::RGBA16Uint => FormatDescription::RGBA(Repr {
                bits: 16,
                ty: FormatType::Uint,
            }),
            Self::RGBA16Sint => FormatDescription::RGBA(Repr {
                bits: 16,
                ty: FormatType::Sint,
            }),
            Self::RGBA16Sfloat => FormatDescription::RGBA(Repr {
                bits: 16,
                ty: FormatType::Sfloat,
            }),
            Self::R32Uint => FormatDescription::R(Repr {
                bits: 32,
                ty: FormatType::Uint,
            }),
            Self::R32Sint => FormatDescription::R(Repr {
                bits: 32,
                ty: FormatType::Sint,
            }),
            Self::R32Sfloat => FormatDescription::R(Repr {
                bits: 32,
                ty: FormatType::Sfloat,
            }),
            Self::RG32Uint => FormatDescription::RG(Repr {
                bits: 32,
                ty: FormatType::Uint,
            }),
            Self::RG32Sint => FormatDescription::RG(Repr {
                bits: 32,
                ty: FormatType::Sint,
            }),
            Self::RG32Sfloat => FormatDescription::RG(Repr {
                bits: 32,
                ty: FormatType::Sfloat,
            }),
            Self::RGB32Uint => FormatDescription::RGB(Repr {
                bits: 32,
                ty: FormatType::Uint,
            }),
            Self::RGB32Sint => FormatDescription::RGB(Repr {
                bits: 32,
                ty: FormatType::Sint,
            }),
            Self::RGB32Sfloat => FormatDescription::RGB(Repr {
                bits: 32,
                ty: FormatType::Sfloat,
            }),
            Self::RGBA32Uint => FormatDescription::RGBA(Repr {
                bits: 32,
                ty: FormatType::Uint,
            }),
            Self::RGBA32Sint => FormatDescription::RGBA(Repr {
                bits: 32,
                ty: FormatType::Sint,
            }),
            Self::RGBA32Sfloat => FormatDescription::RGBA(Repr {
                bits: 32,
                ty: FormatType::Sfloat,
            }),
            Self::R64Uint => FormatDescription::R(Repr {
                bits: 64,
                ty: FormatType::Uint,
            }),
            Self::R64Sint => FormatDescription::R(Repr {
                bits: 64,
                ty: FormatType::Sint,
            }),
            Self::R64Sfloat => FormatDescription::R(Repr {
                bits: 64,
                ty: FormatType::Sfloat,
            }),
            Self::RG64Uint => FormatDescription::RG(Repr {
                bits: 64,
                ty: FormatType::Uint,
            }),
            Self::RG64Sint => FormatDescription::RG(Repr {
                bits: 64,
                ty: FormatType::Sint,
            }),
            Self::RG64Sfloat => FormatDescription::RG(Repr {
                bits: 64,
                ty: FormatType::Sfloat,
            }),
            Self::RGB64Uint => FormatDescription::RGB(Repr {
                bits: 64,
                ty: FormatType::Uint,
            }),
            Self::RGB64Sint => FormatDescription::RGB(Repr {
                bits: 64,
                ty: FormatType::Sint,
            }),
            Self::RGB64Sfloat => FormatDescription::RGB(Repr {
                bits: 64,
                ty: FormatType::Sfloat,
            }),
            Self::RGBA64Uint => FormatDescription::RGBA(Repr {
                bits: 64,
                ty: FormatType::Uint,
            }),
            Self::RGBA64Sint => FormatDescription::RGBA(Repr {
                bits: 64,
                ty: FormatType::Sint,
            }),
            Self::RGBA64Sfloat => FormatDescription::RGBA(Repr {
                bits: 64,
                ty: FormatType::Sfloat,
            }),
            Self::D16Unorm => FormatDescription::Depth(Repr {
                bits: 16,
                ty: FormatType::Unorm,
            }),
            Self::D32Sfloat => FormatDescription::Depth(Repr {
                bits: 32,
                ty: FormatType::Sfloat,
            }),
            Self::S8Uint => FormatDescription::Stencil(Repr {
                bits: 8,
                ty: FormatType::Uint,
            }),
            Self::D16UnormS8Uint => FormatDescription::DepthStencil {
                depth: Repr {
                    bits: 16,
                    ty: FormatType::Unorm,
                },
                stencil: Repr {
                    bits: 8,
                    ty: FormatType::Uint,
                },
            },
            Self::D24UnormS8Uint => FormatDescription::DepthStencil {
                depth: Repr {
                    bits: 24,
                    ty: FormatType::Unorm,
                },
                stencil: Repr {
                    bits: 8,
                    ty: FormatType::Uint,
                },
            },
            Self::D32SfloatS8Uint => FormatDescription::DepthStencil {
                depth: Repr {
                    bits: 32,
                    ty: FormatType::Sfloat,
                },
                stencil: Repr {
                    bits: 8,
                    ty: FormatType::Uint,
                },
            },
        }
    }
}
