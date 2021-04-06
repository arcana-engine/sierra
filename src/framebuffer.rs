pub use crate::backend::Framebuffer;
use crate::{
    format::Format,
    image::{Image, ImageInfo, ImageUsage, Samples, Samples1, SubresourceRange},
    render_pass::RenderPass,
    view::{ImageView, ImageViewInfo, ImageViewKind},
    CreateImageError, CreateRenderPassError, Device, Extent2d, OutOfMemory,
};

#[derive(Clone, Debug, Hash)]
pub struct FramebufferInfo {
    pub render_pass: RenderPass,
    pub attachments: Vec<ImageView>,
    pub extent: Extent2d,
}

/// Trait for types that can be used for attachments in declarative render-pass.
pub trait Attachment {
    /// Samples for this attachment.
    /// None if unspecified.
    fn samples(&self) -> Option<Samples>;

    /// Format for this attachment.
    fn format(&self) -> Format;

    fn eq(&self, view: &ImageView) -> bool;

    fn max_extent(&self) -> Extent2d;

    fn get_view(
        &self,
        device: &Device,
        usage: ImageUsage,
        extent: Extent2d,
    ) -> Result<ImageView, CreateImageError>;
}

impl Attachment for ImageView {
    fn samples(&self) -> Option<Samples> {
        Some(self.info().image.info().samples)
    }

    fn format(&self) -> Format {
        self.info().image.info().format
    }

    fn eq(&self, view: &ImageView) -> bool {
        *self == *view
    }

    fn max_extent(&self) -> Extent2d {
        let mut extent = self.info().image.info().extent.into_2d();
        extent.width >>= self.info().range.first_level;
        extent.height >>= self.info().range.first_level;
        extent
    }

    fn get_view(
        &self,
        _device: &Device,
        usage: ImageUsage,
        mut extent: Extent2d,
    ) -> Result<ImageView, CreateImageError> {
        assert_eq!(self.info().range.layer_count, 1);
        assert_eq!(self.info().range.level_count, 1);

        extent.width <<= self.info().range.first_level;
        extent.height <<= self.info().range.first_level;

        assert!(self.info().image.info().usage.contains(usage));
        assert!(self.info().image.info().extent.into_2d() >= extent);

        Ok(self.clone())
    }
}

impl Attachment for Image {
    fn samples(&self) -> Option<Samples> {
        Some(self.info().samples)
    }

    fn format(&self) -> Format {
        self.info().format
    }

    fn max_extent(&self) -> Extent2d {
        self.info().extent.into_2d()
    }

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

    fn get_view(
        &self,
        device: &Device,
        usage: ImageUsage,
        extent: Extent2d,
    ) -> Result<ImageView, CreateImageError> {
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
        })?;

        Ok(view)
    }
}

impl Attachment for Format {
    fn samples(&self) -> Option<Samples> {
        None
    }

    fn format(&self) -> Format {
        *self
    }

    fn eq(&self, view: &ImageView) -> bool {
        *self == view.info().image.info().format
    }

    fn max_extent(&self) -> Extent2d {
        Extent2d {
            width: !0,
            height: !0,
        }
    }

    fn get_view(
        &self,
        device: &Device,
        usage: ImageUsage,
        extent: Extent2d,
    ) -> Result<ImageView, CreateImageError> {
        let image = device.create_image(ImageInfo {
            extent: extent.into(),
            format: *self,
            levels: 1,
            layers: 1,
            samples: Samples1,
            usage,
        })?;

        let view = device.create_image_view(ImageViewInfo {
            view_kind: ImageViewKind::D2,
            range: SubresourceRange {
                aspect: image.info().format.aspect_flags(),
                first_level: 0,
                level_count: 1,
                first_layer: 0,
                layer_count: 1,
            },
            image: image.clone(),
        })?;

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

impl From<CreateImageError> for FramebufferError {
    fn from(err: CreateImageError) -> Self {
        match err {
            CreateImageError::OutOfMemory { source } => FramebufferError::OutOfMemory { source },
            CreateImageError::Unsupported { info } => FramebufferError::Unsupported { info },
        }
    }
}
