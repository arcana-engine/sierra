pub use crate::backend::RenderPass;
use crate::{
    encode::{Encoder, RenderPassEncoder},
    format::Format,
    framebuffer::FramebufferError,
    image::{Layout, Samples},
    stage::PipelineStageFlags,
    Device, ImageView, OutOfMemory, Rect2d,
};

/// Defines render pass, its attachments and one implicit subpass.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct RenderPassInfo {
    /// Describes attachments used in the render pass.
    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "Vec::is_empty", default)
    )]
    pub attachments: Vec<AttachmentInfo>,
    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "Vec::is_empty", default)
    )]
    pub subpasses: Vec<Subpass>,
    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "Vec::is_empty", default)
    )]
    pub dependencies: Vec<SubpassDependency>,
}

/// Describes one attachment of a render pass.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct AttachmentInfo {
    pub format: Format,

    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "is_default", default)
    )]
    pub samples: Samples,

    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "is_default", default)
    )]
    pub load_op: LoadOp,

    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "is_default", default)
    )]
    pub store_op: StoreOp,

    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub initial_layout: Option<Layout>,

    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "is_default", default)
    )]
    pub final_layout: Layout,
}

/// Specifies how render pass treats attachment content at the beginning.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum LoadOp {
    /// Render pass will load this attachment content before first subpass that
    /// access this attachment starts.
    Load,

    /// Render pass will clear this attachment content before first subpass
    /// that access this attachment starts. Value to which attachment
    /// should be cleared must be provided in `Encoder::begin_render_pass`
    /// call.
    Clear,

    /// Render pass will not attempt to load attachment content or clear it -
    /// basically no-op. Attachment content visible to read operations
    /// inside render pass is undefined before it is written.
    ///
    /// This is fastest variant suitable when old content can be discarded and
    /// whole attachment is going to be written by operations in render
    /// pass, or only written parts are later read.
    DontCare,
}

impl Default for LoadOp {
    fn default() -> Self {
        Self::DontCare
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum StoreOp {
    /// Render pass will store this attachment content after last subpass that
    /// access this attachment finishes.
    Store,

    /// Render pass will not attempt to store attachment content - basically
    /// no-op. Attachment content visible to read operations after render
    /// pass is undefined before it is written.
    ///
    /// This is fastest variant suitable when attchment content produced by
    /// render pass (or before it) won't be used again after render pass.
    DontCare,
}

impl Default for StoreOp {
    fn default() -> Self {
        Self::DontCare
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct Subpass {
    /// Indices of attachments that are used as color attachments in this
    /// subpass.
    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "Vec::is_empty", default)
    )]
    pub colors: Vec<(u32, Layout)>,

    /// Index of an attachment that is used as depth attachment in this
    /// subpass.
    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub depth: Option<(u32, Layout)>,
}

/// Defines memory dependency between two subpasses
/// or subpass and commands outside render pass.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct SubpassDependency {
    /// Index of the first subpass in dependency.
    /// `None` for defining dependency between commands before render pass and
    /// subpass.
    ///
    /// Both `src` and `dst` cannot be `None`.
    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub src: Option<u32>,

    /// Index of the second subpass in dependency.
    /// `None` for defining dependency between subpass and commands after
    /// render pass.
    ///
    /// Both `src` and `dst` cannot be `None`.
    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub dst: Option<u32>,

    /// Stages of the first subpass that will be synchronized
    /// with stages for second subpass specified in `dst_stages`.
    pub src_stages: PipelineStageFlags,

    /// Stages of the second subpass that will be synchronized
    /// with stages for first subpass specified in `src_stages`.
    pub dst_stages: PipelineStageFlags,
}

/// Value for attachment load clear operation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClearValue {
    Color(f32, f32, f32, f32),
    DepthStencil(f32, u32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClearColor(pub f32, pub f32, pub f32, pub f32);

impl From<ClearColor> for ClearValue {
    fn from(ClearColor(r, g, b, a): ClearColor) -> Self {
        ClearValue::Color(r, g, b, a)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClearDepth(pub f32);

impl From<ClearDepth> for ClearValue {
    fn from(ClearDepth(d): ClearDepth) -> Self {
        ClearValue::DepthStencil(d, 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClearStencil(pub u32);

impl From<ClearStencil> for ClearValue {
    fn from(ClearStencil(s): ClearStencil) -> Self {
        ClearValue::DepthStencil(0.0, s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClearDepthStencil(pub f32, pub u32);

impl From<ClearDepthStencil> for ClearValue {
    fn from(ClearDepthStencil(d, s): ClearDepthStencil) -> Self {
        ClearValue::DepthStencil(d, s)
    }
}

#[cfg(feature = "serde-1")]
fn is_default<T: Default + Eq>(value: &T) -> bool {
    *value == T::default()
}

#[derive(Clone, Copy, Debug, thiserror::Error, PartialEq, Eq)]
pub enum CreateRenderPassError {
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
}

pub trait RenderPassInstance {
    type Input;

    fn begin_render_pass<'a, 'b>(
        &'a mut self,
        input: &Self::Input,
        device: &Device,
        encoder: &'b mut Encoder<'a>,
    ) -> Result<RenderPassEncoder<'b, 'a>, FramebufferError>;
}

pub trait Pass {
    type Instance: RenderPassInstance<Input = Self>;

    fn instance() -> Self::Instance;
}

#[derive(Clone, Debug)]
pub struct RenderingInfo<'a> {
    pub render_area: Rect2d,
    pub colors: &'a [RenderingAttachmentInfo],
    pub depth: Option<RenderingAttachmentInfo>,
    pub stencil: Option<RenderingAttachmentInfo>,
}

#[derive(Clone, Debug)]
pub struct RenderingAttachmentInfo {
    pub image_view: ImageView,
    pub image_layout: Layout,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
    pub clear_value: Option<ClearValue>,
}
