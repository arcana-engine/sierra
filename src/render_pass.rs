pub use crate::backend::RenderPass;
use crate::{
    encode::{Encoder, RenderPassEncoder},
    format::Format,
    framebuffer::FramebufferError,
    image::{Layout, Samples},
    stage::PipelineStages,
    Device, ImageView, OutOfMemory, Rect,
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
pub enum LoadOp<T = ()> {
    /// Render pass will load this attachment content before first subpass that
    /// access this attachment starts.
    Load,

    /// Render pass will clear this attachment content before first subpass
    /// that access this attachment starts. Value to which attachment
    /// should be cleared must be provided in `Encoder::begin_render_pass`
    /// call.
    Clear(T),

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
    pub src_stages: PipelineStages,

    /// Stages of the second subpass that will be synchronized
    /// with stages for first subpass specified in `src_stages`.
    pub dst_stages: PipelineStages,
}

/// Value for attachment load clear operation.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum ClearValue {
    Color(f32, f32, f32, f32),
    DepthStencil(f32, u32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct ClearColor(pub f32, pub f32, pub f32, pub f32);

impl From<ClearColor> for ClearValue {
    fn from(ClearColor(r, g, b, a): ClearColor) -> Self {
        ClearValue::Color(r, g, b, a)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct ClearDepth(pub f32);

impl From<ClearDepth> for ClearValue {
    fn from(ClearDepth(d): ClearDepth) -> Self {
        ClearValue::DepthStencil(d, 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
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

#[derive(Clone, Debug, PartialEq)]
pub struct RenderingInfo<'a> {
    pub render_area: Option<Rect>,
    pub colors: &'a [RenderingColorInfo],
    pub depth_stencil: Option<RenderingDepthStencilAttachmentInfo>,
}

impl<'a> RenderingInfo<'a> {
    pub const fn new() -> Self {
        RenderingInfo {
            render_area: None,
            colors: &[],
            depth_stencil: None,
        }
    }

    pub fn render_area(mut self, render_area: Rect) -> Self {
        self.render_area = Some(render_area);
        self
    }

    pub fn colors(mut self, colors: &'a [RenderingColorInfo]) -> Self {
        self.colors = colors;
        self
    }

    pub fn color(mut self, color: &'a RenderingColorInfo) -> Self {
        self.colors = std::slice::from_ref(color);
        self
    }

    pub fn depth(mut self, depth: RenderingDepthInfo) -> Self {
        self.depth_stencil = Some(depth.into());
        self
    }

    pub fn stencil(mut self, stencil: RenderingStencilInfo) -> Self {
        self.depth_stencil = Some(stencil.into());
        self
    }

    pub fn depth_stencil(mut self, depth_stencil: RenderingDepthStencilInfo) -> Self {
        self.depth_stencil = Some(depth_stencil.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderingColorInfo {
    pub color_view: ImageView,
    pub color_layout: Layout,
    pub color_load_op: LoadOp<ClearColor>,
    pub color_store_op: StoreOp,
}

impl RenderingColorInfo {
    pub fn new(color_view: ImageView) -> Self {
        RenderingColorInfo {
            color_view,
            color_layout: Layout::ColorAttachmentOptimal,
            color_load_op: LoadOp::Load,
            color_store_op: StoreOp::Store,
        }
    }

    pub fn load_op(mut self, load_op: LoadOp<ClearColor>) -> Self {
        self.color_load_op = load_op;
        self
    }

    pub fn store_op(mut self, store_op: StoreOp) -> Self {
        self.color_store_op = store_op;
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.color_layout = layout;
        self
    }

    pub fn load(self) -> Self {
        self.load_op(LoadOp::Load)
    }

    pub fn clear(self, color: ClearColor) -> Self {
        self.load_op(LoadOp::Clear(color))
    }

    pub fn store(self) -> Self {
        self.store_op(StoreOp::Store)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderingDepthInfo {
    pub depth_view: ImageView,
    pub depth_layout: Layout,
    pub depth_load_op: LoadOp<ClearDepth>,
    pub depth_store_op: StoreOp,
}

impl RenderingDepthInfo {
    pub fn new(depth_view: ImageView) -> Self {
        RenderingDepthInfo {
            depth_view,
            depth_layout: Layout::DepthStencilAttachmentOptimal,
            depth_load_op: LoadOp::Load,
            depth_store_op: StoreOp::Store,
        }
    }

    pub fn load_op(mut self, load_op: LoadOp<ClearDepth>) -> Self {
        self.depth_load_op = load_op;
        self
    }

    pub fn store_op(mut self, store_op: StoreOp) -> Self {
        self.depth_store_op = store_op;
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.depth_layout = layout;
        self
    }

    pub fn load(self) -> Self {
        self.load_op(LoadOp::Load)
    }

    pub fn clear(self, depth: ClearDepth) -> Self {
        self.load_op(LoadOp::Clear(depth))
    }

    pub fn store(self) -> Self {
        self.store_op(StoreOp::Store)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderingStencilInfo {
    pub stencil_view: ImageView,
    pub stencil_layout: Layout,
    pub stencil_load_op: LoadOp<ClearStencil>,
    pub stencil_store_op: StoreOp,
}

impl RenderingStencilInfo {
    pub fn new(stencil_view: ImageView) -> Self {
        RenderingStencilInfo {
            stencil_view,
            stencil_layout: Layout::DepthStencilAttachmentOptimal,
            stencil_load_op: LoadOp::Load,
            stencil_store_op: StoreOp::Store,
        }
    }

    pub fn load_op(mut self, load_op: LoadOp<ClearStencil>) -> Self {
        self.stencil_load_op = load_op;
        self
    }

    pub fn store_op(mut self, store_op: StoreOp) -> Self {
        self.stencil_store_op = store_op;
        self
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.stencil_layout = layout;
        self
    }

    pub fn load(self) -> Self {
        self.load_op(LoadOp::Load)
    }

    pub fn clear(self, stencil: ClearStencil) -> Self {
        self.load_op(LoadOp::Clear(stencil))
    }

    pub fn store(self) -> Self {
        self.store_op(StoreOp::Store)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderingDepthStencilInfo {
    pub depth_stencil_view: ImageView,
    pub depth_layout: Layout,
    pub depth_load_op: LoadOp<ClearDepth>,
    pub depth_store_op: StoreOp,
    pub stencil_layout: Layout,
    pub stencil_load_op: LoadOp<ClearStencil>,
    pub stencil_store_op: StoreOp,
}

impl RenderingDepthStencilInfo {
    pub fn new(depth_stencil_view: ImageView) -> Self {
        RenderingDepthStencilInfo {
            depth_stencil_view,
            depth_layout: Layout::DepthStencilAttachmentOptimal,
            stencil_layout: Layout::DepthStencilAttachmentOptimal,
            depth_load_op: LoadOp::Load,
            depth_store_op: StoreOp::Store,
            stencil_load_op: LoadOp::Load,
            stencil_store_op: StoreOp::Store,
        }
    }

    pub fn depth_load_op(mut self, depth_load_op: LoadOp<ClearDepth>) -> Self {
        self.depth_load_op = depth_load_op;
        self
    }

    pub fn depth_store_op(mut self, depth_store_op: StoreOp) -> Self {
        self.depth_store_op = depth_store_op;
        self
    }

    pub fn depth_layout(mut self, depth_layout: Layout) -> Self {
        self.depth_layout = depth_layout;
        self
    }

    pub fn depth_load(self) -> Self {
        self.depth_load_op(LoadOp::Load)
    }

    pub fn depth_clear(self, depth: ClearDepth) -> Self {
        self.depth_load_op(LoadOp::Clear(depth))
    }

    pub fn depth_store(self) -> Self {
        self.depth_store_op(StoreOp::Store)
    }

    pub fn stencil_load_op(mut self, stencil_load_op: LoadOp<ClearStencil>) -> Self {
        self.stencil_load_op = stencil_load_op;
        self
    }

    pub fn stencil_store_op(mut self, stencil_store_op: StoreOp) -> Self {
        self.stencil_store_op = stencil_store_op;
        self
    }

    pub fn stencil_layout(mut self, stencil_layout: Layout) -> Self {
        self.stencil_layout = stencil_layout;
        self
    }

    pub fn stencil_load(self) -> Self {
        self.stencil_load_op(LoadOp::Load)
    }

    pub fn stencil_clear(self, stencil: ClearStencil) -> Self {
        self.stencil_load_op(LoadOp::Clear(stencil))
    }

    pub fn stencil_store(self) -> Self {
        self.stencil_store_op(StoreOp::Store)
    }

    pub fn depth_stencil_layout(mut self, depth_stencil_layout: Layout) -> Self {
        self.depth_layout = depth_stencil_layout;
        self.stencil_layout = depth_stencil_layout;
        self
    }

    pub fn depth_stencil_clear(mut self, depth_stencil: ClearDepthStencil) -> Self {
        self.depth_load_op = LoadOp::Clear(ClearDepth(depth_stencil.0));
        self.stencil_load_op = LoadOp::Clear(ClearStencil(depth_stencil.1));
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderingDepthStencilAttachmentInfo {
    pub depth_stencil_view: ImageView,
    pub depth: Option<(LoadOp<ClearDepth>, StoreOp, Layout)>,
    pub stencil: Option<(LoadOp<ClearStencil>, StoreOp, Layout)>,
}

impl From<RenderingDepthInfo> for RenderingDepthStencilAttachmentInfo {
    #[inline]
    fn from(info: RenderingDepthInfo) -> Self {
        RenderingDepthStencilAttachmentInfo {
            depth_stencil_view: info.depth_view,
            depth: Some((info.depth_load_op, info.depth_store_op, info.depth_layout)),
            stencil: None,
        }
    }
}

impl From<RenderingStencilInfo> for RenderingDepthStencilAttachmentInfo {
    #[inline]
    fn from(info: RenderingStencilInfo) -> Self {
        RenderingDepthStencilAttachmentInfo {
            depth_stencil_view: info.stencil_view,
            depth: None,
            stencil: Some((
                info.stencil_load_op,
                info.stencil_store_op,
                info.stencil_layout,
            )),
        }
    }
}

impl From<RenderingDepthStencilInfo> for RenderingDepthStencilAttachmentInfo {
    #[inline]
    fn from(info: RenderingDepthStencilInfo) -> Self {
        RenderingDepthStencilAttachmentInfo {
            depth_stencil_view: info.depth_stencil_view,
            depth: Some((info.depth_load_op, info.depth_store_op, info.depth_layout)),
            stencil: Some((
                info.stencil_load_op,
                info.stencil_store_op,
                info.stencil_layout,
            )),
        }
    }
}
