pub use crate::backend::Framebuffer;
use {
    crate::{
        render_pass::{RenderPass, RENDERPASS_SMALLVEC_ATTACHMENTS},
        view::ImageView,
        Extent2d,
    },
    smallvec::SmallVec,
};

#[derive(Clone, Debug, Hash)]
pub struct FramebufferInfo {
    pub render_pass: RenderPass,
    pub views: SmallVec<[ImageView; RENDERPASS_SMALLVEC_ATTACHMENTS]>,
    pub extent: Extent2d,
}
