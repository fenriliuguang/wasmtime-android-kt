//! S6+: `get-device` + `get-shader-module` + `[method]gpu-device.create-render-pipeline-async`
//! WIT: async `(borrow<gpu-device>, gpu-render-pipeline-descriptor)
//!      -> result<own<gpu-render-pipeline>, create-pipeline-error>`.
//! Guest passes shader borrow, layout=auto, other options none; drops own on ok;
//! `run` returns harness 1. True CM async.

use futures::channel::oneshot;
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

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuPipelineErrorReason {
    #[component(name = "validation")]
    Validation,
    #[component(name = "internal")]
    Internal,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum CreatePipelineErrorKind {
    #[component(name = "gpu-pipeline-error")]
    GpuPipelineError(GpuPipelineErrorReason),
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct CreatePipelineError {
    kind: CreatePipelineErrorKind,
    message: String,
}

struct TestHost {
    table: ResourceTable,
}

fn register_method_create_render_pipeline(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    webgpu.resource(
        "gpu-device",
        ResourceType::host::<GpuDevice>(),
        |mut store, rep| {
            let resource = Resource::<GpuDevice>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-shader-module",
        ResourceType::host::<GpuShaderModule>(),
        |mut store, rep| {
            let resource = Resource::<GpuShaderModule>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-pipeline-layout",
        ResourceType::host::<GpuPipelineLayout>(),
        |mut store, rep| {
            let resource = Resource::<GpuPipelineLayout>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-render-pipeline",
        ResourceType::host::<GpuRenderPipeline>(),
        |mut store, rep| {
            let resource = Resource::<GpuRenderPipeline>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "record-gpu-pipeline-constant-value",
        ResourceType::host::<RecordGpuPipelineConstantValue>(),
        |mut store, rep| {
            let resource = Resource::<RecordGpuPipelineConstantValue>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-device", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuDevice { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-shader-module", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuShaderModule { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap_concurrent(
        "[method]gpu-device.create-render-pipeline-async",
        |accessor, (device, descriptor): (Resource<GpuDevice>, GpuRenderPipelineDescriptor)| {
            Box::pin(async move {
                accessor.with(|mut access| access.data_mut().table.get(&device).map(|_| ()))?;
                accessor.with(|mut access| {
                    access
                        .data_mut()
                        .table
                        .get(&descriptor.vertex.module)
                        .map(|_| ())
                })?;
                assert!(matches!(descriptor.layout, GpuLayoutMode::Auto));
                assert!(descriptor.vertex.buffers.is_none());
                assert!(descriptor.vertex.entry_point.is_none());
                assert!(descriptor.vertex.constants.is_none());
                assert!(descriptor.primitive.is_none());
                assert!(descriptor.depth_stencil.is_none());
                assert!(descriptor.multisample.is_none());
                assert!(descriptor.fragment.is_none());
                assert!(descriptor.label.is_none());
                let (tx, rx) = oneshot::channel::<()>();
                std::thread::spawn(move || {
                    let _ = tx.send(());
                });
                let _ = rx.await;
                let resource = accessor.with(|mut access| {
                    access.data_mut().table.push(GpuRenderPipeline { rep: 71 })
                })?;
                Ok((Ok::<_, CreatePipelineError>(resource),))
            })
        },
    )?;
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

#[test]
fn wasi_webgpu_method_create_render_pipeline_async_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_render_pipeline_async.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_render_pipeline(&mut linker)?;
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
    assert_eq!(
        v, 1,
        "guest run must drop own<gpu-render-pipeline> on ok and return harness 1"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_create_render_pipeline_async_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_render_pipeline_async.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_render_pipeline(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 1,
        "guest run must drop own<gpu-render-pipeline> on ok and return harness 1 via call_async"
    );
    Ok(())
}
