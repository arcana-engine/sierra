pub use {
    self::Samples::*,
    crate::{access::AccessFlags, backend::Image},
};
use {
    crate::{
        format::{AspectFlags, Format},
        Extent2d, Extent3d, ImageSize, Offset3d,
    },
    std::ops::Range,
};

bitflags::bitflags! {
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct ImageUsage: u32 {
        const TRANSFER_SRC =                0x001;
        const TRANSFER_DST =                0x002;
        const SAMPLED =                     0x004;
        const STORAGE =                     0x008;
        const COLOR_ATTACHMENT =            0x010;
        const DEPTH_STENCIL_ATTACHMENT =    0x020;
        const TRANSIENT_ATTACHMENT =        0x040;
        const INPUT_ATTACHMENT =            0x080;
        const TRANSIENT =                   0x100;
    }
}

impl ImageUsage {
    pub fn is_render_target(self) -> bool {
        self.intersects(Self::COLOR_ATTACHMENT | Self::DEPTH_STENCIL_ATTACHMENT)
    }

    pub fn is_render_target_only(self) -> bool {
        self.is_render_target()
            && !self.intersects(
                Self::TRANSFER_SRC
                    | Self::TRANSFER_DST
                    | Self::SAMPLED
                    | Self::STORAGE
                    | Self::INPUT_ATTACHMENT,
            )
    }

    pub fn is_read_only(self) -> bool {
        !self.intersects(
            Self::TRANSFER_DST
                | Self::STORAGE
                | Self::COLOR_ATTACHMENT
                | Self::DEPTH_STENCIL_ATTACHMENT,
        )
    }
}

/// Image layout defines how texels are placed in memory.
/// Operations can be used in one or more layouts.
/// User is responsible to insert layout transition commands to ensure
/// that the image is in valid layout for each operation.
/// Pipeline barriers can be used to change layouts.
/// Additionally render pass can change layout of its attachments.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum Layout {
    /// Can be used with all device operations.
    /// Only presentation is not possible in this layout.
    /// Operations may perform slower in this layout.
    General,

    /// Can be used for color attachments.
    ColorAttachmentOptimal,

    /// Can be used for depth-stencil attachments.
    DepthStencilAttachmentOptimal,

    /// Can be used for depth-stencil attachments
    /// without writes.
    DepthStencilReadOnlyOptimal,

    /// Can be used for images accessed from shaders
    /// without writes.
    ShaderReadOnlyOptimal,

    /// Can be used for copy, blit and other transferring operations
    /// on source image.
    TransferSrcOptimal,

    /// Can be used for copy, blit and other transferring operations
    /// on destination image.
    TransferDstOptimal,

    /// Layout for swapchain images presentation.
    /// Should not be used if presentation feature is not enabled.
    Present,
}

/// Extent of the image.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum ImageExtent {
    /// One dimensional extent.
    D1 {
        /// Width of the image
        width: ImageSize,
    },
    /// Two dimensional extent.
    D2 {
        /// Width of the image
        width: ImageSize,

        /// Height of the image.
        height: ImageSize,
    },
    /// Three dimensional extent.
    D3 {
        /// Width of the image
        width: ImageSize,

        /// Height of the image.
        height: ImageSize,

        /// Depth of the image.
        depth: ImageSize,
    },
}

impl From<Extent2d> for ImageExtent {
    fn from(extent: Extent2d) -> Self {
        ImageExtent::D2 {
            width: extent.width,
            height: extent.height,
        }
    }
}

impl From<Extent3d> for ImageExtent {
    fn from(extent: Extent3d) -> Self {
        ImageExtent::D3 {
            width: extent.width,
            height: extent.height,
            depth: extent.depth,
        }
    }
}

impl ImageExtent {
    /// Convert image extent (1,2 or 3 dimensional) into 3 dimensional extent.
    /// If image doesn't have `height` or `depth`  they are set to 1.
    pub fn into_3d(self) -> Extent3d {
        match self {
            Self::D1 { width } => Extent3d {
                width,
                height: 1,
                depth: 1,
            },
            Self::D2 { width, height } => Extent3d {
                width,
                height,
                depth: 1,
            },
            Self::D3 {
                width,
                height,
                depth,
            } => Extent3d {
                width,
                height,
                depth,
            },
        }
    }

    /// Convert image extent (1,2 or 3 dimensional) into 2 dimensional extent.
    /// If image doesn't have `height` it is set to 1.
    /// `depth` is ignored.
    pub fn into_2d(self) -> Extent2d {
        match self {
            Self::D1 { width } => Extent2d { width, height: 1 },
            Self::D2 { width, height } => Extent2d { width, height },
            Self::D3 { width, height, .. } => Extent2d { width, height },
        }
    }
}

impl PartialEq<Extent2d> for ImageExtent {
    fn eq(&self, rhs: &Extent2d) -> bool {
        match self {
            ImageExtent::D2 { width, height } => *width == rhs.width && *height == rhs.height,
            _ => false,
        }
    }
}

impl PartialEq<Extent3d> for ImageExtent {
    fn eq(&self, rhs: &Extent3d) -> bool {
        match self {
            ImageExtent::D3 {
                width,
                height,
                depth,
            } => *width == rhs.width && *height == rhs.height && *depth == rhs.depth,
            _ => false,
        }
    }
}

impl PartialEq<ImageExtent> for Extent2d {
    fn eq(&self, rhs: &ImageExtent) -> bool {
        match rhs {
            ImageExtent::D2 { width, height } => self.width == *width && self.height == *height,
            _ => false,
        }
    }
}

impl PartialEq<ImageExtent> for Extent3d {
    fn eq(&self, rhs: &ImageExtent) -> bool {
        match rhs {
            ImageExtent::D3 {
                width,
                height,
                depth,
            } => self.width == *width && self.height == *height && self.depth == *depth,
            _ => false,
        }
    }
}

/// Number of samples for an image.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum Samples {
    /// 1 sample.
    Samples1,
    /// 2 samples.
    Samples2,
    /// 4 samples.
    Samples4,
    /// 8 samples.
    Samples8,
    /// 16 samples.
    Samples16,
    /// 32 samples.
    Samples32,
    /// 64 samples.
    Samples64,
}

impl Default for Samples {
    fn default() -> Self {
        Samples::Samples1
    }
}

/// Information required to create an image.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct ImageInfo {
    /// Dimensionality and size of those dimensions.
    pub extent: ImageExtent,

    /// Format for image texels.
    pub format: Format,

    /// Number of MIP levels.
    pub levels: u32,

    /// Number of array layers.
    pub layers: u32,

    /// Number of samples per texel.
    pub samples: Samples,

    /// Usage types supported by image.
    pub usage: ImageUsage,
}
/// Subresorce range of the image.
/// Used to create `ImageView`s.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct ImageSubresourceRange {
    pub aspect: AspectFlags,
    pub first_level: u32,
    pub level_count: u32,
    pub first_layer: u32,
    pub layer_count: u32,
}

impl ImageSubresourceRange {
    pub fn new(aspect: AspectFlags, levels: Range<u32>, layers: Range<u32>) -> Self {
        assert!(levels.end >= levels.start);

        assert!(layers.end >= layers.start);

        ImageSubresourceRange {
            aspect,
            first_level: levels.start,
            level_count: levels.end - levels.start,
            first_layer: layers.start,
            layer_count: layers.end - layers.start,
        }
    }

    pub fn whole(info: &ImageInfo) -> Self {
        ImageSubresourceRange {
            aspect: info.format.aspect_flags(),
            first_level: 0,
            level_count: info.levels,
            first_layer: 0,
            layer_count: info.layers,
        }
    }

    pub fn color(levels: Range<u32>, layers: Range<u32>) -> Self {
        Self::new(AspectFlags::COLOR, levels, layers)
    }

    pub fn depth(levels: Range<u32>, layers: Range<u32>) -> Self {
        Self::new(AspectFlags::DEPTH, levels, layers)
    }

    pub fn stencil(levels: Range<u32>, layers: Range<u32>) -> Self {
        Self::new(AspectFlags::STENCIL, levels, layers)
    }

    pub fn depth_stencil(levels: Range<u32>, layers: Range<u32>) -> Self {
        Self::new(AspectFlags::DEPTH | AspectFlags::STENCIL, levels, layers)
    }
}

/// Subresorce layers of the image.
/// Unlike `ImageSubresourceRange` it specifies only single mip-level.
/// Used in image copy operations.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct ImageSubresourceLayers {
    pub aspect: AspectFlags,
    pub level: u32,
    pub first_layer: u32,
    pub layer_count: u32,
}

impl ImageSubresourceLayers {
    pub fn new(aspect: AspectFlags, level: u32, layers: Range<u32>) -> Self {
        assert!(layers.end >= layers.start);

        ImageSubresourceLayers {
            aspect,
            level,
            first_layer: layers.start,
            layer_count: layers.end - layers.start,
        }
    }

    pub fn all_layers(info: &ImageInfo, level: u32) -> Self {
        assert!(level < info.levels);

        ImageSubresourceLayers {
            aspect: info.format.aspect_flags(),
            level,
            first_layer: 0,
            layer_count: info.layers,
        }
    }

    pub fn color(level: u32, layers: Range<u32>) -> Self {
        Self::new(AspectFlags::COLOR, level, layers)
    }

    pub fn depth(level: u32, layers: Range<u32>) -> Self {
        Self::new(AspectFlags::DEPTH, level, layers)
    }

    pub fn stencil(level: u32, layers: Range<u32>) -> Self {
        Self::new(AspectFlags::STENCIL, level, layers)
    }

    pub fn depth_stencil(level: u32, layers: Range<u32>) -> Self {
        Self::new(AspectFlags::DEPTH | AspectFlags::STENCIL, level, layers)
    }
}

/// Subresorce of the image.
/// Unlike `ImageSubresourceRange` it specifies only single mip-level and single
/// array layer.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct ImageSubresource {
    pub aspect: AspectFlags,
    pub level: u32,
    pub layer: u32,
}

impl ImageSubresource {
    pub fn new(aspect: AspectFlags, level: u32, layer: u32) -> Self {
        ImageSubresource {
            aspect,
            level,
            layer,
        }
    }

    pub fn from_info(info: &ImageInfo, level: u32, layer: u32) -> Self {
        assert!(level < info.levels);

        assert!(layer < info.layers);

        ImageSubresource {
            aspect: info.format.aspect_flags(),
            level,
            layer,
        }
    }

    pub fn color(level: u32, layer: u32) -> Self {
        Self::new(AspectFlags::COLOR, level, layer)
    }

    pub fn depth(level: u32, layer: u32) -> Self {
        Self::new(AspectFlags::DEPTH, level, layer)
    }

    pub fn stencil(level: u32, layer: u32) -> Self {
        Self::new(AspectFlags::STENCIL, level, layer)
    }

    pub fn depth_stencil(level: u32, layer: u32) -> Self {
        Self::new(AspectFlags::DEPTH | AspectFlags::STENCIL, level, layer)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct ImageBlit {
    pub src_subresource: ImageSubresourceLayers,
    pub src_offsets: [Offset3d; 2],
    pub dst_subresource: ImageSubresourceLayers,
    pub dst_offsets: [Offset3d; 2],
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ImageLayoutTransition<'a> {
    pub image: &'a Image,
    pub old: Option<(AccessFlags, Layout)>,
    pub new: (AccessFlags, Layout),
    pub subresource: ImageSubresourceRange,
}

impl<'a> ImageLayoutTransition<'a> {
    pub fn transition_whole(image: &'a Image, range: Range<(AccessFlags, Layout)>) -> Self {
        ImageLayoutTransition {
            subresource: ImageSubresourceRange::whole(image.info()),
            image,
            old: Some(range.start),
            new: range.end,
        }
    }

    pub fn initialize_whole(image: &'a Image, access: AccessFlags, layout: Layout) -> Self {
        ImageLayoutTransition {
            subresource: ImageSubresourceRange::whole(image.info()),
            image,
            old: None,
            new: (access, layout),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ImageMemoryBarrier<'a> {
    pub image: &'a Image,
    pub old: Option<(AccessFlags, Layout)>,
    pub new: (AccessFlags, Layout),
    pub family_transfer: Option<Range<u32>>,
    pub subresource: ImageSubresourceRange,
}

impl<'a> From<ImageLayoutTransition<'a>> for ImageMemoryBarrier<'a> {
    fn from(value: ImageLayoutTransition<'a>) -> Self {
        ImageMemoryBarrier {
            image: value.image,
            old: value.old,
            new: value.new,
            family_transfer: None,
            subresource: value.subresource,
        }
    }
}
