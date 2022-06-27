pub use crate::backend::Framebuffer;
use crate::{
    format::Format,
    image::{Image, ImageInfo, ImageUsage, Samples, Samples1, SubresourceRange},
    render_pass::RenderPass,
    view::{ComponentMapping, ImageView, ImageViewInfo, ImageViewKind},
    CreateRenderPassError, Device, Extent2, OutOfMemory,
};

/// Defines [`Framebuffer`] state.
/// Can be used to [`Device::create_framebuffer`].
#[derive(Clone, Debug, Hash)]
pub struct FramebufferInfo {
    /// [`RenderPass`] with which framebuffer can be used.
    pub render_pass: RenderPass,

    /// [`ImageView`]s that will be used as attachments for render pass started with framebuffer.
    pub attachments: Vec<ImageView>,

    /// Specifies dimensions of the rendering operations over framebuffer.
    pub extent: Extent2,
}

/// Trait for types that can be used for attachments in declarative render-pass.
pub trait Attachment {
    /// Samples for this attachment.
    /// None if unspecified.
    fn samples(&self) -> Option<Samples>;

    /// Format for this attachment.
    fn format(&self) -> Format;

    /// Returns if this attachment is equivalent to image view.
    ///
    /// They are considered equivalent if replacing image
    /// view with new one from this attachment will make no difference.
    ///
    /// Mainly there are thee possibilities:
    /// 1. Attachment is `ImageView`, in which case they equivalent only if same.
    /// 2. Attachment is `Image`, in which case any `ImageView` with same sub-resource from this image is equivalent.
    /// 3. Attachment is just bunch of properties (e.g. `Format`), in which case any `ImageView` with matching properties is equivalent.
    fn eq(&self, view: &ImageView) -> bool;

    /// Maximum extend of the image view that can be make for this attachment.
    fn max_extent(&self) -> Extent2;

    /// Returns image view with specified usage and extent for this attachment.
    fn get_view(
        &self,
        device: &Device,
        usage: ImageUsage,
        extent: Extent2,
    ) -> Result<ImageView, OutOfMemory>;
}

impl Attachment for ImageView {
    #[inline]
    fn samples(&self) -> Option<Samples> {
        Some(self.info().image.info().samples)
    }

    #[inline]
    fn format(&self) -> Format {
        self.info().image.info().format
    }

    #[inline]
    fn eq(&self, view: &ImageView) -> bool {
        *self == *view
    }

    #[inline]
    fn max_extent(&self) -> Extent2 {
        let mut extent = self.info().image.info().extent.into_2d();
        extent.width >>= self.info().range.first_level;
        extent.height >>= self.info().range.first_level;
        extent
    }

    #[inline]
    fn get_view(
        &self,
        _device: &Device,
        usage: ImageUsage,
        extent: Extent2,
    ) -> Result<ImageView, OutOfMemory> {
        assert_eq!(self.info().range.layer_count, 1);
        assert_eq!(self.info().range.level_count, 1);

        assert!(self.info().image.info().usage.contains(usage));
        assert!(self.max_extent() >= extent);

        Ok(self.clone())
    }
}

impl Attachment for Image {
    #[inline]
    fn samples(&self) -> Option<Samples> {
        Some(self.info().samples)
    }

    #[inline]
    fn format(&self) -> Format {
        self.info().format
    }

    #[inline]
    fn max_extent(&self) -> Extent2 {
        self.info().extent.into_2d()
    }

    #[inline]
    fn eq(&self, view: &ImageView) -> bool {
        *self == view.info().image
            && ImageViewKind::D2 == view.info().view_kind
            && SubresourceRange {
                aspect: self.info().format.aspect_flags(),
                first_level: 0,
                level_count: 1,
                first_layer: 0,
                layer_count: 1,
            } == view.info().range
    }

    #[inline]
    fn get_view(
        &self,
        device: &Device,
        usage: ImageUsage,
        extent: Extent2,
    ) -> Result<ImageView, OutOfMemory> {
        assert!(self.info().usage.contains(usage));
        assert!(self.info().extent.into_2d() >= extent);

        let view = device.create_image_view(ImageViewInfo {
            view_kind: ImageViewKind::D2,
            range: SubresourceRange {
                aspect: self.info().format.aspect_flags(),
                first_level: 0,
                level_count: 1,
                first_layer: 0,
                layer_count: 1,
            },
            image: self.clone(),
            mapping: ComponentMapping::default(),
        })?;

        Ok(view)
    }
}

impl Attachment for Format {
    #[inline]
    fn samples(&self) -> Option<Samples> {
        None
    }

    #[inline]
    fn format(&self) -> Format {
        *self
    }

    #[inline]
    fn eq(&self, view: &ImageView) -> bool {
        *self == view.info().image.info().format
    }

    #[inline]
    fn max_extent(&self) -> Extent2 {
        Extent2::new(u32::MAX, u32::MAX)
    }

    #[inline]
    fn get_view(
        &self,
        device: &Device,
        usage: ImageUsage,
        extent: Extent2,
    ) -> Result<ImageView, OutOfMemory> {
        let image = device.create_image(ImageInfo {
            extent: extent.into(),
            format: *self,
            levels: 1,
            layers: 1,
            samples: Samples1,
            usage,
        })?;

        let view = device.create_image_view(ImageViewInfo::new(image))?;

        Ok(view)
    }
}

#[derive(Clone, Copy, Debug, thiserror::Error)]
pub enum FramebufferError {
    #[error(transparent)]
    OutOfMemory {
        #[from]
        source: OutOfMemory,
    },

    #[error(
        "Subpass {subpass} attachment index {attachment} for color attachment {index} is out of bounds"
    )]
    ColorAttachmentReferenceOutOfBound {
        subpass: usize,
        index: usize,
        attachment: u32,
    },

    #[error(
        "Subpass {subpass} attachment index {attachment} for depth attachment is out of bounds"
    )]
    DepthAttachmentReferenceOutOfBound { subpass: usize, attachment: u32 },

    #[error("Parameters combination `{info:?}` is unsupported")]
    Unsupported { info: ImageInfo },
}

impl From<CreateRenderPassError> for FramebufferError {
    #[inline]
    fn from(err: CreateRenderPassError) -> Self {
        match err {
            CreateRenderPassError::OutOfMemory { source } => {
                FramebufferError::OutOfMemory { source }
            }
            CreateRenderPassError::ColorAttachmentReferenceOutOfBound {
                subpass,
                index,
                attachment,
            } => FramebufferError::ColorAttachmentReferenceOutOfBound {
                subpass,
                index,
                attachment,
            },
            CreateRenderPassError::DepthAttachmentReferenceOutOfBound {
                subpass,
                attachment,
            } => FramebufferError::DepthAttachmentReferenceOutOfBound {
                subpass,
                attachment,
            },
        }
    }
}
