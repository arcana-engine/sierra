use std::hash::{Hash, Hasher};

use {
    super::PipelineLayout,
    crate::{
        format::Format,
        render_pass::RenderPass,
        sampler::CompareOp,
        shader::{FragmentShader, VertexShader},
        Device, Extent2d, Extent3d, Offset2d, OutOfMemory,
    },
    ordered_float::OrderedFloat,
};

pub use {
    self::State::{Dynamic, Static},
    crate::backend::GraphicsPipeline,
};

/// Wrapper for pipeline states that can be either static or dynamic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum State<T> {
    /// Static state value.
    Static { value: T },

    /// Dynamic state marker,
    /// When some state is dynamic then it must be set via
    /// specific command before rendering.
    Dynamic,
}

impl<T> State<T> {
    pub fn is_dynamic(&self) -> bool {
        matches!(self, Self::Dynamic)
    }

    pub fn dynamic() -> Self {
        Self::Dynamic
    }
}

impl<T> From<T> for State<T> {
    fn from(value: T) -> Self {
        State::Static { value }
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct Bounds {
    pub offset: f32,
    pub size: f32,
}

impl PartialEq for Bounds {
    fn eq(&self, other: &Self) -> bool {
        OrderedFloat(self.offset) == OrderedFloat(other.offset)
            && OrderedFloat(self.size) == OrderedFloat(other.size)
    }
}

impl Eq for Bounds {}

impl Hash for Bounds {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        Hash::hash(&OrderedFloat(self.offset), hasher);
        Hash::hash(&OrderedFloat(self.size), hasher);
    }
}

/// Graphics pipeline state definition.
/// Fields are ordered to match pipeline stages, including fixed functions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GraphicsPipelineInfo {
    /// For each vertex buffer specifies how it is bound.
    pub vertex_bindings: Vec<VertexInputBinding>,

    /// For each vertex attribute specifies where is the data is read from.
    pub vertex_attributes: Vec<VertexInputAttribute>,

    /// Input primitives topology.
    pub primitive_topology: PrimitiveTopology,

    /// If `True` then special marker index value `!0` will restart
    /// primitive assembly next index, discarding any incomplete primitives.
    pub primitive_restart_enable: bool,

    /// Vertex shader for pipeline.
    pub vertex_shader: VertexShader,

    /// Primitives rasteriazation behavior.
    /// If `None` then no rasterization is performed.
    /// This is useful when only side-effects of earlier stages are needed.
    pub rasterizer: Option<Rasterizer>,

    /// Pipeline layout.
    pub layout: PipelineLayout,

    /// Render pass within which this pipeline will be executed.
    pub render_pass: RenderPass,

    /// Subpass of the render pass within which this pipeline will be executed.
    pub subpass: u32,
}

/// Vertex buffer binding bahavior.
/// Controls what subrange corresponds for vertex X of instance Y.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct VertexInputBinding {
    /// Controls iteration frequency.
    pub rate: VertexInputRate,

    /// Size of the iteration step.
    pub stride: u32,
}

/// Controls vertex input iteration frequency.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum VertexInputRate {
    /// Iterate value once per vertex.
    /// Repeat for each instance.
    Vertex,

    /// Iterate value once per instance.
    /// All vertices of an instance will use same value.
    Instance,
}

/// Vertex sub-range to attribute mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct VertexInputAttribute {
    /// Attribute index.
    /// Each index must appear at most once in `VertexInput::attributes` array.
    pub location: u32,

    /// Attribute format.
    /// Controls how attribute data is interpreted.
    /// Must match attribute type in vertex shader.
    pub format: Format,

    /// Index of vertex buffer from which attribute data is read from.
    pub binding: u32,

    /// Offset of this attribute in the vertex buffer sub-range.
    pub offset: u32,
}

/// Topology of primitives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum PrimitiveTopology {
    /// Vertices are assembled into points.
    /// Each vertex form one point primitive.
    ///
    /// # Example
    ///
    /// Vertirces `a`, `b`, `c`, `d` will form points `a`, `b`, `c`, `d`.
    PointList,

    /// Vertices are assembled into lines.
    /// Each separate pair of vertices forms one line primitive.
    ///
    /// # Example
    ///
    /// Vertirces `a`, `b`, `c`, `d` will form lines `a, b` and `c, d`.
    LineList,

    /// Vertices are assembled into lines.
    /// Each pair of vertices forms one line primitive.
    ///
    /// # Example
    ///
    /// Vertirces `a`, `b`, `c`, `d` will form lines `a, b`, `b, c` and `c, d`.
    LineStrip,

    /// Vertices are assempled into triangles.
    /// Each separate triplet of vertices forms one triangle primitive.
    ///
    /// # Example
    ///
    /// Vertirces `a`, `b`, `c`, `d`, `e`, `f` will form triangles `a, b, c`
    /// and `d, e, f`.
    TriangleList,

    /// Vertices are assempled into triangles.
    /// Each triplet of vertices forms one triangle primitive.
    ///
    /// # Example
    ///
    /// Vertirces `a`, `b`, `c`, `d`, `e`, `f` will form triangles `a, b, c`,
    /// `b, c, d`, `c, d, e` and `d, e, f`.
    TriangleStrip,

    /// Vertices are assempled into triangles.
    /// First vertex is shared with all triangles.
    /// Then each pair and shared vertex form one triangle primitive.
    ///
    ///
    /// # Example
    ///
    /// Vertirces `a`, `b`, `c`, `d`, `e`, `f` will form triangles `a, b, c`,
    /// `a, c, d`, `a, d, e` and `a, e, f`.
    TriangleFan,
}

impl Default for PrimitiveTopology {
    fn default() -> Self {
        PrimitiveTopology::TriangleList
    }
}

/// Viewport transformation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct Viewport {
    /// Viewport bounds along X (horizontal) axis.
    pub x: Bounds,

    /// Viewport bounds along Y (vertical) axis.
    pub y: Bounds,

    /// Viewport bounds along Z (depth) axis.
    pub z: Bounds,
}

impl From<Extent2d> for Viewport {
    fn from(extent: Extent2d) -> Self {
        Viewport {
            x: Bounds {
                offset: 0.0,
                size: extent.width as f32,
            },
            y: Bounds {
                offset: 0.0,
                size: extent.height as f32,
            },
            z: Bounds {
                offset: 0.0,
                size: 1.0,
            },
        }
    }
}

impl From<Extent3d> for Viewport {
    fn from(extent: Extent3d) -> Self {
        Viewport {
            x: Bounds {
                offset: 0.0,
                size: extent.width as f32,
            },
            y: Bounds {
                offset: 0.0,
                size: extent.height as f32,
            },
            z: Bounds {
                offset: 0.0,
                size: extent.depth as f32,
            },
        }
    }
}

impl From<Extent2d> for State<Viewport> {
    fn from(extent: Extent2d) -> Self {
        State::Static {
            value: extent.into(),
        }
    }
}

impl From<Extent3d> for State<Viewport> {
    fn from(extent: Extent3d) -> Self {
        State::Static {
            value: extent.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct Rect2d {
    pub offset: Offset2d,
    pub extent: Extent2d,
}

impl From<Extent2d> for Rect2d {
    fn from(extent: Extent2d) -> Self {
        Rect2d {
            offset: Offset2d::ZERO,
            extent,
        }
    }
}

impl From<Extent2d> for State<Rect2d> {
    fn from(extent: Extent2d) -> Self {
        State::Static {
            value: extent.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Rasterizer {
    /// Rendering viewport transformation.
    /// Determines how vertex coordinates are transformed to framebuffer
    /// coordinates.
    pub viewport: State<Viewport>,

    /// Scissors for the viewport.
    /// Determines bounds for scissor test.
    /// If the test fails for generated fragment that fragment is discared.
    pub scissor: State<Rect2d>,

    /// Should fragments out of bounds on Z axis are clamped or discared.
    /// If `true` - fragments are clamped. This also disables primitive
    /// clipping. Otherwise they are clipped.
    ///
    /// If `DepthClamp` feature is not enabled this value must be `false`.
    pub depth_clamp: bool,

    /// How polygon front face is determined.
    pub front_face: FrontFace,

    /// How polygons are culled before rasterization.
    pub culling: Option<Culling>,

    /// How polygons are rasterized.
    /// See `PolygonMode` for description.
    ///
    /// If `fillModeNonSolid` is not enabled this value must be
    /// `PolygonMode::Fill`.
    pub polygon_mode: PolygonMode,

    /// Depth test and operations.
    pub depth_test: Option<DepthTest>,

    /// Stencil test and operations.
    pub stencil_tests: Option<StencilTests>,

    /// Depth-bounds test.
    pub depth_bounds: Option<State<Bounds>>,

    /// Fragment shader used by the pipeline.
    pub fragment_shader: Option<FragmentShader>,

    /// Attachment color blending.
    pub color_blend: ColorBlend,
}

impl Default for Rasterizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Rasterizer {
    pub const fn new() -> Self {
        Rasterizer {
            viewport: Dynamic,
            scissor: Dynamic,
            depth_clamp: false,
            front_face: FrontFace::Clockwise,
            culling: None,
            polygon_mode: PolygonMode::Fill,
            depth_test: None,
            stencil_tests: None,
            depth_bounds: None,
            fragment_shader: None,
            color_blend: ColorBlend::Blending {
                blending: Some(Blending {
                    color_src_factor: BlendFactor::SrcAlpha,
                    color_dst_factor: BlendFactor::OneMinusSrcAlpha,
                    color_op: BlendOp::Add,
                    alpha_src_factor: BlendFactor::One,
                    alpha_dst_factor: BlendFactor::OneMinusSrcAlpha,
                    alpha_op: BlendOp::Add,
                }),
                write_mask: ComponentMask::RGBA,
                constants: Static {
                    value: [0.0, 0.0, 0.0, 0.0],
                },
            },
        }
    }
}

/// Polygon front face definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum FrontFace {
    /// Polygon which has vertices ordered in clockwise
    /// from some point of view is front faced to that point.
    Clockwise,

    /// Polygon which has vertices ordered in counter-clockwise
    /// from some point of view is front faced to that point.
    CounterClockwise,
}

impl Default for FrontFace {
    fn default() -> Self {
        FrontFace::Clockwise
    }
}

/// Polygione culling mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum Culling {
    /// Front facing polygons are culled.
    Front,

    /// Back facing polygons are culled.
    Back,

    /// All polygons are culled.
    FrontAndBack,
}

/// PolygonMode rasterization mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum PolygonMode {
    /// Whole polygon is rasterized.
    /// That is, fragments are generated to cover all points inside the
    /// polygon.
    Fill,

    /// Edges are rasterized as lines.
    /// That is, fragments are generated to cover all points on polygon edges.
    Line,

    /// Vertices are rasterized as points.
    /// That is, fragments are generated to cover only points that are polygon
    /// vertices.
    Point,
}

impl Default for PolygonMode {
    fn default() -> Self {
        PolygonMode::Fill
    }
}

/// Defines how depth testing is performed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct DepthTest {
    /// Comparison operation between value stored in depth buffer and
    /// fragment's depth.
    pub compare: CompareOp,

    /// Whether fragment's depth should be written.
    pub write: bool,
}

impl DepthTest {
    pub const LESS_WRITE: Self = DepthTest {
        compare: CompareOp::Less,
        write: true,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct StencilTests {
    pub front: StencilTest,
    pub back: StencilTest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct StencilTest {
    /// Comparison operation between value stored in stencil buffer and refence
    /// value.
    pub compare: CompareOp,

    /// Selects the bits of the unsigned integer stencil values participating
    /// in the stencil test.
    pub compare_mask: State<u32>,

    /// Selects the bits of the unsigned integer stencil values updated by the
    /// stencil test in the stencil buffer.
    pub write_mask: State<u32>,

    /// Reference value for comparison and operations.
    pub reference: State<u32>,

    /// Action performed on samples that fail the stencil test.
    pub fail: StencilOp,

    /// Action performed on samples that pass both the depth and stencil tests.
    pub pass: StencilOp,

    /// Action performed on samples that pass the stencil test and fail the
    /// depth test.
    pub depth_fail: StencilOp,
}

/// Defines what operation should be peformed on value in stencil buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum StencilOp {
    /// Keep the current value.
    Keep,

    /// Write 0.
    Zero,

    /// Replace value with reference value.
    Replace,

    /// Increment value and clamp it to maximum value representable in stencil
    /// buffer format.
    IncrementAndClamp,

    /// Decrement value and clamp to 0.
    DecrementAndClamp,

    /// Invert all bits.
    Invert,

    /// Increment value and wrap to 0 if maximum value representable in stencil
    /// buffer format would be exeeded.
    IncrementAndWrap,

    /// Decrement value and wraps to maximum value representable in stencil
    /// buffer format if value would go below 0.
    DecrementAndWrap,
}

/// Defines how color stored in attachment should be blended with color output
/// of fragment shader.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum ColorBlend {
    /// Values should be treated as unsigned integers and logic operation
    /// perforned. Color format must support logic operations.
    Logic {
        /// Logical operations to be applied.
        op: LogicOp,
    },

    /// Color and alpha of all attachments should be blended in the same way.
    Blending {
        /// Blending state.
        /// If `None` then fragment's output color is written unmodified.
        blending: Option<Blending>,

        /// Bitmask that specifies components that will be written to the
        /// attachment.
        write_mask: ComponentMask,

        /// Constants for certain blending factors.
        constants: State<[f32; 4]>,
    },

    /// Color and alpha of all attachments should be blended in specified way.
    IndependentBlending {
        /// A tuple of two states:
        /// 1. Blending state for each attachment.
        /// If `None` then fragment's output color is written unmodified.
        ///
        /// 2. Bitmask that specifies components that will be written to the
        /// attachment.
        blending: Vec<(Option<Blending>, ComponentMask)>,

        /// Constants for certain blending factors.
        constants: State<[f32; 4]>,
    },
}

impl PartialEq for ColorBlend {
    fn eq(&self, other: &Self) -> bool {
        use ColorBlend::*;

        match (self, other) {
            (Logic { op: l_op }, Logic { op: r_op }) => *l_op == *r_op,
            (
                Blending {
                    blending: l_blending,
                    write_mask: l_write_mask,
                    constants: l_constants,
                },
                Blending {
                    blending: r_blending,
                    write_mask: r_write_mask,
                    constants: r_constants,
                },
            ) => {
                *l_blending == *r_blending
                    && *l_write_mask == *r_write_mask
                    && match (l_constants, r_constants) {
                        (State::Dynamic, State::Dynamic) => true,
                        (State::Static { value: l_value }, State::Static { value: r_value }) => {
                            (*l_value).map(OrderedFloat) == (*r_value).map(OrderedFloat)
                        }
                        _ => false,
                    }
            }
            (
                IndependentBlending {
                    blending: l_blending,
                    constants: l_constants,
                },
                IndependentBlending {
                    blending: r_blending,
                    constants: r_constants,
                },
            ) => {
                *l_blending == *r_blending
                    && match (l_constants, r_constants) {
                        (State::Dynamic, State::Dynamic) => true,
                        (State::Static { value: l_value }, State::Static { value: r_value }) => {
                            (*l_value).map(OrderedFloat) == (*r_value).map(OrderedFloat)
                        }
                        _ => false,
                    }
            }
            _ => false,
        }
    }
}

impl Eq for ColorBlend {}

impl Hash for ColorBlend {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        match self {
            ColorBlend::Logic { op } => {
                Hash::hash(op, hasher);
            }
            ColorBlend::Blending {
                blending,
                write_mask,
                constants,
            } => {
                hasher.write_u8(1);
                if let Some(blending) = blending {
                    Hash::hash(blending, hasher);
                }
                Hash::hash(write_mask, hasher);
                if let State::Static { value } = constants {
                    Hash::hash(&(*value).map(OrderedFloat), hasher);
                }
            }
            ColorBlend::IndependentBlending {
                blending,
                constants,
            } => {
                hasher.write_u8(2);
                for (blending, mask) in blending {
                    if let Some(blending) = blending {
                        Hash::hash(blending, hasher);
                    }
                    Hash::hash(mask, hasher);
                }
                if let State::Static { value } = constants {
                    Hash::hash(&(*value).map(OrderedFloat), hasher);
                }
            }
        }
    }
}

impl Default for ColorBlend {
    fn default() -> Self {
        ColorBlend::Blending {
            blending: Some(Blending {
                color_src_factor: BlendFactor::SrcAlpha,
                color_dst_factor: BlendFactor::OneMinusSrcAlpha,
                color_op: BlendOp::Add,
                alpha_src_factor: BlendFactor::One,
                alpha_dst_factor: BlendFactor::OneMinusSrcAlpha,
                alpha_op: BlendOp::Add,
            }),
            write_mask: ComponentMask::RGBA,
            constants: Static { value: [0.0; 4] },
        }
    }
}

/// Defines how color value from fragment shader's color output should be
/// blended with value stored in attachment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct Blending {
    /// Blend factory to apply to color component values from fragment shader's
    /// color output.
    pub color_src_factor: BlendFactor,

    /// Blend factory to apply to color component values stored in attachment.
    pub color_dst_factor: BlendFactor,

    /// Operation to be performed over color component values.
    pub color_op: BlendOp,

    /// Blend factory to apply to alpha component value from fragment shader's
    /// color output.
    pub alpha_src_factor: BlendFactor,

    /// Blend factory to apply to alpha component value stored in attachment.
    pub alpha_dst_factor: BlendFactor,

    /// Operation to be performed over alpha component values.
    pub alpha_op: BlendOp,
}

/// Logical operation to be applied between color value from fragment shader's
/// color output and value stored in attachment.
///
/// For each operation comment contains an equivalent Rust expression
/// where `s` is value from fragment shader's color output
/// and `d` is value stored in attachment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum LogicOp {
    /// `0`.
    Clear,

    /// `s & d`
    And,

    /// `s & !d`
    AndReverse,

    /// `s`
    Copy,

    /// `!s & d`
    AndInverted,

    /// `d`
    NoOp,

    /// `s ^ d`
    Xor,

    /// `s | d`
    Or,

    /// `!(s | d)`
    Nor,

    /// `!(s ^ d)`
    Equivalent,

    /// `!d`
    Invert,

    /// `s | !d`
    OrReverse,

    /// `!s`
    CopyInverted,

    /// `!s | d`
    OrInverted,

    /// `!(s & d)`
    Nand,

    /// `!0`
    Set,
}

/// Defines how blend factor is calculated.
///
/// For each variant comment contains an equivalent Rust expression
/// where `Rs`, `Gs`, `Bs` and `As` are value components from fragment shader's
/// color output, `Rd`, `Gd`, `Bd` and `Ad` are value components stored in
/// attachment, and `Rc`, `Gc`, `Bc` and `Ac` are value components defined in
/// `constants`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum BlendFactor {
    /// Color: `(0.0, 0.0, 0.0)`
    /// Alpha: `0.0`
    Zero,

    /// Color: `(1.0, 1.0, 1.0)`
    /// Alpha: `1.0`
    One,

    /// Color: `(Rs, Gs, Bs)`
    /// Alpha: `As`
    SrcColor,

    /// Color: `(1.0 - Rs, 1.0 - Gs, 1.0 - Bs)`
    /// Alpha: `1.0 - As`
    OneMinusSrcColor,

    /// Color: `(Rd, Gd, Bd)`
    /// Alpha: `Ad`
    DstColor,

    /// Color: `(1.0 - Rd, 1.0 - Gd, 1.0 - Bd)`
    /// Alpha: `1.0 - Ad`
    OneMinusDstColor,

    /// Color: `(As, As, As)`
    /// Alpha: `As`
    SrcAlpha,

    /// Color: `(1.0 - As, 1.0 - As, 1.0 - As)`
    /// Alpha: `1.0 - As`
    OneMinusSrcAlpha,

    /// Color: `(Ad, Ad, Ad)`
    /// Alpha: `Ad`
    DstAlpha,

    /// Color: `(1.0 - Ad, 1.0 - Ad, 1.0 - Ad)`
    /// Alpha: `1.0 - Ad`
    OneMinusDstAlpha,

    /// Color: `(Rc, Gc, Bc)`
    /// Alpha: `Ac`
    ConstantColor,

    /// Color: `(1.0 - Rc, 1.0 - Gc, 1.0 - Bc)`
    /// Alpha: `1.0 - Ac`
    OneMinusConstantColor,

    /// Color: `(Ac, Ac, Ac)`
    /// Alpha: `Ac`
    ConstantAlpha,

    /// Color: `(1.0 - Ac, 1.0 - Ac, 1.0 - Ac)`
    /// Alpha: `1.0 - Ac`
    OneMinusConstantAlpha,

    /// Color: `{let f = min(As, 1.0 - Ad); (f,f,f)}`
    /// Alpha: `1.0`
    SrcAlphaSaturate,
}

/// Blending operation to be applied between color value from fragment shader's
/// color output and value stored in attachment.
///
/// For each operation comment contains an equivalent Rust expression
/// where `S` is value from fragment shader's color output, `Sf` is factor
/// calculated for fragment shader's color output, D` is value stored in
/// attachment and `Df` is factor calculated for value stored in attachment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum BlendOp {
    /// `S * Sf + D * Df`.
    Add,

    /// `S * Sf - D * Df`
    Subtract,

    /// `D * Df - S * Sf`
    ReverseSubtract,

    /// `min(S, D)`
    Min,

    /// `max(S, D)`
    Max,
}

bitflags::bitflags! {
    /// Flags for each of color components.
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct ComponentMask: u8 {
        const R = 0b0001;
        const G = 0b0010;
        const B = 0b0100;
        const A = 0b1000;
        const RGB = 0b0111;
        const RGBA = 0b1111;
    }
}

#[derive(Debug)]
pub struct GraphicsPipelineDesc {
    pub vertex_shader: VertexShader,
    pub layout: PipelineLayout,
    pub vertex_bindings: Vec<VertexInputBinding>,
    pub vertex_attributes: Vec<VertexInputAttribute>,
    pub primitive_topology: PrimitiveTopology,
    pub primitive_restart_enable: bool,
    pub rasterizer: Option<Rasterizer>,
}

#[derive(Debug)]
pub struct DynamicGraphicsPipeline {
    graphics_pipeline: Option<GraphicsPipeline>,
    pub desc: GraphicsPipelineDesc,
}

impl DynamicGraphicsPipeline {
    pub fn new(desc: GraphicsPipelineDesc) -> Self {
        DynamicGraphicsPipeline {
            desc,
            graphics_pipeline: None,
        }
    }

    pub fn get(
        &mut self,
        render_pass: &RenderPass,
        subpass: u32,
        device: &Device,
    ) -> Result<&GraphicsPipeline, OutOfMemory> {
        match &mut self.graphics_pipeline {
            Some(graphics_pipeline) => {
                let mut compatible = true;
                let info = graphics_pipeline.info();

                if info.render_pass != *render_pass {
                    compatible = false;
                }
                if info.subpass != subpass {
                    compatible = false;
                }
                if info.vertex_shader != self.desc.vertex_shader {
                    compatible = false;
                }
                if info.primitive_topology != self.desc.primitive_topology {
                    compatible = false;
                }
                if info.primitive_restart_enable != self.desc.primitive_restart_enable {
                    compatible = false;
                }
                if info.layout != self.desc.layout {
                    compatible = false;
                }
                if info.rasterizer != self.desc.rasterizer {
                    compatible = false;
                }
                if info.vertex_bindings != self.desc.vertex_bindings {
                    compatible = false;
                }
                if info.vertex_attributes != self.desc.vertex_attributes {
                    compatible = false;
                }

                if !compatible {
                    *graphics_pipeline = device.create_graphics_pipeline(desc_to_info(
                        &self.desc,
                        render_pass,
                        subpass,
                    ))?;
                }

                Ok(graphics_pipeline)
            }
            graphics_pipeline => Ok(graphics_pipeline.get_or_insert(
                device.create_graphics_pipeline(desc_to_info(&self.desc, render_pass, subpass))?,
            )),
        }
    }
}

fn desc_to_info(
    desc: &GraphicsPipelineDesc,
    render_pass: &RenderPass,
    subpass: u32,
) -> GraphicsPipelineInfo {
    GraphicsPipelineInfo {
        vertex_bindings: desc.vertex_bindings.clone(),
        vertex_attributes: desc.vertex_attributes.clone(),
        primitive_topology: desc.primitive_topology,
        primitive_restart_enable: desc.primitive_restart_enable,
        vertex_shader: desc.vertex_shader.clone(),
        rasterizer: desc.rasterizer.clone(),
        layout: desc.layout.clone(),
        render_pass: render_pass.clone(),
        subpass,
    }
}
