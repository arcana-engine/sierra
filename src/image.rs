pub use {
    self::Samples::*,
    crate::{
        access::Access,
        backend::Image,
        encode::Encoder,
        queue::{Ownership, QueueId},
        stage::PipelineStageFlags,
    },
};
use {
    crate::{
        format::{AspectFlags, Format},
        Extent2d, Extent3d, ImageSize, Offset3d,
    },
    std::ops::Range,
};

bitflags::bitflags! {
    /// Flags to specify allowed usages for image.
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct ImageUsage: u32 {
        /// Image with this usage flag can be used as source for various transfer operations.
        const TRANSFER_SRC =                0x001;

        /// Image with this usage flag can be used as destination for various transfer operations.
        const TRANSFER_DST =                0x002;

        /// Image with this usage flag can be used as `SampledImage` descriptor.
        const SAMPLED =                     0x004;

        /// Image with this usage flag can be used as `StorageImage` descriptor.
        const STORAGE =                     0x008;

        /// Image with this usage flag can be used as color attachment in render passes.
        const COLOR_ATTACHMENT =            0x010;

        /// Image with this usage flag can be used as depth-stencil attachment in render passes.
        const DEPTH_STENCIL_ATTACHMENT =    0x020;

        /// Image with this usage flag can be used as input attachment in render passes.
        const INPUT_ATTACHMENT =            0x080;
    }
}

impl ImageUsage {
    /// Returns `true` if image with this usage flags can be used as render target, either color or depth.
    pub fn is_render_target(self) -> bool {
        self.intersects(Self::COLOR_ATTACHMENT | Self::DEPTH_STENCIL_ATTACHMENT)
    }

    /// Returns `true` if image with this usage flags can be used as render target, either color or depth,
    /// and no other usage is allowed.
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

    /// Returns `true` if no mutable usages allowed.
    /// Content still can be modified through memory mapping.
    pub fn is_read_only(self) -> bool {
        !self.intersects(
            Self::TRANSFER_DST
                | Self::STORAGE
                | Self::COLOR_ATTACHMENT
                | Self::DEPTH_STENCIL_ATTACHMENT,
        )
    }
}

/// When performing a synchronization operation which may transition
/// an image's layout, chooses whether to automatically compute the optimal
/// layout based on specified [`Access`]es or whether to transition manually
/// to a specified [`Layout`].
///
/// `Optimal` is usually preferred.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum SyncLayout {
    /// Choose the most optimal layout for each usage. Performs layout transitions as appropriate for the access.
    Optimal,

    /// Manually choose a RawLayout regardless of usage or access provided.
    Manual(Layout),
}

impl Default for SyncLayout {
    fn default() -> Self {
        SyncLayout::Optimal
    }
}

/// Image layout defines how texel are placed in memory.
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

impl Default for Layout {
    fn default() -> Self {
        Self::General
    }
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
pub struct SubresourceRange {
    pub aspect: AspectFlags,
    pub first_level: u32,
    pub level_count: u32,
    pub first_layer: u32,
    pub layer_count: u32,
}

impl SubresourceRange {
    pub fn new(aspect: AspectFlags, levels: Range<u32>, layers: Range<u32>) -> Self {
        assert!(levels.end >= levels.start);

        assert!(layers.end >= layers.start);

        SubresourceRange {
            aspect,
            first_level: levels.start,
            level_count: levels.end - levels.start,
            first_layer: layers.start,
            layer_count: layers.end - layers.start,
        }
    }

    pub fn whole(info: &ImageInfo) -> Self {
        SubresourceRange {
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
/// Unlike `SubresourceRange` it specifies only single mip-level.
/// Used in image copy operations.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct SubresourceLayers {
    pub aspect: AspectFlags,
    pub level: u32,
    pub first_layer: u32,
    pub layer_count: u32,
}

impl SubresourceLayers {
    pub fn new(aspect: AspectFlags, level: u32, layers: Range<u32>) -> Self {
        assert!(layers.end >= layers.start);

        SubresourceLayers {
            aspect,
            level,
            first_layer: layers.start,
            layer_count: layers.end - layers.start,
        }
    }

    pub fn all_layers(info: &ImageInfo, level: u32) -> Self {
        assert!(level < info.levels);

        SubresourceLayers {
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
/// Unlike `SubresourceRange` it specifies only single mip-level and single
/// array layer.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct Subresource {
    pub aspect: AspectFlags,
    pub level: u32,
    pub layer: u32,
}

impl Subresource {
    pub fn new(aspect: AspectFlags, level: u32, layer: u32) -> Self {
        Subresource {
            aspect,
            level,
            layer,
        }
    }

    pub fn from_info(info: &ImageInfo, level: u32, layer: u32) -> Self {
        assert!(level < info.levels);

        assert!(layer < info.layers);

        Subresource {
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
    pub src_subresource: SubresourceLayers,
    pub src_offsets: [Offset3d; 2],
    pub dst_subresource: SubresourceLayers,
    pub dst_offsets: [Offset3d; 2],
}

/// Transitions `image` between layouts. Shorthand helper for creating
/// an [`ImageMemoryBarrier`] when a layout transition with simple accesses
/// is desired. Some adavanced use cases should use [`ImageMemoryBarrier`] directly
/// instead.
///
/// If `old_layout` is empty, the contents of the image become
/// undefined after the barrier is executed, which can result in a performance
/// boost over attempting to preserve the contents. This is particularly useful
/// for transient images where the contents are going to be immediately overwritten.
/// A good example of when to use this is when an application re-uses a presented
/// image after acquiring the next swap chain image.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct LayoutTransition<'a> {
    pub image: &'a Image,
    pub prev_access: Access,
    pub next_access: Access,
    pub old_layout: Option<SyncLayout>,
    pub new_layout: SyncLayout,
    pub range: SubresourceRange,
}

impl<'a> LayoutTransition<'a> {
    pub fn transition_whole(
        image: &'a Image,
        access: Range<Access>,
        layout: Range<SyncLayout>,
    ) -> Self {
        LayoutTransition {
            range: SubresourceRange::whole(image.info()),
            image,
            prev_access: access.start,
            next_access: access.end,
            old_layout: Some(layout.start),
            new_layout: layout.end,
        }
    }

    pub fn initialize_whole(image: &'a Image, next_access: Access, layout: SyncLayout) -> Self {
        LayoutTransition {
            range: SubresourceRange::whole(image.info()),
            image,
            prev_access: Access::None,
            next_access,
            old_layout: None,
            new_layout: layout,
        }
    }
}

/// Image barriers should only be used when a queue family ownership transfer
/// or an image layout transition is required - prefer global barriers at all
/// other times.
///
/// In general it is better to use image barriers with `Layout::Optimal`
/// than it is to use global barriers with images using the
/// `RawLayout::General` layout.
///
/// Access types are defined in the same way as for a global memory barrier, but
/// they only affect the image subresource range identified by `image` and
/// `range`, rather than all resources.
///
/// An image barrier defining a queue ownership transfer needs to be executed
/// twice - once by a queue in the source queue family, and then once again by a
/// queue in the destination queue family, with a semaphore guaranteeing
/// execution order between them.
///
/// If `old_layout` is empty, the contents of the image become
/// undefined after the barrier is executed, which can result in a performance
/// boost over attempting to preserve the contents. This is particularly useful
/// for transient images where the contents are going to be immediately overwritten.
/// A good example of when to use this is when an application re-uses a presented
/// image after acquiring the next swap chain image.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ImageMemoryBarrier<'a> {
    pub image: &'a Image,
    pub prev_access: Access,
    pub next_access: Access,
    pub old_layout: Option<SyncLayout>,
    pub new_layout: SyncLayout,
    pub family_transfer: Option<(u32, u32)>,
    pub range: SubresourceRange,
}

impl<'a> From<&'a LayoutTransition<'a>> for ImageMemoryBarrier<'a> {
    fn from(value: &'a LayoutTransition<'a>) -> Self {
        ImageMemoryBarrier {
            image: value.image,
            prev_access: value.prev_access,
            next_access: value.next_access,
            old_layout: value.old_layout,
            new_layout: value.new_layout,
            family_transfer: None,
            range: value.range,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageSubresourceRange {
    pub image: Image,
    pub range: SubresourceRange,
}

/// Image region with access mask,
/// specifying how it may be accessed "before".
///
/// Note that "before" is loosely defined
/// as whatever previous owners do.
/// Which should be translated to "earlier GPU operations"
/// but this crate doesn't attempt to enforce that.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageSubresourceState {
    pub subresource: ImageSubresourceRange,
    pub prev_access: Access,
    pub old_layout: Option<SyncLayout>,
    pub family: Ownership,
}

impl ImageSubresourceState {
    ///
    pub fn access<'a>(
        &'a mut self,
        next_access: Access,
        new_layout: SyncLayout,
        queue: QueueId,
        encoder: &mut Encoder<'a>,
    ) -> &'a Self {
        match self.family {
            Ownership::NotOwned => {
                encoder.image_barriers(encoder.scope().to_scope([ImageMemoryBarrier {
                    image: &self.subresource.image,
                    prev_access: self.prev_access,
                    next_access,
                    old_layout: self.old_layout,
                    new_layout,
                    family_transfer: None,
                    range: self.subresource.range,
                }]))
            }
            Ownership::Owned { family } => {
                assert_eq!(family, queue.family, "Wrong queue family owns the buffer");

                encoder.image_barriers(encoder.scope().to_scope([ImageMemoryBarrier {
                    image: &self.subresource.image,
                    prev_access: self.prev_access,
                    next_access,
                    old_layout: self.old_layout,
                    new_layout,
                    family_transfer: None,
                    range: self.subresource.range,
                }]))
            }
            Ownership::Transition { from, to } => {
                assert_eq!(
                    to, queue.family,
                    "Image is being transitioned to wrong queue family"
                );

                encoder.image_barriers(encoder.scope().to_scope([ImageMemoryBarrier {
                    image: &self.subresource.image,
                    prev_access: self.prev_access,
                    next_access,
                    old_layout: self.old_layout,
                    new_layout,
                    family_transfer: Some((from, to)),
                    range: self.subresource.range,
                }]))
            }
        }
        self.prev_access = next_access;
        self.old_layout = Some(new_layout);
        self
    }

    ///
    pub fn overwrite<'a>(
        &'a mut self,
        next_access: Access,
        new_layout: SyncLayout,
        queue: QueueId,
        encoder: &mut Encoder<'a>,
    ) -> &'a ImageSubresourceRange {
        encoder.image_barriers(encoder.scope().to_scope([ImageMemoryBarrier {
            image: &self.subresource.image,
            prev_access: Access::None,
            next_access,
            old_layout: None,
            new_layout,
            family_transfer: None,
            range: self.subresource.range,
        }]));
        self.family = Ownership::Owned {
            family: queue.family,
        };
        self.prev_access = next_access;
        self.old_layout = Some(new_layout);
        &self.subresource
    }
}
