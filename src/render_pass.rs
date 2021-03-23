pub use crate::backend::RenderPass;
use {
    crate::{
        format::Format,
        image::{Layout, Samples},
        stage::PipelineStageFlags,
    },
    smallvec::SmallVec,
};

/// Upper limit for smallvec array size for attachments.
pub const RENDERPASS_SMALLVEC_ATTACHMENTS: usize = 8;

/// Upper limit for smallvec array size for subpasses.
pub const SMALLVEC_SUBPASSES: usize = 4;

/// Defines render pass, its attachments and one implicit subpass.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct RenderPassInfo {
    /// Describes attachments used in the render pass.
    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "SmallVec::is_empty", default)
    )]
    pub attachments:
        SmallVec<[AttachmentInfo; RENDERPASS_SMALLVEC_ATTACHMENTS]>,
    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "SmallVec::is_empty", default)
    )]
    pub subpasses: SmallVec<[Subpass; SMALLVEC_SUBPASSES]>,
    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "SmallVec::is_empty", default)
    )]
    pub dependencies: SmallVec<[SubpassDependency; SMALLVEC_SUBPASSES]>,
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
    pub load_op: AttachmentLoadOp,
    pub store_op: AttachmentStoreOp,

    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub initial_layout: Option<Layout>,
    pub final_layout: Layout,
}

/// Specifies how render pass treats attachment content at the beginning.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum AttachmentLoadOp {
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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum AttachmentStoreOp {
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct Subpass {
    /// Indices of attachments that are used as color attachments in this
    /// subpass.
    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "SmallVec::is_empty", default)
    )]
    pub colors: SmallVec<[usize; RENDERPASS_SMALLVEC_ATTACHMENTS]>,

    /// Index of an attachment that is used as depth attachmetn in this
    /// subpass.
    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub depth: Option<usize>,
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
    pub src: Option<usize>,

    /// Index of the second subpass in dependency.
    /// `None` for defining dependency between subpass and commands after
    /// render pass.
    ///
    /// Both `src` and `dst` cannot be `None`.
    #[cfg_attr(
        feature = "serde-1",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub dst: Option<usize>,

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

#[cfg(feature = "serde-1")]
fn is_default<T: Default + Eq>(value: &T) -> bool {
    *value == T::default()
}
