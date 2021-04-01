pub use crate::backend::Framebuffer;
use {
    crate::{
        format::Format,
        image::{Image, ImageInfo, ImageUsage, Samples1},
        render_pass::{RenderPass, RENDERPASS_SMALLVEC_ATTACHMENTS},
        view::{ImageView, ImageViewInfo},
        CreateImageError, Device, Extent2d, OutOfMemory,
    },
    smallvec::SmallVec,
};

#[derive(Clone, Debug, Hash)]
pub struct FramebufferInfo {
    pub render_pass: RenderPass,
    pub views: SmallVec<[ImageView; RENDERPASS_SMALLVEC_ATTACHMENTS]>,
    pub extent: Extent2d,
}

///
pub trait ViewForFramebuffer {
    fn make_view_for_framebuffer(
        &self,
        device: &Device,
        extent: Extent2d,
    ) -> Result<ImageView, CreateImageError>;
}

impl ViewForFramebuffer for Format {
    fn make_view_for_framebuffer(
        &self,
        device: &Device,
        extent: Extent2d,
    ) -> Result<ImageView, CreateImageError> {
        let image = device.create_image(ImageInfo {
            extent: extent.into(),
            format: *self,
            levels: 1,
            layers: 1,
            samples: Samples1,
            usage: if self.is_color() {
                ImageUsage::COLOR_ATTACHMENT
            } else {
                ImageUsage::DEPTH_STENCIL_ATTACHMENT
            },
        })?;
        let view = device.create_image_view(ImageViewInfo::new(image))?;
        Ok(view)
    }
}

impl ViewForFramebuffer for Image {
    fn make_view_for_framebuffer(
        &self,
        device: &Device,
        extent: Extent2d,
    ) -> Result<ImageView, CreateImageError> {
        debug_assert!(self.info().extent.into_2d() >= extent);
        let view = device.create_image_view(ImageViewInfo::new(self.clone()))?;
        Ok(view)
    }
}

impl ViewForFramebuffer for ImageView {
    fn make_view_for_framebuffer(
        &self,
        _device: &Device,
        extent: Extent2d,
    ) -> Result<ImageView, CreateImageError> {
        debug_assert!(self.info().image.info().extent.into_2d() >= extent);
        Ok(self.clone())
    }
}
