//! WG-6: guest-drawn canvas present.
//! `get-canvas-context` + configure + shader + VERTEX buffer + render pipeline
//! + `get-current-texture` + create-view + draw(3) + `queue.submit`.
//! Not host-clear present. Not `create-texture` 1×1 cite. Not `@builtin(vertex_index)`.
//! Guest drops owns; `run` returns harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{
    flags, Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuDevice {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuShaderModule {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuPipelineLayout {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuRenderPipeline {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuBuffer {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuTexture {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuTextureView {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuCommandEncoder {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuRenderPassEncoder {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuQueue {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuCommandBuffer {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuQuerySet;

#[derive(Debug)]
struct GpuCanvasContext;

#[derive(Debug)]
struct Gpu;

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum PredefinedColorSpace {
    #[component(name = "srgb")]
    Srgb,
    #[component(name = "display-p3")]
    DisplayP3,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuCanvasAlphaMode {
    #[component(name = "opaque")]
    Opaque,
    #[component(name = "premultiplied")]
    Premultiplied,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuCanvasToneMappingMode {
    #[component(name = "standard")]
    Standard,
    #[component(name = "extended")]
    Extended,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuCanvasToneMapping {
    mode: Option<GpuCanvasToneMappingMode>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuCompareFunction {
    #[component(name = "never")]
    Never,
    #[component(name = "less")]
    Less,
    #[component(name = "equal")]
    Equal,
    #[component(name = "less-equal")]
    LessEqual,
    #[component(name = "greater")]
    Greater,
    #[component(name = "not-equal")]
    NotEqual,
    #[component(name = "greater-equal")]
    GreaterEqual,
    #[component(name = "always")]
    Always,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuTextureFormat {
    #[component(name = "r8unorm")]
    R8unorm,
    #[component(name = "r8snorm")]
    R8snorm,
    #[component(name = "r8uint")]
    R8uint,
    #[component(name = "r8sint")]
    R8sint,
    #[component(name = "r16unorm")]
    R16unorm,
    #[component(name = "r16snorm")]
    R16snorm,
    #[component(name = "r16uint")]
    R16uint,
    #[component(name = "r16sint")]
    R16sint,
    #[component(name = "r16float")]
    R16float,
    #[component(name = "rg8unorm")]
    Rg8unorm,
    #[component(name = "rg8snorm")]
    Rg8snorm,
    #[component(name = "rg8uint")]
    Rg8uint,
    #[component(name = "rg8sint")]
    Rg8sint,
    #[component(name = "r32uint")]
    R32uint,
    #[component(name = "r32sint")]
    R32sint,
    #[component(name = "r32float")]
    R32float,
    #[component(name = "rg16unorm")]
    Rg16unorm,
    #[component(name = "rg16snorm")]
    Rg16snorm,
    #[component(name = "rg16uint")]
    Rg16uint,
    #[component(name = "rg16sint")]
    Rg16sint,
    #[component(name = "rg16float")]
    Rg16float,
    #[component(name = "rgba8unorm")]
    Rgba8unorm,
    #[component(name = "rgba8unorm-srgb")]
    Rgba8unormSrgb,
    #[component(name = "rgba8snorm")]
    Rgba8snorm,
    #[component(name = "rgba8uint")]
    Rgba8uint,
    #[component(name = "rgba8sint")]
    Rgba8sint,
    #[component(name = "bgra8unorm")]
    Bgra8unorm,
    #[component(name = "bgra8unorm-srgb")]
    Bgra8unormSrgb,
    #[component(name = "rgb9e5ufloat")]
    Rgb9e5ufloat,
    #[component(name = "rgb10a2uint")]
    Rgb10a2uint,
    #[component(name = "rgb10a2unorm")]
    Rgb10a2unorm,
    #[component(name = "rg11b10ufloat")]
    Rg11b10ufloat,
    #[component(name = "rg32uint")]
    Rg32uint,
    #[component(name = "rg32sint")]
    Rg32sint,
    #[component(name = "rg32float")]
    Rg32float,
    #[component(name = "rgba16unorm")]
    Rgba16unorm,
    #[component(name = "rgba16snorm")]
    Rgba16snorm,
    #[component(name = "rgba16uint")]
    Rgba16uint,
    #[component(name = "rgba16sint")]
    Rgba16sint,
    #[component(name = "rgba16float")]
    Rgba16float,
    #[component(name = "rgba32uint")]
    Rgba32uint,
    #[component(name = "rgba32sint")]
    Rgba32sint,
    #[component(name = "rgba32float")]
    Rgba32float,
    #[component(name = "stencil8")]
    Stencil8,
    #[component(name = "depth16unorm")]
    Depth16unorm,
    #[component(name = "depth24plus")]
    Depth24plus,
    #[component(name = "depth24plus-stencil8")]
    Depth24plusStencil8,
    #[component(name = "depth32float")]
    Depth32float,
    #[component(name = "depth32float-stencil8")]
    Depth32floatStencil8,
    #[component(name = "bc1-rgba-unorm")]
    Bc1RgbaUnorm,
    #[component(name = "bc1-rgba-unorm-srgb")]
    Bc1RgbaUnormSrgb,
    #[component(name = "bc2-rgba-unorm")]
    Bc2RgbaUnorm,
    #[component(name = "bc2-rgba-unorm-srgb")]
    Bc2RgbaUnormSrgb,
    #[component(name = "bc3-rgba-unorm")]
    Bc3RgbaUnorm,
    #[component(name = "bc3-rgba-unorm-srgb")]
    Bc3RgbaUnormSrgb,
    #[component(name = "bc4-r-unorm")]
    Bc4RUnorm,
    #[component(name = "bc4-r-snorm")]
    Bc4RSnorm,
    #[component(name = "bc5-rg-unorm")]
    Bc5RgUnorm,
    #[component(name = "bc5-rg-snorm")]
    Bc5RgSnorm,
    #[component(name = "bc6h-rgb-ufloat")]
    Bc6hRgbUfloat,
    #[component(name = "bc6h-rgb-float")]
    Bc6hRgbFloat,
    #[component(name = "bc7-rgba-unorm")]
    Bc7RgbaUnorm,
    #[component(name = "bc7-rgba-unorm-srgb")]
    Bc7RgbaUnormSrgb,
    #[component(name = "etc2-rgb8unorm")]
    Etc2Rgb8unorm,
    #[component(name = "etc2-rgb8unorm-srgb")]
    Etc2Rgb8unormSrgb,
    #[component(name = "etc2-rgb8a1unorm")]
    Etc2Rgb8a1unorm,
    #[component(name = "etc2-rgb8a1unorm-srgb")]
    Etc2Rgb8a1unormSrgb,
    #[component(name = "etc2-rgba8unorm")]
    Etc2Rgba8unorm,
    #[component(name = "etc2-rgba8unorm-srgb")]
    Etc2Rgba8unormSrgb,
    #[component(name = "eac-r11unorm")]
    EacR11unorm,
    #[component(name = "eac-r11snorm")]
    EacR11snorm,
    #[component(name = "eac-rg11unorm")]
    EacRg11unorm,
    #[component(name = "eac-rg11snorm")]
    EacRg11snorm,
    #[component(name = "astc4x4-unorm")]
    Astc4x4Unorm,
    #[component(name = "astc4x4-unorm-srgb")]
    Astc4x4UnormSrgb,
    #[component(name = "astc5x4-unorm")]
    Astc5x4Unorm,
    #[component(name = "astc5x4-unorm-srgb")]
    Astc5x4UnormSrgb,
    #[component(name = "astc5x5-unorm")]
    Astc5x5Unorm,
    #[component(name = "astc5x5-unorm-srgb")]
    Astc5x5UnormSrgb,
    #[component(name = "astc6x5-unorm")]
    Astc6x5Unorm,
    #[component(name = "astc6x5-unorm-srgb")]
    Astc6x5UnormSrgb,
    #[component(name = "astc6x6-unorm")]
    Astc6x6Unorm,
    #[component(name = "astc6x6-unorm-srgb")]
    Astc6x6UnormSrgb,
    #[component(name = "astc8x5-unorm")]
    Astc8x5Unorm,
    #[component(name = "astc8x5-unorm-srgb")]
    Astc8x5UnormSrgb,
    #[component(name = "astc8x6-unorm")]
    Astc8x6Unorm,
    #[component(name = "astc8x6-unorm-srgb")]
    Astc8x6UnormSrgb,
    #[component(name = "astc8x8-unorm")]
    Astc8x8Unorm,
    #[component(name = "astc8x8-unorm-srgb")]
    Astc8x8UnormSrgb,
    #[component(name = "astc10x5-unorm")]
    Astc10x5Unorm,
    #[component(name = "astc10x5-unorm-srgb")]
    Astc10x5UnormSrgb,
    #[component(name = "astc10x6-unorm")]
    Astc10x6Unorm,
    #[component(name = "astc10x6-unorm-srgb")]
    Astc10x6UnormSrgb,
    #[component(name = "astc10x8-unorm")]
    Astc10x8Unorm,
    #[component(name = "astc10x8-unorm-srgb")]
    Astc10x8UnormSrgb,
    #[component(name = "astc10x10-unorm")]
    Astc10x10Unorm,
    #[component(name = "astc10x10-unorm-srgb")]
    Astc10x10UnormSrgb,
    #[component(name = "astc12x10-unorm")]
    Astc12x10Unorm,
    #[component(name = "astc12x10-unorm-srgb")]
    Astc12x10UnormSrgb,
    #[component(name = "astc12x12-unorm")]
    Astc12x12Unorm,
    #[component(name = "astc12x12-unorm-srgb")]
    Astc12x12UnormSrgb,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum GpuLayoutMode {
    #[component(name = "specific")]
    Specific(Resource<GpuPipelineLayout>),
    #[component(name = "auto")]
    Auto,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuShaderModuleCompilationHint {
    #[component(name = "entry-point")]
    entry_point: String,
    layout: Option<GpuLayoutMode>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuShaderModuleDescriptor {
    code: String,
    #[component(name = "compilation-hints")]
    compilation_hints: Option<Vec<GpuShaderModuleCompilationHint>>,
    label: Option<String>,
}

/// WIT `resource record-gpu-pipeline-constant-value` (pipeline constant map).
#[derive(Debug)]
struct RecordGpuPipelineConstantValue;

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuBlendFactor {
    #[component(name = "zero")]
    Zero,
    #[component(name = "one")]
    One,
    #[component(name = "src")]
    Src,
    #[component(name = "one-minus-src")]
    OneMinusSrc,
    #[component(name = "src-alpha")]
    SrcAlpha,
    #[component(name = "one-minus-src-alpha")]
    OneMinusSrcAlpha,
    #[component(name = "dst")]
    Dst,
    #[component(name = "one-minus-dst")]
    OneMinusDst,
    #[component(name = "dst-alpha")]
    DstAlpha,
    #[component(name = "one-minus-dst-alpha")]
    OneMinusDstAlpha,
    #[component(name = "src-alpha-saturated")]
    SrcAlphaSaturated,
    #[component(name = "constant")]
    Constant,
    #[component(name = "one-minus-constant")]
    OneMinusConstant,
    #[component(name = "src1")]
    Src1,
    #[component(name = "one-minus-src1")]
    OneMinusSrc1,
    #[component(name = "src1-alpha")]
    Src1Alpha,
    #[component(name = "one-minus-src1-alpha")]
    OneMinusSrc1Alpha,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuBlendOperation {
    #[component(name = "add")]
    Add,
    #[component(name = "subtract")]
    Subtract,
    #[component(name = "reverse-subtract")]
    ReverseSubtract,
    #[component(name = "min")]
    Min,
    #[component(name = "max")]
    Max,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuBlendComponent {
    operation: Option<GpuBlendOperation>,
    #[component(name = "src-factor")]
    src_factor: Option<GpuBlendFactor>,
    #[component(name = "dst-factor")]
    dst_factor: Option<GpuBlendFactor>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuBlendState {
    color: GpuBlendComponent,
    alpha: GpuBlendComponent,
}

flags! {
    GpuColorWrite {
        #[component(name = "red")]
        const RED;
        #[component(name = "green")]
        const GREEN;
        #[component(name = "blue")]
        const BLUE;
        #[component(name = "alpha")]
        const ALPHA;
        #[component(name = "all")]
        const ALL;
    }
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuColorTargetState {
    format: GpuTextureFormat,
    blend: Option<GpuBlendState>,
    #[component(name = "write-mask")]
    write_mask: Option<GpuColorWrite>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuPrimitiveTopology {
    #[component(name = "point-list")]
    PointList,
    #[component(name = "line-list")]
    LineList,
    #[component(name = "line-strip")]
    LineStrip,
    #[component(name = "triangle-list")]
    TriangleList,
    #[component(name = "triangle-strip")]
    TriangleStrip,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuIndexFormat {
    #[component(name = "uint16")]
    Uint16,
    #[component(name = "uint32")]
    Uint32,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuFrontFace {
    #[component(name = "ccw")]
    Ccw,
    #[component(name = "cw")]
    Cw,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuCullMode {
    #[component(name = "none")]
    None,
    #[component(name = "front")]
    Front,
    #[component(name = "back")]
    Back,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuPrimitiveState {
    topology: Option<GpuPrimitiveTopology>,
    #[component(name = "strip-index-format")]
    strip_index_format: Option<GpuIndexFormat>,
    #[component(name = "front-face")]
    front_face: Option<GpuFrontFace>,
    #[component(name = "cull-mode")]
    cull_mode: Option<GpuCullMode>,
    #[component(name = "unclipped-depth")]
    unclipped_depth: Option<bool>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuStencilOperation {
    #[component(name = "keep")]
    Keep,
    #[component(name = "zero")]
    Zero,
    #[component(name = "replace")]
    Replace,
    #[component(name = "invert")]
    Invert,
    #[component(name = "increment-clamp")]
    IncrementClamp,
    #[component(name = "decrement-clamp")]
    DecrementClamp,
    #[component(name = "increment-wrap")]
    IncrementWrap,
    #[component(name = "decrement-wrap")]
    DecrementWrap,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuStencilFaceState {
    compare: Option<GpuCompareFunction>,
    #[component(name = "fail-op")]
    fail_op: Option<GpuStencilOperation>,
    #[component(name = "depth-fail-op")]
    depth_fail_op: Option<GpuStencilOperation>,
    #[component(name = "pass-op")]
    pass_op: Option<GpuStencilOperation>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuDepthStencilState {
    format: GpuTextureFormat,
    #[component(name = "depth-write-enabled")]
    depth_write_enabled: Option<bool>,
    #[component(name = "depth-compare")]
    depth_compare: Option<GpuCompareFunction>,
    #[component(name = "stencil-front")]
    stencil_front: Option<GpuStencilFaceState>,
    #[component(name = "stencil-back")]
    stencil_back: Option<GpuStencilFaceState>,
    #[component(name = "stencil-read-mask")]
    stencil_read_mask: Option<u32>,
    #[component(name = "stencil-write-mask")]
    stencil_write_mask: Option<u32>,
    #[component(name = "depth-bias")]
    depth_bias: Option<i32>,
    #[component(name = "depth-bias-slope-scale")]
    depth_bias_slope_scale: Option<f32>,
    #[component(name = "depth-bias-clamp")]
    depth_bias_clamp: Option<f32>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuMultisampleState {
    count: Option<u32>,
    mask: Option<u32>,
    #[component(name = "alpha-to-coverage-enabled")]
    alpha_to_coverage_enabled: Option<bool>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuVertexFormat {
    #[component(name = "uint8")]
    Uint8,
    #[component(name = "uint8x2")]
    Uint8x2,
    #[component(name = "uint8x4")]
    Uint8x4,
    #[component(name = "sint8")]
    Sint8,
    #[component(name = "sint8x2")]
    Sint8x2,
    #[component(name = "sint8x4")]
    Sint8x4,
    #[component(name = "unorm8")]
    Unorm8,
    #[component(name = "unorm8x2")]
    Unorm8x2,
    #[component(name = "unorm8x4")]
    Unorm8x4,
    #[component(name = "snorm8")]
    Snorm8,
    #[component(name = "snorm8x2")]
    Snorm8x2,
    #[component(name = "snorm8x4")]
    Snorm8x4,
    #[component(name = "uint16")]
    Uint16,
    #[component(name = "uint16x2")]
    Uint16x2,
    #[component(name = "uint16x4")]
    Uint16x4,
    #[component(name = "sint16")]
    Sint16,
    #[component(name = "sint16x2")]
    Sint16x2,
    #[component(name = "sint16x4")]
    Sint16x4,
    #[component(name = "unorm16")]
    Unorm16,
    #[component(name = "unorm16x2")]
    Unorm16x2,
    #[component(name = "unorm16x4")]
    Unorm16x4,
    #[component(name = "snorm16")]
    Snorm16,
    #[component(name = "snorm16x2")]
    Snorm16x2,
    #[component(name = "snorm16x4")]
    Snorm16x4,
    #[component(name = "float16")]
    Float16,
    #[component(name = "float16x2")]
    Float16x2,
    #[component(name = "float16x4")]
    Float16x4,
    #[component(name = "float32")]
    Float32,
    #[component(name = "float32x2")]
    Float32x2,
    #[component(name = "float32x3")]
    Float32x3,
    #[component(name = "float32x4")]
    Float32x4,
    #[component(name = "uint32")]
    Uint32,
    #[component(name = "uint32x2")]
    Uint32x2,
    #[component(name = "uint32x3")]
    Uint32x3,
    #[component(name = "uint32x4")]
    Uint32x4,
    #[component(name = "sint32")]
    Sint32,
    #[component(name = "sint32x2")]
    Sint32x2,
    #[component(name = "sint32x3")]
    Sint32x3,
    #[component(name = "sint32x4")]
    Sint32x4,
    #[component(name = "unorm1010102")]
    Unorm1010102,
    #[component(name = "unorm8x4-bgra")]
    Unorm8x4Bgra,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuVertexAttribute {
    format: GpuVertexFormat,
    offset: u64,
    #[component(name = "shader-location")]
    shader_location: u32,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuVertexStepMode {
    #[component(name = "vertex")]
    Vertex,
    #[component(name = "instance")]
    Instance,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuVertexBufferLayout {
    #[component(name = "array-stride")]
    array_stride: u64,
    #[component(name = "step-mode")]
    step_mode: Option<GpuVertexStepMode>,
    attributes: Vec<GpuVertexAttribute>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuProgrammableStage {
    module: Resource<GpuShaderModule>,
    #[component(name = "entry-point")]
    entry_point: Option<String>,
    constants: Option<Resource<RecordGpuPipelineConstantValue>>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuVertexState {
    buffers: Option<Vec<Option<GpuVertexBufferLayout>>>,
    module: Resource<GpuShaderModule>,
    #[component(name = "entry-point")]
    entry_point: Option<String>,
    constants: Option<Resource<RecordGpuPipelineConstantValue>>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuFragmentState {
    targets: Vec<Option<GpuColorTargetState>>,
    module: Resource<GpuShaderModule>,
    #[component(name = "entry-point")]
    entry_point: Option<String>,
    constants: Option<Resource<RecordGpuPipelineConstantValue>>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuComputePipelineDescriptor {
    compute: GpuProgrammableStage,
    layout: GpuLayoutMode,
    label: Option<String>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuRenderPipelineDescriptor {
    vertex: GpuVertexState,
    primitive: Option<GpuPrimitiveState>,
    #[component(name = "depth-stencil")]
    depth_stencil: Option<GpuDepthStencilState>,
    multisample: Option<GpuMultisampleState>,
    fragment: Option<GpuFragmentState>,
    layout: GpuLayoutMode,
    label: Option<String>,
}

flags! {
    GpuBufferUsage {
        #[component(name = "map-read")]
        const MAP_READ;
        #[component(name = "map-write")]
        const MAP_WRITE;
        #[component(name = "copy-src")]
        const COPY_SRC;
        #[component(name = "copy-dst")]
        const COPY_DST;
        #[component(name = "index")]
        const INDEX;
        #[component(name = "vertex")]
        const VERTEX;
        #[component(name = "uniform")]
        const UNIFORM;
        #[component(name = "storage")]
        const STORAGE;
        #[component(name = "indirect")]
        const INDIRECT;
        #[component(name = "query-resolve")]
        const QUERY_RESOLVE;
    }
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuBufferDescriptor {
    size: u64,
    usage: GpuBufferUsage,
    #[component(name = "mapped-at-creation")]
    mapped_at_creation: Option<bool>,
    label: Option<String>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuTextureViewDimension {
    #[component(name = "d1")]
    D1,
    #[component(name = "d2")]
    D2,
    #[component(name = "d2-array")]
    D2Array,
    #[component(name = "cube")]
    Cube,
    #[component(name = "cube-array")]
    CubeArray,
    #[component(name = "d3")]
    D3,
}

flags! {
    GpuTextureUsage {
        #[component(name = "copy-src")]
        const COPY_SRC;
        #[component(name = "copy-dst")]
        const COPY_DST;
        #[component(name = "texture-binding")]
        const TEXTURE_BINDING;
        #[component(name = "storage-binding")]
        const STORAGE_BINDING;
        #[component(name = "render-attachment")]
        const RENDER_ATTACHMENT;
        #[component(name = "transient-attachment")]
        const TRANSIENT_ATTACHMENT;
    }
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuTextureDimension {
    #[component(name = "d1")]
    D1,
    #[component(name = "d2")]
    D2,
    #[component(name = "d3")]
    D3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuTextureAspect {
    #[component(name = "all")]
    All,
    #[component(name = "stencil-only")]
    StencilOnly,
    #[component(name = "depth-only")]
    DepthOnly,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuExtent3D {
    width: u32,
    height: Option<u32>,
    #[component(name = "depth-or-array-layers")]
    depth_or_array_layers: Option<u32>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuTextureDescriptor {
    size: GpuExtent3D,
    #[component(name = "mip-level-count")]
    mip_level_count: Option<u32>,
    #[component(name = "sample-count")]
    sample_count: Option<u32>,
    dimension: Option<GpuTextureDimension>,
    format: GpuTextureFormat,
    usage: GpuTextureUsage,
    #[component(name = "view-formats")]
    view_formats: Option<Vec<GpuTextureFormat>>,
    #[component(name = "texture-binding-view-dimension")]
    texture_binding_view_dimension: Option<GpuTextureViewDimension>,
    label: Option<String>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuTextureViewDescriptor {
    format: Option<GpuTextureFormat>,
    dimension: Option<GpuTextureViewDimension>,
    usage: Option<GpuTextureUsage>,
    aspect: Option<GpuTextureAspect>,
    #[component(name = "base-mip-level")]
    base_mip_level: Option<u32>,
    #[component(name = "mip-level-count")]
    mip_level_count: Option<u32>,
    #[component(name = "base-array-layer")]
    base_array_layer: Option<u32>,
    #[component(name = "array-layer-count")]
    array_layer_count: Option<u32>,
    swizzle: Option<String>,
    label: Option<String>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuCommandEncoderDescriptor {
    label: Option<String>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuLoadOp {
    #[component(name = "load")]
    Load,
    #[component(name = "clear")]
    Clear,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuStoreOp {
    #[component(name = "store")]
    Store,
    #[component(name = "discard")]
    Discard,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuColor {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuRenderPassColorAttachment {
    view: Resource<GpuTextureView>,
    #[component(name = "depth-slice")]
    depth_slice: Option<u32>,
    #[component(name = "resolve-target")]
    resolve_target: Option<Resource<GpuTextureView>>,
    #[component(name = "clear-value")]
    clear_value: Option<GpuColor>,
    #[component(name = "load-op")]
    load_op: GpuLoadOp,
    #[component(name = "store-op")]
    store_op: GpuStoreOp,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuRenderPassDepthStencilAttachment {
    view: Resource<GpuTextureView>,
    #[component(name = "depth-clear-value")]
    depth_clear_value: Option<f32>,
    #[component(name = "depth-load-op")]
    depth_load_op: Option<GpuLoadOp>,
    #[component(name = "depth-store-op")]
    depth_store_op: Option<GpuStoreOp>,
    #[component(name = "depth-read-only")]
    depth_read_only: Option<bool>,
    #[component(name = "stencil-clear-value")]
    stencil_clear_value: Option<u32>,
    #[component(name = "stencil-load-op")]
    stencil_load_op: Option<GpuLoadOp>,
    #[component(name = "stencil-store-op")]
    stencil_store_op: Option<GpuStoreOp>,
    #[component(name = "stencil-read-only")]
    stencil_read_only: Option<bool>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuRenderPassTimestampWrites {
    #[component(name = "query-set")]
    query_set: Resource<GpuQuerySet>,
    #[component(name = "beginning-of-pass-write-index")]
    beginning_of_pass_write_index: Option<u32>,
    #[component(name = "end-of-pass-write-index")]
    end_of_pass_write_index: Option<u32>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuRenderPassDescriptor {
    #[component(name = "color-attachments")]
    color_attachments: Vec<Option<GpuRenderPassColorAttachment>>,
    #[component(name = "depth-stencil-attachment")]
    depth_stencil_attachment: Option<GpuRenderPassDepthStencilAttachment>,
    #[component(name = "occlusion-query-set")]
    occlusion_query_set: Option<Resource<GpuQuerySet>>,
    #[component(name = "timestamp-writes")]
    timestamp_writes: Option<GpuRenderPassTimestampWrites>,
    #[component(name = "max-draw-count")]
    max_draw_count: Option<u64>,
    label: Option<String>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuCommandBufferDescriptor {
    label: Option<String>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuCanvasConfiguration {
    device: Resource<GpuDevice>,
    format: GpuTextureFormat,
    usage: Option<GpuTextureUsage>,
    #[component(name = "view-formats")]
    view_formats: Option<Vec<GpuTextureFormat>>,
    #[component(name = "color-space")]
    color_space: Option<PredefinedColorSpace>,
    #[component(name = "tone-mapping")]
    tone_mapping: Option<GpuCanvasToneMapping>,
    #[component(name = "alpha-mode")]
    alpha_mode: Option<GpuCanvasAlphaMode>,
}

struct TestHost {
    table: ResourceTable,
}

struct Flags {
    shader: Arc<AtomicBool>,
    buffer: Arc<AtomicBool>,
    pipeline: Arc<AtomicBool>,
    set_vertex_buffer: Arc<AtomicBool>,
    draw: Arc<AtomicBool>,
    submitted: Arc<AtomicBool>,
    current_texture: Arc<AtomicBool>,
    preferred: Arc<AtomicBool>,
}

fn push_res<T: Send + Sync + 'static>(
    webgpu: &mut wasmtime::component::LinkerInstance<'_, TestHost>,
    name: &str,
) -> wasmtime::Result<()> {
    webgpu.resource(name, ResourceType::host::<T>(), |mut store, rep| {
        let resource = Resource::<T>::new_own(rep);
        store.data_mut().table.delete(resource)?;
        Ok(())
    })?;
    Ok(())
}

fn clone_flags(flags: &Flags) -> Flags {
    Flags {
        shader: flags.shader.clone(),
        buffer: flags.buffer.clone(),
        pipeline: flags.pipeline.clone(),
        set_vertex_buffer: flags.set_vertex_buffer.clone(),
        draw: flags.draw.clone(),
        submitted: flags.submitted.clone(),
        current_texture: flags.current_texture.clone(),
        preferred: flags.preferred.clone(),
    }
}

fn register_dawn_guest_canvas_present(
    linker: &mut Linker<TestHost>,
    flags: Flags,
) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    push_res::<GpuDevice>(&mut webgpu, "gpu-device")?;
    push_res::<GpuShaderModule>(&mut webgpu, "gpu-shader-module")?;
    push_res::<GpuPipelineLayout>(&mut webgpu, "gpu-pipeline-layout")?;
    push_res::<GpuRenderPipeline>(&mut webgpu, "gpu-render-pipeline")?;
    push_res::<RecordGpuPipelineConstantValue>(&mut webgpu, "record-gpu-pipeline-constant-value")?;
    push_res::<GpuBuffer>(&mut webgpu, "gpu-buffer")?;
    push_res::<GpuTexture>(&mut webgpu, "gpu-texture")?;
    push_res::<GpuTextureView>(&mut webgpu, "gpu-texture-view")?;
    push_res::<GpuCommandEncoder>(&mut webgpu, "gpu-command-encoder")?;
    push_res::<GpuRenderPassEncoder>(&mut webgpu, "gpu-render-pass-encoder")?;
    push_res::<GpuQueue>(&mut webgpu, "gpu-queue")?;
    push_res::<GpuCommandBuffer>(&mut webgpu, "gpu-command-buffer")?;
    push_res::<GpuQuerySet>(&mut webgpu, "gpu-query-set")?;
    push_res::<GpuCanvasContext>(&mut webgpu, "gpu-canvas-context")?;
    push_res::<Gpu>(&mut webgpu, "gpu")?;
    webgpu.func_wrap("get-device", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuDevice { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-canvas-context", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuCanvasContext)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-gpu", |mut store, ()| {
        let resource = store.data_mut().table.push(Gpu)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("[method]gpu.get-preferred-canvas-format", {
        let preferred = flags.preferred.clone();
        move |mut caller, (gpu,): (Resource<Gpu>,)| {
            caller.data_mut().table.get(&gpu).map(|_| ())?;
            preferred.store(true, Ordering::SeqCst);
            Ok((GpuTextureFormat::Bgra8unorm,))
        }
    })?;
    webgpu.func_wrap(
        "[method]gpu-canvas-context.configure",
        |mut caller, (ctx, config): (Resource<GpuCanvasContext>, GpuCanvasConfiguration)| {
            caller.data_mut().table.get(&ctx).map(|_| ())?;
            caller.data_mut().table.get(&config.device).map(|_| ())?;
            assert!(
                matches!(config.format, GpuTextureFormat::Bgra8unorm),
                "guest must pass format from get-preferred-canvas-format (bgra8unorm)"
            );
            assert!(config.usage.is_none());
            assert!(config.view_formats.is_none());
            assert!(config.color_space.is_none());
            assert!(config.tone_mapping.is_none());
            assert!(config.alpha_mode.is_none());
            Ok(())
        },
    )?;
    webgpu.func_wrap("[method]gpu-canvas-context.get-current-texture", {
        let current_texture = flags.current_texture.clone();
        move |mut caller, (ctx,): (Resource<GpuCanvasContext>,)| {
            caller.data_mut().table.get(&ctx).map(|_| ())?;
            current_texture.store(true, Ordering::SeqCst);
            let resource = caller.data_mut().table.push(GpuTexture { rep: 37 })?;
            Ok((resource,))
        }
    })?;
    webgpu.func_wrap("[method]gpu-device.create-shader-module", {
        let shader = flags.shader.clone();
        move |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuShaderModuleDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert!(
                descriptor.code.contains("@location(0)"),
                "guest must pass a vertex attribute, not vertex_index"
            );
            assert!(
                !descriptor.code.contains("vertex_index"),
                "WG-6 3D must not use @builtin(vertex_index)"
            );
            assert!(descriptor.compilation_hints.is_none());
            assert_eq!(descriptor.label.as_deref(), Some("wg6"));
            shader.store(true, Ordering::SeqCst);
            let resource = caller.data_mut().table.push(GpuShaderModule { rep: 11 })?;
            Ok((resource,))
        }
    })?;
    webgpu.func_wrap("[method]gpu-device.create-buffer", {
        let buffer = flags.buffer.clone();
        move |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuBufferDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert_eq!(descriptor.size, 36, "guest must pass 3 × float32x3");
            assert!(
                descriptor.usage.contains(GpuBufferUsage::VERTEX),
                "guest must pass VERTEX"
            );
            assert!(
                descriptor.usage.contains(GpuBufferUsage::COPY_DST),
                "guest must pass COPY_DST"
            );
            assert!(descriptor.mapped_at_creation.is_none());
            buffer.store(true, Ordering::SeqCst);
            let resource = caller.data_mut().table.push(GpuBuffer { rep: 31 })?;
            Ok((resource,))
        }
    })?;
    webgpu.func_wrap("[method]gpu-device.create-render-pipeline", {
        let pipeline = flags.pipeline.clone();
        move |mut caller,
              (device, descriptor): (Resource<GpuDevice>, GpuRenderPipelineDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            caller
                .data_mut()
                .table
                .get(&descriptor.vertex.module)
                .map(|_| ())?;
            assert!(matches!(descriptor.layout, GpuLayoutMode::Auto));
            let buffers = descriptor
                .vertex
                .buffers
                .as_ref()
                .expect("guest must pass vertex.buffers");
            assert_eq!(buffers.len(), 1, "guest must pass one vertex buffer slot");
            let layout = buffers[0]
                .as_ref()
                .expect("guest must pass a vertex buffer layout");
            assert_eq!(layout.array_stride, 12);
            assert_eq!(layout.attributes.len(), 1);
            assert!(
                matches!(layout.attributes[0].format, GpuVertexFormat::Float32x3),
                "guest must pass float32x3 attribute"
            );
            assert_eq!(layout.attributes[0].offset, 0);
            assert_eq!(layout.attributes[0].shader_location, 0);
            assert_eq!(descriptor.vertex.entry_point.as_deref(), Some("vs_main"));
            assert!(descriptor.vertex.constants.is_none());
            let primitive = descriptor
                .primitive
                .as_ref()
                .expect("guest must pass primitive some");
            assert!(
                matches!(primitive.cull_mode, Some(GpuCullMode::Back)),
                "guest must pass cull-mode=back"
            );
            assert!(
                descriptor.depth_stencil.is_none(),
                "guest must omit depth-stencil (cite pass has no depth)"
            );
            assert!(descriptor.multisample.is_none());
            let fragment = descriptor
                .fragment
                .as_ref()
                .expect("guest must pass fragment");
            assert_eq!(fragment.targets.len(), 1);
            assert!(
                matches!(
                    fragment.targets[0].as_ref().map(|t| t.format),
                    Some(GpuTextureFormat::Bgra8unorm)
                ),
                "guest must pass fragment target format from get-preferred-canvas-format"
            );
            assert_eq!(
                fragment.targets[0].as_ref().and_then(|t| t.write_mask),
                Some(GpuColorWrite::ALL),
                "guest must pass write-mask=all"
            );
            assert_eq!(fragment.entry_point.as_deref(), Some("fs_main"));
            assert_eq!(descriptor.label.as_deref(), Some("wg6"));
            pipeline.store(true, Ordering::SeqCst);
            let resource = caller
                .data_mut()
                .table
                .push(GpuRenderPipeline { rep: 71 })?;
            Ok((resource,))
        }
    })?;
    webgpu.func_wrap(
        "[method]gpu-texture.create-view",
        |mut caller,
         (texture, descriptor): (Resource<GpuTexture>, Option<GpuTextureViewDescriptor>)| {
            caller.data_mut().table.get(&texture).map(|_| ())?;
            assert!(descriptor.is_none(), "guest must pass view descriptor none");
            let resource = caller.data_mut().table.push(GpuTextureView { rep: 41 })?;
            Ok((resource,))
        },
    )?;
    webgpu.func_wrap(
        "[method]gpu-device.create-command-encoder",
        |mut caller,
         (device, descriptor): (Resource<GpuDevice>, Option<GpuCommandEncoderDescriptor>)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert!(descriptor.is_none());
            let resource = caller
                .data_mut()
                .table
                .push(GpuCommandEncoder { rep: 17 })?;
            Ok((resource,))
        },
    )?;
    webgpu.func_wrap(
        "[method]gpu-device.queue",
        |mut caller, (device,): (Resource<GpuDevice>,)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            let resource = caller.data_mut().table.push(GpuQueue { rep: 3 })?;
            Ok((resource,))
        },
    )?;
    webgpu.func_wrap(
        "[method]gpu-command-encoder.begin-render-pass",
        |mut caller,
         (encoder, descriptor): (Resource<GpuCommandEncoder>, GpuRenderPassDescriptor)| {
            caller.data_mut().table.get(&encoder).map(|_| ())?;
            assert_eq!(descriptor.color_attachments.len(), 1);
            let att = descriptor.color_attachments[0]
                .as_ref()
                .expect("guest color-attachment must be some");
            caller.data_mut().table.get(&att.view).map(|_| ())?;
            assert!(matches!(att.load_op, GpuLoadOp::Clear));
            assert!(matches!(att.store_op, GpuStoreOp::Store));
            let clear = att
                .clear_value
                .as_ref()
                .expect("guest must pass color clear-value");
            assert_eq!(clear.r, 0.0);
            assert_eq!(clear.g, 0.0);
            assert_eq!(clear.b, 0.0);
            assert_eq!(clear.a, 1.0);
            assert!(descriptor.depth_stencil_attachment.is_none());
            let resource = caller
                .data_mut()
                .table
                .push(GpuRenderPassEncoder { rep: 29 })?;
            Ok((resource,))
        },
    )?;
    webgpu.func_wrap(
        "[method]gpu-render-pass-encoder.set-pipeline",
        |mut caller,
         (pass, pipeline): (Resource<GpuRenderPassEncoder>, Resource<GpuRenderPipeline>)| {
            caller.data_mut().table.get(&pass).map(|_| ())?;
            caller.data_mut().table.get(&pipeline).map(|_| ())?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("[method]gpu-render-pass-encoder.set-vertex-buffer", {
        let set_vertex_buffer = flags.set_vertex_buffer.clone();
        move |mut caller,
              (pass, slot, buffer, offset, size): (
            Resource<GpuRenderPassEncoder>,
            u32,
            Option<Resource<GpuBuffer>>,
            Option<u64>,
            Option<u64>,
        )| {
            caller.data_mut().table.get(&pass).map(|_| ())?;
            assert_eq!(slot, 0);
            let buffer = buffer.expect("guest must pass buffer=some");
            caller.data_mut().table.get(&buffer).map(|_| ())?;
            assert!(offset.is_none());
            assert!(size.is_none());
            set_vertex_buffer.store(true, Ordering::SeqCst);
            Ok(())
        }
    })?;
    webgpu.func_wrap("[method]gpu-render-pass-encoder.draw", {
        let draw = flags.draw.clone();
        move |mut caller,
              (pass, vertex_count, instance_count, first_vertex, first_instance): (
            Resource<GpuRenderPassEncoder>,
            u32,
            Option<u32>,
            Option<u32>,
            Option<u32>,
        )| {
            caller.data_mut().table.get(&pass).map(|_| ())?;
            assert_eq!(vertex_count, 3);
            assert!(instance_count.is_none());
            assert!(first_vertex.is_none());
            assert!(first_instance.is_none());
            draw.store(true, Ordering::SeqCst);
            Ok(())
        }
    })?;
    webgpu.func_wrap(
        "[method]gpu-render-pass-encoder.end",
        |mut caller, (pass,): (Resource<GpuRenderPassEncoder>,)| {
            caller.data_mut().table.get(&pass).map(|_| ())?;
            Ok(())
        },
    )?;
    webgpu.func_wrap(
        "[method]gpu-command-encoder.finish",
        |mut caller,
         (encoder, descriptor): (
            Resource<GpuCommandEncoder>,
            Option<GpuCommandBufferDescriptor>,
        )| {
            caller.data_mut().table.get(&encoder).map(|_| ())?;
            assert!(descriptor.is_none());
            let resource = caller.data_mut().table.push(GpuCommandBuffer { rep: 19 })?;
            Ok((resource,))
        },
    )?;
    webgpu.func_wrap("[method]gpu-queue.submit", {
        let submitted = flags.submitted.clone();
        move |mut caller,
              (queue, commands): (Resource<GpuQueue>, Vec<Resource<GpuCommandBuffer>>)| {
            caller.data_mut().table.get(&queue).map(|_| ())?;
            assert_eq!(commands.len(), 1);
            caller.data_mut().table.get(&commands[0]).map(|_| ())?;
            submitted.store(true, Ordering::SeqCst);
            Ok(())
        }
    })?;
    Ok(())
}

fn new_store(engine: &Engine) -> Store<TestHost> {
    Store::new(
        engine,
        TestHost {
            table: ResourceTable::new(),
        },
    )
}

fn flags() -> Flags {
    Flags {
        shader: Arc::new(AtomicBool::new(false)),
        buffer: Arc::new(AtomicBool::new(false)),
        pipeline: Arc::new(AtomicBool::new(false)),
        set_vertex_buffer: Arc::new(AtomicBool::new(false)),
        draw: Arc::new(AtomicBool::new(false)),
        submitted: Arc::new(AtomicBool::new(false)),
        current_texture: Arc::new(AtomicBool::new(false)),
        preferred: Arc::new(AtomicBool::new(false)),
    }
}

fn assert_chain(flags: &Flags) {
    assert!(flags.shader.load(Ordering::SeqCst), "create-shader-module");
    assert!(flags.buffer.load(Ordering::SeqCst), "create-buffer VERTEX");
    assert!(
        flags.pipeline.load(Ordering::SeqCst),
        "create-render-pipeline float32x3"
    );
    assert!(
        flags.set_vertex_buffer.load(Ordering::SeqCst),
        "set-vertex-buffer"
    );
    assert!(flags.draw.load(Ordering::SeqCst), "draw 3");
    assert!(
        flags.current_texture.load(Ordering::SeqCst),
        "get-current-texture"
    );
    assert!(
        flags.preferred.load(Ordering::SeqCst),
        "get-preferred-canvas-format"
    );
    assert!(flags.submitted.load(Ordering::SeqCst), "submit");
}

#[test]
fn wasi_webgpu_method_dawn_guest_canvas_present_smoke() -> wasmtime::Result<()> {
    let wat = include_str!("../../../fixtures/w1/webgpu_method_dawn_guest_canvas_present.wat");
    assert!(
        wat.contains("[method]gpu-canvas-context.get-current-texture"),
        "fixture must acquire canvas swapchain texture"
    );
    assert!(
        wat.contains("[method]gpu.get-preferred-canvas-format"),
        "fixture must query preferred canvas format"
    );
    assert!(
        !wat.contains("[method]gpu-device.create-texture"),
        "not create-texture 1x1 cite"
    );
    assert!(
        wat.contains("@location(0)"),
        "fixture WGSL uses vertex attribute"
    );
    assert!(
        !wat.contains("@builtin(vertex_index)"),
        "not vertex_index stub"
    );
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_dawn_guest_canvas_present.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let flags = flags();
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_dawn_guest_canvas_present(&mut linker, clone_flags(&flags))?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let v = pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                let func = accessor
                    .with(|mut access| instance.get_typed_func::<(), (u32,)>(&mut access, "run"))?;
                let (value,) = func.call_concurrent(accessor, ()).await?;
                Ok(value)
            })
            .await?
    })?;
    assert_eq!(v, 1, "guest run must drop owns and return harness 1");
    assert_chain(&flags);
    Ok(())
}

#[test]
fn wasi_webgpu_method_dawn_guest_canvas_present_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_dawn_guest_canvas_present.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let flags = flags();
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_dawn_guest_canvas_present(&mut linker, clone_flags(&flags))?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 1, "guest run must return harness 1 via call_async");
    assert_chain(&flags);
    Ok(())
}
