//! Dawn C `webgpu.h` loader + pin-method wrappers.
//!
//! `libwebgpu_dawn.so` is optional (Cloud / missing recipe). Desktop tests stay
//! table-backed. This module must not import `jni`.

#![allow(non_camel_case_types, dead_code)]

use std::ffi::c_void;
use std::sync::OnceLock;

use crate::native_gpu::{DawnSlot, ResourceKind};

pub type WgpuObj = *mut c_void;
pub type WgpuEnum = u32;
pub type WgpuBool = u32;
pub type WgpuFlags = u64;

const RTLD_NOW: i32 = 2;
const WGPU_STRLEN: usize = usize::MAX;
pub const WGPU_WHOLE_SIZE: u64 = u64::MAX;
pub const WGPU_DEPTH_SLICE_UNDEFINED: u32 = u32::MAX;
const WGPU_MIP_UNDEFINED: u32 = u32::MAX;
const WGPU_ARRAY_UNDEFINED: u32 = u32::MAX;

const STYPE_SHADER_WGSL: WgpuEnum = 0x0000_0002;
const STYPE_SURFACE_ANDROID: WgpuEnum = 0x0000_0008;

const CALLBACK_WAIT_ANY: WgpuEnum = 0x0000_0001;
const CALLBACK_PROCESS_EVENTS: WgpuEnum = 0x0000_0002;
const INSTANCE_TIMED_WAIT_ANY: WgpuEnum = 0x0000_0001;
const BACKEND_VULKAN: WgpuEnum = 0x0000_0006;
const STATUS_SUCCESS: WgpuEnum = 0x0000_0001;
const SURFACE_OK_OPTIMAL: WgpuEnum = 0x0000_0001;
const SURFACE_OK_SUBOPTIMAL: WgpuEnum = 0x0000_0002;
const PRESENT_FIFO: WgpuEnum = 0x0000_0001;
pub const ALPHA_AUTO: WgpuEnum = 0x0000_0000;
const FORMAT_RGBA8_UNORM: WgpuEnum = 0x0000_0016;
const USAGE_RENDER_ATTACHMENT: WgpuFlags = 1 << 4;
const COLOR_WRITE_ALL: WgpuFlags = 0xF;
const WGPU_COPY_STRIDE_UNDEFINED: u32 = u32::MAX;
const WGPU_QUERY_INDEX_UNDEFINED: u32 = u32::MAX;
const SAMPLER_FILTERING: WgpuEnum = 0x0000_0001;
const SAMPLER_NON_FILTERING: WgpuEnum = 0x0000_0002;
const SAMPLER_COMPARISON: WgpuEnum = 0x0000_0003;
const TEX_SAMPLE_FLOAT: WgpuEnum = 0x0000_0001;
const TEX_SAMPLE_UNFILTERABLE: WgpuEnum = 0x0000_0002;
const TEX_SAMPLE_DEPTH: WgpuEnum = 0x0000_0003;
const TEX_SAMPLE_SINT: WgpuEnum = 0x0000_0004;
const TEX_SAMPLE_UINT: WgpuEnum = 0x0000_0005;
const BINDING_UNIFORM: WgpuEnum = 0x0000_0002;
const BINDING_STORAGE: WgpuEnum = 0x0000_0003;
const BINDING_RO_STORAGE: WgpuEnum = 0x0000_0004;
const TOPOLOGY_TRIANGLE_LIST: WgpuEnum = 0x0000_0004;
const FRONT_CCW: WgpuEnum = 0x0000_0001;
const CULL_BACK: WgpuEnum = 0x0000_0003;
const LOAD_CLEAR: WgpuEnum = 0x0000_0002;
const STORE_STORE: WgpuEnum = 0x0000_0001;
const STEP_VERTEX: WgpuEnum = 0x0000_0001;

#[repr(C)]
#[derive(Clone, Copy)]
struct Chained {
    next: *mut Chained,
    s_type: WgpuEnum,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct StringView {
    data: *const i8,
    length: usize,
}

impl StringView {
    fn empty() -> Self {
        Self {
            data: std::ptr::null(),
            length: WGPU_STRLEN,
        }
    }

    fn from_str(s: &str) -> Self {
        if s.is_empty() {
            return Self::empty();
        }
        Self {
            data: s.as_ptr() as *const i8,
            length: s.len(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Future {
    id: u64,
}

#[repr(C)]
struct FutureWaitInfo {
    future: Future,
    completed: WgpuBool,
}

#[repr(C)]
struct CallbackInfo {
    next_in_chain: *mut Chained,
    mode: WgpuEnum,
    callback: *const c_void,
    userdata1: *mut c_void,
    userdata2: *mut c_void,
}

#[repr(C)]
struct UncapturedInfo {
    next_in_chain: *mut Chained,
    callback: *const c_void,
    userdata1: *mut c_void,
    userdata2: *mut c_void,
}

#[repr(C)]
struct InstanceDesc {
    next_in_chain: *mut Chained,
    required_feature_count: usize,
    required_features: *const WgpuEnum,
    required_limits: *const c_void,
}

#[repr(C)]
struct RequestAdapterOptions {
    next_in_chain: *mut Chained,
    feature_level: WgpuEnum,
    power_preference: WgpuEnum,
    force_fallback_adapter: WgpuBool,
    backend_type: WgpuEnum,
    compatible_surface: WgpuObj,
}

#[repr(C)]
struct QueueDesc {
    next_in_chain: *mut Chained,
    label: StringView,
}

#[repr(C)]
struct DeviceDesc {
    next_in_chain: *mut Chained,
    label: StringView,
    required_feature_count: usize,
    required_features: *const WgpuEnum,
    required_limits: *const c_void,
    default_queue: QueueDesc,
    device_lost: CallbackInfo,
    uncaptured: UncapturedInfo,
}

#[repr(C)]
struct BufferDesc {
    next_in_chain: *mut Chained,
    label: StringView,
    usage: WgpuFlags,
    size: u64,
    mapped_at_creation: WgpuBool,
}

#[repr(C)]
struct ShaderWgsl {
    chain: Chained,
    code: StringView,
}

#[repr(C)]
struct ShaderDesc {
    next_in_chain: *mut Chained,
    label: StringView,
}

#[repr(C)]
struct BufferBindingLayout {
    next_in_chain: *mut Chained,
    ty: WgpuEnum,
    has_dynamic_offset: WgpuBool,
    min_binding_size: u64,
}

#[repr(C)]
struct SamplerBindingLayout {
    next_in_chain: *mut Chained,
    ty: WgpuEnum,
}

#[repr(C)]
struct TextureBindingLayout {
    next_in_chain: *mut Chained,
    sample_type: WgpuEnum,
    view_dimension: WgpuEnum,
    multisampled: WgpuBool,
}

#[repr(C)]
struct StorageTextureBindingLayout {
    next_in_chain: *mut Chained,
    access: WgpuEnum,
    format: WgpuEnum,
    view_dimension: WgpuEnum,
}

#[repr(C)]
struct BindGroupLayoutEntry {
    next_in_chain: *mut Chained,
    binding: u32,
    visibility: WgpuFlags,
    binding_array_size: u32,
    buffer: BufferBindingLayout,
    sampler: SamplerBindingLayout,
    texture: TextureBindingLayout,
    storage_texture: StorageTextureBindingLayout,
}

#[repr(C)]
struct BindGroupLayoutDesc {
    next_in_chain: *mut Chained,
    label: StringView,
    entry_count: usize,
    entries: *const BindGroupLayoutEntry,
}

#[repr(C)]
struct PipelineLayoutDesc {
    next_in_chain: *mut Chained,
    label: StringView,
    bind_group_layout_count: usize,
    bind_group_layouts: *const WgpuObj,
    immediate_size: u32,
}

#[repr(C)]
struct BindGroupEntry {
    next_in_chain: *mut Chained,
    binding: u32,
    buffer: WgpuObj,
    offset: u64,
    size: u64,
    sampler: WgpuObj,
    texture_view: WgpuObj,
}

#[repr(C)]
struct BindGroupDesc {
    next_in_chain: *mut Chained,
    label: StringView,
    layout: WgpuObj,
    entry_count: usize,
    entries: *const BindGroupEntry,
}

#[repr(C)]
struct VertexAttribute {
    next_in_chain: *mut Chained,
    format: WgpuEnum,
    offset: u64,
    shader_location: u32,
}

#[repr(C)]
struct VertexBufferLayout {
    next_in_chain: *mut Chained,
    step_mode: WgpuEnum,
    array_stride: u64,
    attribute_count: usize,
    attributes: *const VertexAttribute,
}

#[repr(C)]
struct VertexState {
    next_in_chain: *mut Chained,
    module: WgpuObj,
    entry_point: StringView,
    constant_count: usize,
    constants: *const ConstantEntry,
    buffer_count: usize,
    buffers: *const VertexBufferLayout,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct BlendComponent {
    operation: WgpuEnum,
    src_factor: WgpuEnum,
    dst_factor: WgpuEnum,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct BlendState {
    color: BlendComponent,
    alpha: BlendComponent,
}

#[repr(C)]
struct ColorTargetState {
    next_in_chain: *mut Chained,
    format: WgpuEnum,
    blend: *const BlendState,
    write_mask: WgpuFlags,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct StencilFaceState {
    compare: WgpuEnum,
    fail_op: WgpuEnum,
    depth_fail_op: WgpuEnum,
    pass_op: WgpuEnum,
}

#[repr(C)]
struct DepthStencilState {
    next_in_chain: *mut Chained,
    format: WgpuEnum,
    /// `WGPUOptionalBool`: False=0, True=1, Undefined=2.
    depth_write_enabled: WgpuEnum,
    depth_compare: WgpuEnum,
    stencil_front: StencilFaceState,
    stencil_back: StencilFaceState,
    stencil_read_mask: u32,
    stencil_write_mask: u32,
    depth_bias: i32,
    depth_bias_slope_scale: f32,
    depth_bias_clamp: f32,
}

#[repr(C)]
struct ConstantEntry {
    next_in_chain: *mut Chained,
    key: StringView,
    value: f64,
}

#[repr(C)]
struct FragmentState {
    next_in_chain: *mut Chained,
    module: WgpuObj,
    entry_point: StringView,
    constant_count: usize,
    constants: *const ConstantEntry,
    target_count: usize,
    targets: *const ColorTargetState,
}

#[repr(C)]
struct PrimitiveState {
    next_in_chain: *mut Chained,
    topology: WgpuEnum,
    strip_index_format: WgpuEnum,
    front_face: WgpuEnum,
    cull_mode: WgpuEnum,
    unclipped_depth: WgpuBool,
}

#[repr(C)]
struct MultisampleState {
    next_in_chain: *mut Chained,
    count: u32,
    mask: u32,
    alpha_to_coverage_enabled: WgpuBool,
}

#[repr(C)]
struct RenderPipelineDesc {
    next_in_chain: *mut Chained,
    label: StringView,
    layout: WgpuObj,
    vertex: VertexState,
    primitive: PrimitiveState,
    depth_stencil: *const DepthStencilState,
    multisample: MultisampleState,
    fragment: *const FragmentState,
}

#[repr(C)]
struct CommandEncoderDesc {
    next_in_chain: *mut Chained,
    label: StringView,
}

#[repr(C)]
struct CommandBufferDesc {
    next_in_chain: *mut Chained,
    label: StringView,
}

#[repr(C)]
struct Color {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

#[repr(C)]
struct RenderPassColorAttachment {
    next_in_chain: *mut Chained,
    view: WgpuObj,
    depth_slice: u32,
    resolve_target: WgpuObj,
    load_op: WgpuEnum,
    store_op: WgpuEnum,
    clear_value: Color,
}

#[repr(C)]
struct RenderPassDesc {
    next_in_chain: *mut Chained,
    label: StringView,
    color_attachment_count: usize,
    color_attachments: *const RenderPassColorAttachment,
    depth_stencil_attachment: *const c_void,
    occlusion_query_set: WgpuObj,
    timestamp_writes: *const c_void,
}

#[repr(C)]
struct SurfaceAndroid {
    chain: Chained,
    window: *mut c_void,
}

#[repr(C)]
struct SurfaceDesc {
    next_in_chain: *mut Chained,
    label: StringView,
}

#[repr(C)]
struct SurfaceConfig {
    next_in_chain: *mut Chained,
    device: WgpuObj,
    format: WgpuEnum,
    usage: WgpuFlags,
    width: u32,
    height: u32,
    view_format_count: usize,
    view_formats: *const WgpuEnum,
    alpha_mode: WgpuEnum,
    present_mode: WgpuEnum,
}

#[repr(C)]
struct SurfaceTexture {
    next_in_chain: *mut Chained,
    texture: WgpuObj,
    status: WgpuEnum,
}

#[repr(C)]
struct SurfaceCaps {
    next_in_chain: *mut Chained,
    usages: WgpuFlags,
    format_count: usize,
    formats: *const WgpuEnum,
    present_mode_count: usize,
    present_modes: *const WgpuEnum,
    alpha_mode_count: usize,
    alpha_modes: *const WgpuEnum,
}

#[repr(C)]
struct TextureViewDesc {
    next_in_chain: *mut Chained,
    label: StringView,
    format: WgpuEnum,
    dimension: WgpuEnum,
    base_mip_level: u32,
    mip_level_count: u32,
    base_array_layer: u32,
    array_layer_count: u32,
    aspect: WgpuEnum,
    usage: WgpuFlags,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Extent3D {
    width: u32,
    height: u32,
    depth: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Origin3D {
    x: u32,
    y: u32,
    z: u32,
}

#[repr(C)]
struct TextureDesc {
    next_in_chain: *mut Chained,
    label: StringView,
    usage: WgpuFlags,
    dimension: WgpuEnum,
    size: Extent3D,
    format: WgpuEnum,
    mip_level_count: u32,
    sample_count: u32,
    view_format_count: usize,
    view_formats: *const WgpuEnum,
}

#[repr(C)]
struct SamplerDesc {
    next_in_chain: *mut Chained,
    label: StringView,
    address_mode_u: WgpuEnum,
    address_mode_v: WgpuEnum,
    address_mode_w: WgpuEnum,
    mag_filter: WgpuEnum,
    min_filter: WgpuEnum,
    mipmap_filter: WgpuEnum,
    lod_min_clamp: f32,
    lod_max_clamp: f32,
    compare: WgpuEnum,
    max_anisotropy: u16,
}

#[repr(C)]
struct ComputeState {
    next_in_chain: *mut Chained,
    module: WgpuObj,
    entry_point: StringView,
    constant_count: usize,
    constants: *const ConstantEntry,
}

#[repr(C)]
struct ComputePipelineDesc {
    next_in_chain: *mut Chained,
    label: StringView,
    layout: WgpuObj,
    compute: ComputeState,
}

#[repr(C)]
struct QuerySetDesc {
    next_in_chain: *mut Chained,
    label: StringView,
    ty: WgpuEnum,
    count: u32,
}

#[repr(C)]
struct TimestampWrites {
    next_in_chain: *mut Chained,
    query_set: WgpuObj,
    beginning: u32,
    end: u32,
}

#[repr(C)]
struct ComputePassDesc {
    next_in_chain: *mut Chained,
    label: StringView,
    timestamp_writes: *const TimestampWrites,
}

#[repr(C)]
struct TexelCopyBufferLayout {
    offset: u64,
    bytes_per_row: u32,
    rows_per_image: u32,
}

#[repr(C)]
struct TexelCopyBufferInfo {
    layout: TexelCopyBufferLayout,
    buffer: WgpuObj,
}

#[repr(C)]
struct TexelCopyTextureInfo {
    texture: WgpuObj,
    mip_level: u32,
    origin: Origin3D,
    aspect: WgpuEnum,
}

#[repr(C)]
struct RenderBundleEncDesc {
    next_in_chain: *mut Chained,
    label: StringView,
    color_format_count: usize,
    color_formats: *const WgpuEnum,
    depth_stencil_format: WgpuEnum,
    sample_count: u32,
    depth_read_only: WgpuBool,
    stencil_read_only: WgpuBool,
}

#[repr(C)]
struct RenderBundleDesc {
    next_in_chain: *mut Chained,
    label: StringView,
}

#[repr(C)]
struct DepthStencilAtt {
    next_in_chain: *mut Chained,
    view: WgpuObj,
    depth_load_op: WgpuEnum,
    depth_store_op: WgpuEnum,
    depth_clear: f32,
    depth_read_only: WgpuBool,
    stencil_load_op: WgpuEnum,
    stencil_store_op: WgpuEnum,
    stencil_clear: u32,
    stencil_read_only: WgpuBool,
}

macro_rules! procs {
    ($($name:ident : $ty:ty),+ $(,)?) => {
        pub struct Api {
            $($name: $ty,)+
        }
    };
}

type FnCreateInstance = unsafe extern "C" fn(*const InstanceDesc) -> WgpuObj;
type FnProcessEvents = unsafe extern "C" fn(WgpuObj);
type FnWaitAny = unsafe extern "C" fn(WgpuObj, usize, *mut FutureWaitInfo, u64) -> WgpuEnum;
type FnRequestAdapter =
    unsafe extern "C" fn(WgpuObj, *const RequestAdapterOptions, CallbackInfo) -> Future;
type FnRequestDevice = unsafe extern "C" fn(WgpuObj, *const DeviceDesc, CallbackInfo) -> Future;
type FnGetQueue = unsafe extern "C" fn(WgpuObj) -> WgpuObj;
type FnCreateBuffer = unsafe extern "C" fn(WgpuObj, *const BufferDesc) -> WgpuObj;
type FnCreateShader = unsafe extern "C" fn(WgpuObj, *const ShaderDesc) -> WgpuObj;
type FnCreateBgl = unsafe extern "C" fn(WgpuObj, *const BindGroupLayoutDesc) -> WgpuObj;
type FnCreatePl = unsafe extern "C" fn(WgpuObj, *const PipelineLayoutDesc) -> WgpuObj;
type FnCreateBg = unsafe extern "C" fn(WgpuObj, *const BindGroupDesc) -> WgpuObj;
type FnCreateRp = unsafe extern "C" fn(WgpuObj, *const RenderPipelineDesc) -> WgpuObj;
type FnCreateEnc = unsafe extern "C" fn(WgpuObj, *const CommandEncoderDesc) -> WgpuObj;
type FnBeginRp = unsafe extern "C" fn(WgpuObj, *const RenderPassDesc) -> WgpuObj;
type FnFinishEnc = unsafe extern "C" fn(WgpuObj, *const CommandBufferDesc) -> WgpuObj;
type FnWriteBuffer = unsafe extern "C" fn(WgpuObj, WgpuObj, u64, *const u8, usize);
type FnSubmit = unsafe extern "C" fn(WgpuObj, usize, *const WgpuObj);
type FnCreateSurface = unsafe extern "C" fn(WgpuObj, *const SurfaceDesc) -> WgpuObj;
type FnSurfaceConfigure = unsafe extern "C" fn(WgpuObj, *const SurfaceConfig);
type FnSurfaceGetTex = unsafe extern "C" fn(WgpuObj, *mut SurfaceTexture);
type FnSurfacePresent = unsafe extern "C" fn(WgpuObj) -> WgpuEnum;
type FnSurfaceCaps = unsafe extern "C" fn(WgpuObj, WgpuObj, *mut SurfaceCaps) -> WgpuEnum;
type FnCapsFree = unsafe extern "C" fn(SurfaceCaps);
type FnTexView = unsafe extern "C" fn(WgpuObj, *const TextureViewDesc) -> WgpuObj;
type FnTexWidth = unsafe extern "C" fn(WgpuObj) -> u32;
type FnTexHeight = unsafe extern "C" fn(WgpuObj) -> u32;
type FnPassSetPipeline = unsafe extern "C" fn(WgpuObj, WgpuObj);
type FnPassSetBg = unsafe extern "C" fn(WgpuObj, u32, WgpuObj, usize, *const u32);
type FnPassSetVb = unsafe extern "C" fn(WgpuObj, u32, WgpuObj, u64, u64);
type FnPassDraw = unsafe extern "C" fn(WgpuObj, u32, u32, u32, u32);
type FnPassEnd = unsafe extern "C" fn(WgpuObj);
type FnRelease = unsafe extern "C" fn(WgpuObj);
type FnCreateTexture = unsafe extern "C" fn(WgpuObj, *const TextureDesc) -> WgpuObj;
type FnCreateSampler = unsafe extern "C" fn(WgpuObj, *const SamplerDesc) -> WgpuObj;
type FnCreateCp = unsafe extern "C" fn(WgpuObj, *const ComputePipelineDesc) -> WgpuObj;
type FnCreateQs = unsafe extern "C" fn(WgpuObj, *const QuerySetDesc) -> WgpuObj;
type FnCreateBundleEnc = unsafe extern "C" fn(WgpuObj, *const RenderBundleEncDesc) -> WgpuObj;
type FnBeginCp = unsafe extern "C" fn(WgpuObj, *const ComputePassDesc) -> WgpuObj;
type FnCopyB2B = unsafe extern "C" fn(WgpuObj, WgpuObj, u64, WgpuObj, u64, u64);
type FnCopyB2T = unsafe extern "C" fn(
    WgpuObj,
    *const TexelCopyBufferInfo,
    *const TexelCopyTextureInfo,
    *const Extent3D,
);
type FnCopyT2B = unsafe extern "C" fn(
    WgpuObj,
    *const TexelCopyTextureInfo,
    *const TexelCopyBufferInfo,
    *const Extent3D,
);
type FnCopyT2T = unsafe extern "C" fn(
    WgpuObj,
    *const TexelCopyTextureInfo,
    *const TexelCopyTextureInfo,
    *const Extent3D,
);
type FnClearBuf = unsafe extern "C" fn(WgpuObj, WgpuObj, u64, u64);
type FnResolveQs = unsafe extern "C" fn(WgpuObj, WgpuObj, u32, u32, WgpuObj, u64);
type FnWriteTex = unsafe extern "C" fn(
    WgpuObj,
    *const TexelCopyTextureInfo,
    *const u8,
    usize,
    *const TexelCopyBufferLayout,
    *const Extent3D,
);
type FnWorkDone = unsafe extern "C" fn(WgpuObj, CallbackInfo) -> Future;
type FnMapAsync = unsafe extern "C" fn(WgpuObj, WgpuFlags, usize, usize, CallbackInfo) -> Future;
type FnUnmap = unsafe extern "C" fn(WgpuObj);
type FnMappedRange = unsafe extern "C" fn(WgpuObj, usize, usize) -> *const u8;
type FnDestroy = unsafe extern "C" fn(WgpuObj);
type FnHasFeature = unsafe extern "C" fn(WgpuObj, WgpuEnum) -> WgpuBool;
type FnViewport = unsafe extern "C" fn(WgpuObj, f32, f32, f32, f32, f32, f32);
type FnScissor = unsafe extern "C" fn(WgpuObj, u32, u32, u32, u32);
type FnBlend = unsafe extern "C" fn(WgpuObj, *const Color);
type FnStencil = unsafe extern "C" fn(WgpuObj, u32);
type FnSetIndex = unsafe extern "C" fn(WgpuObj, WgpuObj, WgpuEnum, u64, u64);
type FnDrawIndexed = unsafe extern "C" fn(WgpuObj, u32, u32, u32, i32, u32);
type FnDrawIndirect = unsafe extern "C" fn(WgpuObj, WgpuObj, u64);
type FnOcclusion = unsafe extern "C" fn(WgpuObj, u32);
type FnExecBundles = unsafe extern "C" fn(WgpuObj, usize, *const WgpuObj);
type FnDispatch = unsafe extern "C" fn(WgpuObj, u32, u32, u32);
type FnDispatchIndirect = unsafe extern "C" fn(WgpuObj, WgpuObj, u64);
type FnBundleFinish = unsafe extern "C" fn(WgpuObj, *const RenderBundleDesc) -> WgpuObj;
type FnGetBgl = unsafe extern "C" fn(WgpuObj, u32) -> WgpuObj;
type FnPushError = unsafe extern "C" fn(WgpuObj, WgpuEnum);

procs! {
    create_instance: FnCreateInstance,
    process_events: FnProcessEvents,
    wait_any: FnWaitAny,
    request_adapter: FnRequestAdapter,
    request_device: FnRequestDevice,
    device_get_queue: FnGetQueue,
    create_buffer: FnCreateBuffer,
    create_shader: FnCreateShader,
    create_bgl: FnCreateBgl,
    create_pl: FnCreatePl,
    create_bg: FnCreateBg,
    create_rp: FnCreateRp,
    create_encoder: FnCreateEnc,
    begin_render_pass: FnBeginRp,
    encoder_finish: FnFinishEnc,
    write_buffer: FnWriteBuffer,
    queue_submit: FnSubmit,
    create_surface: FnCreateSurface,
    surface_configure: FnSurfaceConfigure,
    surface_get_current: FnSurfaceGetTex,
    surface_present: FnSurfacePresent,
    surface_caps: FnSurfaceCaps,
    caps_free: FnCapsFree,
    texture_create_view: FnTexView,
    texture_width: FnTexWidth,
    texture_height: FnTexHeight,
    pass_set_pipeline: FnPassSetPipeline,
    pass_set_bind_group: FnPassSetBg,
    pass_set_vertex_buffer: FnPassSetVb,
    pass_draw: FnPassDraw,
    pass_end: FnPassEnd,
    release_adapter: FnRelease,
    release_device: FnRelease,
    release_queue: FnRelease,
    release_buffer: FnRelease,
    release_shader: FnRelease,
    release_bgl: FnRelease,
    release_pl: FnRelease,
    release_bg: FnRelease,
    release_rp: FnRelease,
    release_encoder: FnRelease,
    release_pass: FnRelease,
    release_cmd: FnRelease,
    release_surface: FnRelease,
    release_texture: FnRelease,
    release_view: FnRelease,
    release_instance: FnRelease,
    create_texture: FnCreateTexture,
    create_sampler: FnCreateSampler,
    create_compute_pipeline: FnCreateCp,
    create_query_set: FnCreateQs,
    create_bundle_enc: FnCreateBundleEnc,
    begin_compute_pass: FnBeginCp,
    copy_b2b: FnCopyB2B,
    copy_b2t: FnCopyB2T,
    copy_t2b: FnCopyT2B,
    copy_t2t: FnCopyT2T,
    clear_buffer: FnClearBuf,
    resolve_query: FnResolveQs,
    write_texture: FnWriteTex,
    work_done: FnWorkDone,
    buffer_map: FnMapAsync,
    buffer_unmap: FnUnmap,
    buffer_mapped_range: FnMappedRange,
    buffer_destroy: FnDestroy,
    texture_destroy: FnDestroy,
    query_destroy: FnDestroy,
    device_destroy: FnDestroy,
    adapter_has_feature: FnHasFeature,
    pass_set_viewport: FnViewport,
    pass_set_scissor: FnScissor,
    pass_set_blend: FnBlend,
    pass_set_stencil: FnStencil,
    pass_set_index: FnSetIndex,
    pass_draw_indexed: FnDrawIndexed,
    pass_draw_indirect: FnDrawIndirect,
    pass_draw_indexed_indirect: FnDrawIndirect,
    pass_occlusion_begin: FnOcclusion,
    pass_occlusion_end: FnPassEnd,
    pass_execute_bundles: FnExecBundles,
    compute_set_pipeline: FnPassSetPipeline,
    compute_set_bind_group: FnPassSetBg,
    compute_dispatch: FnDispatch,
    compute_dispatch_indirect: FnDispatchIndirect,
    compute_end: FnPassEnd,
    bundle_finish: FnBundleFinish,
    bundle_set_pipeline: FnPassSetPipeline,
    bundle_set_bind_group: FnPassSetBg,
    bundle_set_vb: FnPassSetVb,
    bundle_set_index: FnSetIndex,
    bundle_draw: FnPassDraw,
    bundle_draw_indexed: FnDrawIndexed,
    bundle_draw_indirect: FnDrawIndirect,
    bundle_draw_indexed_indirect: FnDrawIndirect,
    rp_get_bgl: FnGetBgl,
    cp_get_bgl: FnGetBgl,
    push_error: FnPushError,
    pop_error: FnWorkDone,
    release_sampler: FnRelease,
    release_cp: FnRelease,
    release_qs: FnRelease,
    release_compute_pass: FnRelease,
    release_bundle_enc: FnRelease,
    release_bundle: FnRelease,
}

static API: OnceLock<Option<Api>> = OnceLock::new();

fn log_android(ok: bool, msg: &str) {
    let _ = (ok, msg);
    #[cfg(target_os = "android")]
    unsafe {
        extern "C" {
            fn __android_log_write(prio: i32, tag: *const i8, text: *const i8) -> i32;
        }
        let prio = if ok { 4 } else { 6 };
        let c = std::ffi::CString::new(msg).unwrap_or_default();
        let _ = __android_log_write(
            prio,
            c"NativeGpu".as_ptr() as *const i8,
            c.as_ptr() as *const i8,
        );
    }
}

pub fn try_load() -> bool {
    api().is_some()
}

pub fn api() -> Option<&'static Api> {
    API.get_or_init(load_once).as_ref()
}

fn load_once() -> Option<Api> {
    #[cfg(not(target_os = "android"))]
    {
        return None;
    }
    #[cfg(target_os = "android")]
    unsafe {
        extern "C" {
            fn dlopen(filename: *const i8, flags: i32) -> *mut c_void;
            fn dlsym(handle: *mut c_void, symbol: *const i8) -> *mut c_void;
            fn dlerror() -> *const i8;
        }
        let lib = dlopen(c"libwebgpu_dawn.so".as_ptr() as *const i8, RTLD_NOW);
        if lib.is_null() {
            let err = dlerror();
            let msg = if err.is_null() {
                "dlopen libwebgpu_dawn.so failed".to_string()
            } else {
                std::ffi::CStr::from_ptr(err as *const std::ffi::c_char)
                    .to_string_lossy()
                    .into_owned()
            };
            log_android(false, &msg);
            return None;
        }
        let sym = |name: &std::ffi::CStr| dlsym(lib, name.as_ptr() as *const i8);
        let need = |name: &std::ffi::CStr| {
            let p = sym(name);
            if p.is_null() {
                log_android(false, &format!("dlsym {} missing", name.to_string_lossy()));
            }
            p
        };
        let api = Api {
            create_instance: std::mem::transmute(need(c"wgpuCreateInstance")),
            process_events: std::mem::transmute(need(c"wgpuInstanceProcessEvents")),
            wait_any: std::mem::transmute(need(c"wgpuInstanceWaitAny")),
            request_adapter: std::mem::transmute(need(c"wgpuInstanceRequestAdapter")),
            request_device: std::mem::transmute(need(c"wgpuAdapterRequestDevice")),
            device_get_queue: std::mem::transmute(need(c"wgpuDeviceGetQueue")),
            create_buffer: std::mem::transmute(need(c"wgpuDeviceCreateBuffer")),
            create_shader: std::mem::transmute(need(c"wgpuDeviceCreateShaderModule")),
            create_bgl: std::mem::transmute(need(c"wgpuDeviceCreateBindGroupLayout")),
            create_pl: std::mem::transmute(need(c"wgpuDeviceCreatePipelineLayout")),
            create_bg: std::mem::transmute(need(c"wgpuDeviceCreateBindGroup")),
            create_rp: std::mem::transmute(need(c"wgpuDeviceCreateRenderPipeline")),
            create_encoder: std::mem::transmute(need(c"wgpuDeviceCreateCommandEncoder")),
            begin_render_pass: std::mem::transmute(need(c"wgpuCommandEncoderBeginRenderPass")),
            encoder_finish: std::mem::transmute(need(c"wgpuCommandEncoderFinish")),
            write_buffer: std::mem::transmute(need(c"wgpuQueueWriteBuffer")),
            queue_submit: std::mem::transmute(need(c"wgpuQueueSubmit")),
            create_surface: std::mem::transmute(need(c"wgpuInstanceCreateSurface")),
            surface_configure: std::mem::transmute(need(c"wgpuSurfaceConfigure")),
            surface_get_current: std::mem::transmute(need(c"wgpuSurfaceGetCurrentTexture")),
            surface_present: std::mem::transmute(need(c"wgpuSurfacePresent")),
            surface_caps: std::mem::transmute(need(c"wgpuSurfaceGetCapabilities")),
            caps_free: std::mem::transmute(need(c"wgpuSurfaceCapabilitiesFreeMembers")),
            texture_create_view: std::mem::transmute(need(c"wgpuTextureCreateView")),
            texture_width: std::mem::transmute(need(c"wgpuTextureGetWidth")),
            texture_height: std::mem::transmute(need(c"wgpuTextureGetHeight")),
            pass_set_pipeline: std::mem::transmute(need(c"wgpuRenderPassEncoderSetPipeline")),
            pass_set_bind_group: std::mem::transmute(need(c"wgpuRenderPassEncoderSetBindGroup")),
            pass_set_vertex_buffer: std::mem::transmute(need(
                c"wgpuRenderPassEncoderSetVertexBuffer",
            )),
            pass_draw: std::mem::transmute(need(c"wgpuRenderPassEncoderDraw")),
            pass_end: std::mem::transmute(need(c"wgpuRenderPassEncoderEnd")),
            release_adapter: std::mem::transmute(need(c"wgpuAdapterRelease")),
            release_device: std::mem::transmute(need(c"wgpuDeviceRelease")),
            release_queue: std::mem::transmute(need(c"wgpuQueueRelease")),
            release_buffer: std::mem::transmute(need(c"wgpuBufferRelease")),
            release_shader: std::mem::transmute(need(c"wgpuShaderModuleRelease")),
            release_bgl: std::mem::transmute(need(c"wgpuBindGroupLayoutRelease")),
            release_pl: std::mem::transmute(need(c"wgpuPipelineLayoutRelease")),
            release_bg: std::mem::transmute(need(c"wgpuBindGroupRelease")),
            release_rp: std::mem::transmute(need(c"wgpuRenderPipelineRelease")),
            release_encoder: std::mem::transmute(need(c"wgpuCommandEncoderRelease")),
            release_pass: std::mem::transmute(need(c"wgpuRenderPassEncoderRelease")),
            release_cmd: std::mem::transmute(need(c"wgpuCommandBufferRelease")),
            release_surface: std::mem::transmute(need(c"wgpuSurfaceRelease")),
            release_texture: std::mem::transmute(need(c"wgpuTextureRelease")),
            release_view: std::mem::transmute(need(c"wgpuTextureViewRelease")),
            release_instance: std::mem::transmute(need(c"wgpuInstanceRelease")),
            create_texture: std::mem::transmute(need(c"wgpuDeviceCreateTexture")),
            create_sampler: std::mem::transmute(need(c"wgpuDeviceCreateSampler")),
            create_compute_pipeline: std::mem::transmute(need(c"wgpuDeviceCreateComputePipeline")),
            create_query_set: std::mem::transmute(need(c"wgpuDeviceCreateQuerySet")),
            create_bundle_enc: std::mem::transmute(need(c"wgpuDeviceCreateRenderBundleEncoder")),
            begin_compute_pass: std::mem::transmute(need(c"wgpuCommandEncoderBeginComputePass")),
            copy_b2b: std::mem::transmute(need(c"wgpuCommandEncoderCopyBufferToBuffer")),
            copy_b2t: std::mem::transmute(need(c"wgpuCommandEncoderCopyBufferToTexture")),
            copy_t2b: std::mem::transmute(need(c"wgpuCommandEncoderCopyTextureToBuffer")),
            copy_t2t: std::mem::transmute(need(c"wgpuCommandEncoderCopyTextureToTexture")),
            clear_buffer: std::mem::transmute(need(c"wgpuCommandEncoderClearBuffer")),
            resolve_query: std::mem::transmute(need(c"wgpuCommandEncoderResolveQuerySet")),
            write_texture: std::mem::transmute(need(c"wgpuQueueWriteTexture")),
            work_done: std::mem::transmute(need(c"wgpuQueueOnSubmittedWorkDone")),
            buffer_map: std::mem::transmute(need(c"wgpuBufferMapAsync")),
            buffer_unmap: std::mem::transmute(need(c"wgpuBufferUnmap")),
            buffer_mapped_range: std::mem::transmute(need(c"wgpuBufferGetConstMappedRange")),
            buffer_destroy: std::mem::transmute(need(c"wgpuBufferDestroy")),
            texture_destroy: std::mem::transmute(need(c"wgpuTextureDestroy")),
            query_destroy: std::mem::transmute(need(c"wgpuQuerySetDestroy")),
            device_destroy: std::mem::transmute(need(c"wgpuDeviceDestroy")),
            adapter_has_feature: std::mem::transmute(need(c"wgpuAdapterHasFeature")),
            pass_set_viewport: std::mem::transmute(need(c"wgpuRenderPassEncoderSetViewport")),
            pass_set_scissor: std::mem::transmute(need(c"wgpuRenderPassEncoderSetScissorRect")),
            pass_set_blend: std::mem::transmute(need(c"wgpuRenderPassEncoderSetBlendConstant")),
            pass_set_stencil: std::mem::transmute(need(
                c"wgpuRenderPassEncoderSetStencilReference",
            )),
            pass_set_index: std::mem::transmute(need(c"wgpuRenderPassEncoderSetIndexBuffer")),
            pass_draw_indexed: std::mem::transmute(need(c"wgpuRenderPassEncoderDrawIndexed")),
            pass_draw_indirect: std::mem::transmute(need(c"wgpuRenderPassEncoderDrawIndirect")),
            pass_draw_indexed_indirect: std::mem::transmute(need(
                c"wgpuRenderPassEncoderDrawIndexedIndirect",
            )),
            pass_occlusion_begin: std::mem::transmute(need(
                c"wgpuRenderPassEncoderBeginOcclusionQuery",
            )),
            pass_occlusion_end: std::mem::transmute(need(
                c"wgpuRenderPassEncoderEndOcclusionQuery",
            )),
            pass_execute_bundles: std::mem::transmute(need(c"wgpuRenderPassEncoderExecuteBundles")),
            compute_set_pipeline: std::mem::transmute(need(c"wgpuComputePassEncoderSetPipeline")),
            compute_set_bind_group: std::mem::transmute(need(
                c"wgpuComputePassEncoderSetBindGroup",
            )),
            compute_dispatch: std::mem::transmute(need(
                c"wgpuComputePassEncoderDispatchWorkgroups",
            )),
            compute_dispatch_indirect: std::mem::transmute(need(
                c"wgpuComputePassEncoderDispatchWorkgroupsIndirect",
            )),
            compute_end: std::mem::transmute(need(c"wgpuComputePassEncoderEnd")),
            bundle_finish: std::mem::transmute(need(c"wgpuRenderBundleEncoderFinish")),
            bundle_set_pipeline: std::mem::transmute(need(c"wgpuRenderBundleEncoderSetPipeline")),
            bundle_set_bind_group: std::mem::transmute(need(
                c"wgpuRenderBundleEncoderSetBindGroup",
            )),
            bundle_set_vb: std::mem::transmute(need(c"wgpuRenderBundleEncoderSetVertexBuffer")),
            bundle_set_index: std::mem::transmute(need(c"wgpuRenderBundleEncoderSetIndexBuffer")),
            bundle_draw: std::mem::transmute(need(c"wgpuRenderBundleEncoderDraw")),
            bundle_draw_indexed: std::mem::transmute(need(c"wgpuRenderBundleEncoderDrawIndexed")),
            bundle_draw_indirect: std::mem::transmute(need(c"wgpuRenderBundleEncoderDrawIndirect")),
            bundle_draw_indexed_indirect: std::mem::transmute(need(
                c"wgpuRenderBundleEncoderDrawIndexedIndirect",
            )),
            rp_get_bgl: std::mem::transmute(need(c"wgpuRenderPipelineGetBindGroupLayout")),
            cp_get_bgl: std::mem::transmute(need(c"wgpuComputePipelineGetBindGroupLayout")),
            push_error: std::mem::transmute(need(c"wgpuDevicePushErrorScope")),
            pop_error: std::mem::transmute(need(c"wgpuDevicePopErrorScope")),
            release_sampler: std::mem::transmute(need(c"wgpuSamplerRelease")),
            release_cp: std::mem::transmute(need(c"wgpuComputePipelineRelease")),
            release_qs: std::mem::transmute(need(c"wgpuQuerySetRelease")),
            release_compute_pass: std::mem::transmute(need(c"wgpuComputePassEncoderRelease")),
            release_bundle_enc: std::mem::transmute(need(c"wgpuRenderBundleEncoderRelease")),
            release_bundle: std::mem::transmute(need(c"wgpuRenderBundleRelease")),
        };
        if (api.create_instance as *const c_void).is_null()
            || (api.request_adapter as *const c_void).is_null()
        {
            return None;
        }
        log_android(true, "dlopen libwebgpu_dawn.so ok; wgpu* bound");
        Some(api)
    }
}

pub fn as_ptr(slot: DawnSlot) -> WgpuObj {
    slot as WgpuObj
}

pub fn from_ptr(p: WgpuObj) -> DawnSlot {
    p as DawnSlot
}

unsafe extern "C" fn on_adapter(
    status: WgpuEnum,
    adapter: WgpuObj,
    _msg: StringView,
    userdata1: *mut c_void,
    _userdata2: *mut c_void,
) {
    let out = userdata1 as *mut (WgpuEnum, WgpuObj);
    if !out.is_null() {
        (*out) = (status, adapter);
    }
}

unsafe extern "C" fn on_device(
    status: WgpuEnum,
    device: WgpuObj,
    _msg: StringView,
    userdata1: *mut c_void,
    _userdata2: *mut c_void,
) {
    let out = userdata1 as *mut (WgpuEnum, WgpuObj);
    if !out.is_null() {
        (*out) = (status, device);
    }
}

fn wait_future(
    api: &Api,
    instance: WgpuObj,
    future: Future,
    out: &mut (WgpuEnum, WgpuObj),
) -> bool {
    unsafe {
        let mut wait = FutureWaitInfo {
            future,
            completed: 0,
        };
        let _ = (api.wait_any)(instance, 1, &mut wait, 2_000_000_000);
        if out.0 == 0 {
            (api.process_events)(instance);
        }
    }
    out.0 == STATUS_SUCCESS && !out.1.is_null()
}

pub fn create_instance() -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    let feat = INSTANCE_TIMED_WAIT_ANY;
    let desc = InstanceDesc {
        next_in_chain: std::ptr::null_mut(),
        required_feature_count: 1,
        required_features: &feat,
        required_limits: std::ptr::null(),
    };
    let inst = unsafe { from_ptr((api.create_instance)(&desc)) };
    if inst != 0 {
        return inst;
    }
    let fallback = InstanceDesc {
        next_in_chain: std::ptr::null_mut(),
        required_feature_count: 0,
        required_features: std::ptr::null(),
        required_limits: std::ptr::null(),
    };
    unsafe { from_ptr((api.create_instance)(&fallback)) }
}

fn request_adapter_backend(
    api: &Api,
    instance: WgpuObj,
    backend: WgpuEnum,
    feature_level: WgpuEnum,
    power_preference: WgpuEnum,
    force_fallback: WgpuBool,
) -> DawnSlot {
    let mut out = (0u32, std::ptr::null_mut());
    let opts = RequestAdapterOptions {
        next_in_chain: std::ptr::null_mut(),
        feature_level,
        power_preference,
        force_fallback_adapter: force_fallback,
        backend_type: backend,
        compatible_surface: std::ptr::null_mut(),
    };
    let info = CallbackInfo {
        next_in_chain: std::ptr::null_mut(),
        mode: CALLBACK_WAIT_ANY,
        callback: on_adapter as *const c_void,
        userdata1: (&mut out) as *mut _ as *mut c_void,
        userdata2: std::ptr::null_mut(),
    };
    let future = unsafe { (api.request_adapter)(instance, &opts, info) };
    if wait_future(api, instance, future, &mut out) {
        log_android(
            true,
            &format!("wgpuInstanceRequestAdapter ok backend={backend}"),
        );
        from_ptr(out.1)
    } else {
        log_android(
            false,
            &format!(
                "wgpuInstanceRequestAdapter failed backend={backend} status={}",
                out.0
            ),
        );
        0
    }
}

pub fn request_adapter_vulkan(
    instance: DawnSlot,
    feature_level: WgpuEnum,
    power_preference: WgpuEnum,
    force_fallback: bool,
) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if instance == 0 {
        log_android(false, "wgpuCreateInstance returned null");
        return 0;
    }
    let fallback = if force_fallback { 1 } else { 0 };
    let vulkan = request_adapter_backend(
        api,
        as_ptr(instance),
        BACKEND_VULKAN,
        feature_level,
        power_preference,
        fallback,
    );
    if vulkan != 0 {
        return vulkan;
    }
    request_adapter_backend(
        api,
        as_ptr(instance),
        0,
        feature_level,
        power_preference,
        fallback,
    )
}

pub fn request_device(
    instance: DawnSlot,
    adapter: DawnSlot,
    required_features: &[WgpuEnum],
    label: &str,
    default_queue_label: &str,
) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if adapter == 0 {
        return 0;
    }
    let mut out = (0u32, std::ptr::null_mut());
    let desc = DeviceDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::from_str(label),
        required_feature_count: required_features.len(),
        required_features: if required_features.is_empty() {
            std::ptr::null()
        } else {
            required_features.as_ptr()
        },
        required_limits: std::ptr::null(),
        default_queue: QueueDesc {
            next_in_chain: std::ptr::null_mut(),
            label: StringView::from_str(default_queue_label),
        },
        device_lost: CallbackInfo {
            next_in_chain: std::ptr::null_mut(),
            mode: 0,
            callback: std::ptr::null(),
            userdata1: std::ptr::null_mut(),
            userdata2: std::ptr::null_mut(),
        },
        uncaptured: UncapturedInfo {
            next_in_chain: std::ptr::null_mut(),
            callback: std::ptr::null(),
            userdata1: std::ptr::null_mut(),
            userdata2: std::ptr::null_mut(),
        },
    };
    let info = CallbackInfo {
        next_in_chain: std::ptr::null_mut(),
        mode: CALLBACK_WAIT_ANY,
        callback: on_device as *const c_void,
        userdata1: (&mut out) as *mut _ as *mut c_void,
        userdata2: std::ptr::null_mut(),
    };
    let future = unsafe { (api.request_device)(as_ptr(adapter), &desc, info) };
    if wait_future(api, as_ptr(instance), future, &mut out) {
        log_android(true, "wgpuAdapterRequestDevice ok");
        from_ptr(out.1)
    } else {
        log_android(false, "wgpuAdapterRequestDevice failed");
        0
    }
}

pub fn device_queue(device: DawnSlot) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if device == 0 {
        return 0;
    }
    unsafe { from_ptr((api.device_get_queue)(as_ptr(device))) }
}

pub fn create_buffer(
    device: DawnSlot,
    size: u64,
    usage: u32,
    mapped: bool,
    label: &str,
) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if device == 0 {
        return 0;
    }
    let desc = BufferDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::from_str(label),
        usage: usage as WgpuFlags,
        size,
        mapped_at_creation: if mapped { 1 } else { 0 },
    };
    unsafe { from_ptr((api.create_buffer)(as_ptr(device), &desc)) }
}

pub fn create_shader(device: DawnSlot, code: &str, label: &str) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if device == 0 {
        return 0;
    }
    let mut wgsl = ShaderWgsl {
        chain: Chained {
            next: std::ptr::null_mut(),
            s_type: STYPE_SHADER_WGSL,
        },
        code: StringView::from_str(code),
    };
    let desc = ShaderDesc {
        next_in_chain: &mut wgsl.chain,
        label: StringView::from_str(label),
    };
    unsafe { from_ptr((api.create_shader)(as_ptr(device), &desc)) }
}

fn zero_bgl_entry() -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        next_in_chain: std::ptr::null_mut(),
        binding: 0,
        visibility: 0,
        binding_array_size: 0,
        buffer: BufferBindingLayout {
            next_in_chain: std::ptr::null_mut(),
            ty: 0,
            has_dynamic_offset: 0,
            min_binding_size: 0,
        },
        sampler: SamplerBindingLayout {
            next_in_chain: std::ptr::null_mut(),
            ty: 0,
        },
        texture: TextureBindingLayout {
            next_in_chain: std::ptr::null_mut(),
            sample_type: 0,
            view_dimension: 0,
            multisampled: 0,
        },
        storage_texture: StorageTextureBindingLayout {
            next_in_chain: std::ptr::null_mut(),
            access: 0,
            format: 0,
            view_dimension: 0,
        },
    }
}

fn proc_ok<T>(f: T) -> bool {
    !unsafe { std::mem::transmute_copy::<T, *const c_void>(&f) }.is_null()
}

/// JNI leftover pack: 0=uniform 1=storage 2=read-only-storage −1=unused.
/// Sampler pack: 0=filtering 1=non-filtering 2=comparison −1=unused.
/// Texture sample pack: 0=float 1=unfilterable 2=depth 3=sint 4=uint −1=unused.
pub fn create_bind_group_layout(
    device: DawnSlot,
    bindings: &[i32],
    visibilities: &[i32],
    buffer_types: &[i32],
    sampler_types: &[i32],
    texture_sample_types: &[i32],
) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if device == 0 {
        return 0;
    }
    let mut entries = Vec::with_capacity(bindings.len());
    for i in 0..bindings.len() {
        let mut e = zero_bgl_entry();
        e.binding = bindings[i] as u32;
        e.visibility = visibilities.get(i).copied().unwrap_or(0) as WgpuFlags;
        let packed = buffer_types.get(i).copied().unwrap_or(-1);
        e.buffer.ty = match packed {
            0 => BINDING_UNIFORM,
            1 => BINDING_STORAGE,
            2 => BINDING_RO_STORAGE,
            _ => 0,
        };
        if packed == 0 {
            e.buffer.min_binding_size = 64;
        }
        e.sampler.ty = match sampler_types.get(i).copied().unwrap_or(-1) {
            0 => SAMPLER_FILTERING,
            1 => SAMPLER_NON_FILTERING,
            2 => SAMPLER_COMPARISON,
            _ => 0,
        };
        e.texture.sample_type = match texture_sample_types.get(i).copied().unwrap_or(-1) {
            0 => TEX_SAMPLE_FLOAT,
            1 => TEX_SAMPLE_UNFILTERABLE,
            2 => TEX_SAMPLE_DEPTH,
            3 => TEX_SAMPLE_SINT,
            4 => TEX_SAMPLE_UINT,
            _ => 0,
        };
        entries.push(e);
    }
    let desc = BindGroupLayoutDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::empty(),
        entry_count: entries.len(),
        entries: entries.as_ptr(),
    };
    unsafe { from_ptr((api.create_bgl)(as_ptr(device), &desc)) }
}

pub fn create_pipeline_layout(device: DawnSlot, layouts: &[DawnSlot], label: &str) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if device == 0 {
        return 0;
    }
    let objs: Vec<WgpuObj> = layouts.iter().map(|&s| as_ptr(s)).collect();
    let desc = PipelineLayoutDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::from_str(label),
        bind_group_layout_count: objs.len(),
        bind_group_layouts: objs.as_ptr(),
        immediate_size: 0,
    };
    unsafe { from_ptr((api.create_pl)(as_ptr(device), &desc)) }
}

pub fn create_bind_group(
    device: DawnSlot,
    layout: DawnSlot,
    bindings: &[i32],
    kinds: &[i32],
    slots: &[DawnSlot],
    label: &str,
) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if device == 0 || layout == 0 {
        return 0;
    }
    let mut entries = Vec::with_capacity(bindings.len());
    for i in 0..bindings.len() {
        let kind = kinds.get(i).copied().unwrap_or(0);
        let slot = slots.get(i).copied().unwrap_or(0);
        let mut e = BindGroupEntry {
            next_in_chain: std::ptr::null_mut(),
            binding: bindings[i] as u32,
            buffer: std::ptr::null_mut(),
            offset: 0,
            size: WGPU_WHOLE_SIZE,
            sampler: std::ptr::null_mut(),
            texture_view: std::ptr::null_mut(),
        };
        match kind {
            1 => e.sampler = as_ptr(slot),
            2 => e.texture_view = as_ptr(slot),
            _ => e.buffer = as_ptr(slot),
        }
        entries.push(e);
    }
    let desc = BindGroupDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::from_str(label),
        layout: as_ptr(layout),
        entry_count: entries.len(),
        entries: entries.as_ptr(),
    };
    unsafe { from_ptr((api.create_bg)(as_ptr(device), &desc)) }
}

pub struct VertexPack<'a> {
    pub strides: &'a [i32],
    pub step_modes: &'a [i32],
    pub attr_index: &'a [i32],
    pub attr_formats: &'a [i32],
    pub attr_offsets: &'a [i32],
    pub attr_locations: &'a [i32],
}

/// Packed leftover ctor fields (blend / MSAA / depth-stencil / constants).
pub struct RenderPipelineCtor<'a> {
    pub multisample: &'a [i32],
    pub blend: &'a [i32],
    pub write_mask: &'a [i32],
    pub depth_stencil: &'a [i32],
    pub vertex_constants: &'a [(String, f64)],
    pub fragment_constants: &'a [(String, f64)],
}

impl RenderPipelineCtor<'static> {
    pub const EMPTY: Self = Self {
        multisample: &[],
        blend: &[],
        write_mask: &[],
        depth_stencil: &[],
        vertex_constants: &[],
        fragment_constants: &[],
    };
}

fn constant_entries(pairs: &[(String, f64)]) -> Vec<ConstantEntry> {
    pairs
        .iter()
        .map(|(k, v)| ConstantEntry {
            next_in_chain: std::ptr::null_mut(),
            key: StringView::from_str(k),
            value: *v,
        })
        .collect()
}

fn packed_write_mask(v: i32) -> WgpuFlags {
    if v < 0 || (v & (1 << 4)) != 0 {
        COLOR_WRITE_ALL
    } else {
        (v & 0xF) as WgpuFlags
    }
}

fn parse_blend(blend: &[i32], target: usize) -> Option<BlendState> {
    let o = target.saturating_mul(7);
    if blend.len() < o + 7 || blend[o] == 0 {
        return None;
    }
    Some(BlendState {
        color: BlendComponent {
            operation: blend[o + 1] as WgpuEnum,
            src_factor: blend[o + 2] as WgpuEnum,
            dst_factor: blend[o + 3] as WgpuEnum,
        },
        alpha: BlendComponent {
            operation: blend[o + 4] as WgpuEnum,
            src_factor: blend[o + 5] as WgpuEnum,
            dst_factor: blend[o + 6] as WgpuEnum,
        },
    })
}

fn parse_multisample(packed: &[i32]) -> MultisampleState {
    if packed.len() < 4 {
        return MultisampleState {
            next_in_chain: std::ptr::null_mut(),
            count: 1,
            mask: u32::MAX,
            alpha_to_coverage_enabled: 0,
        };
    }
    let count = if packed[0] <= 0 { 1 } else { packed[0] as u32 };
    let mask = if packed[1] != 0 {
        packed[2] as u32
    } else {
        u32::MAX
    };
    MultisampleState {
        next_in_chain: std::ptr::null_mut(),
        count,
        mask,
        alpha_to_coverage_enabled: if packed[3] == 1 { 1 } else { 0 },
    }
}

fn parse_stencil_face(packed: &[i32], off: usize) -> StencilFaceState {
    let get = |i: usize| packed.get(i).copied().unwrap_or(0);
    if get(off) == 0 {
        return StencilFaceState {
            compare: 0,
            fail_op: 0,
            depth_fail_op: 0,
            pass_op: 0,
        };
    }
    StencilFaceState {
        compare: get(off + 1) as WgpuEnum,
        fail_op: get(off + 2) as WgpuEnum,
        depth_fail_op: get(off + 3) as WgpuEnum,
        pass_op: get(off + 4) as WgpuEnum,
    }
}

fn parse_depth_stencil(packed: &[i32]) -> Option<DepthStencilState> {
    if packed.is_empty() {
        return None;
    }
    let get = |i: usize| packed.get(i).copied().unwrap_or(0);
    let pair_u32 = |off: usize, default: u32| {
        if get(off) != 0 {
            get(off + 1) as u32
        } else {
            default
        }
    };
    let pair_i32 = |off: usize| {
        if get(off) != 0 {
            get(off + 1)
        } else {
            0
        }
    };
    let pair_f32 = |off: usize| {
        if get(off) != 0 {
            f32::from_bits(get(off + 1) as u32)
        } else {
            0.0
        }
    };
    Some(DepthStencilState {
        next_in_chain: std::ptr::null_mut(),
        format: get(0) as WgpuEnum,
        depth_write_enabled: match get(1) {
            1 => 1,
            0 => 0,
            _ => 2,
        },
        depth_compare: get(2) as WgpuEnum,
        stencil_front: parse_stencil_face(packed, 3),
        stencil_back: parse_stencil_face(packed, 8),
        stencil_read_mask: pair_u32(13, u32::MAX),
        stencil_write_mask: pair_u32(15, u32::MAX),
        depth_bias: pair_i32(17),
        depth_bias_slope_scale: pair_f32(19),
        depth_bias_clamp: pair_f32(21),
    })
}

pub fn create_render_pipeline(
    device: DawnSlot,
    layout: DawnSlot,
    vs: DawnSlot,
    vs_entry: &str,
    fs: DawnSlot,
    fs_entry: &str,
    format: WgpuEnum,
    primitive: &[i32],
    pack: VertexPack<'_>,
    label: &str,
    ctor: RenderPipelineCtor<'_>,
) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if device == 0 || vs == 0 {
        return 0;
    }
    let n = pack.strides.len();
    let mut attrs: Vec<Vec<VertexAttribute>> = (0..n).map(|_| Vec::new()).collect();
    for i in 0..pack.attr_index.len() {
        let buf = pack.attr_index[i] as usize;
        if buf >= attrs.len() {
            continue;
        }
        attrs[buf].push(VertexAttribute {
            next_in_chain: std::ptr::null_mut(),
            format: pack.attr_formats.get(i).copied().unwrap_or(0) as WgpuEnum,
            offset: pack.attr_offsets.get(i).copied().unwrap_or(0) as u64,
            shader_location: pack.attr_locations.get(i).copied().unwrap_or(0) as u32,
        });
    }
    let layouts: Vec<VertexBufferLayout> = (0..n)
        .map(|i| VertexBufferLayout {
            next_in_chain: std::ptr::null_mut(),
            step_mode: pack
                .step_modes
                .get(i)
                .copied()
                .unwrap_or(STEP_VERTEX as i32) as WgpuEnum,
            array_stride: pack.strides[i] as u64,
            attribute_count: attrs[i].len(),
            attributes: attrs[i].as_ptr(),
        })
        .collect();
    let vs_constants = constant_entries(ctor.vertex_constants);
    let fs_constants = constant_entries(ctor.fragment_constants);
    let target_n = ctor.write_mask.len().max(ctor.blend.len() / 7).max(1);
    let blends: Vec<Option<BlendState>> =
        (0..target_n).map(|i| parse_blend(ctor.blend, i)).collect();
    let fmt = if format == 0 {
        FORMAT_RGBA8_UNORM
    } else {
        format
    };
    let targets: Vec<ColorTargetState> = (0..target_n)
        .map(|i| ColorTargetState {
            next_in_chain: std::ptr::null_mut(),
            format: fmt,
            blend: blends[i]
                .as_ref()
                .map(|b| b as *const BlendState)
                .unwrap_or(std::ptr::null()),
            write_mask: ctor
                .write_mask
                .get(i)
                .copied()
                .map(packed_write_mask)
                .unwrap_or(COLOR_WRITE_ALL),
        })
        .collect();
    let depth = parse_depth_stencil(ctor.depth_stencil);
    let fs_mod = if fs == 0 { vs } else { fs };
    let fragment = FragmentState {
        next_in_chain: std::ptr::null_mut(),
        module: as_ptr(fs_mod),
        entry_point: StringView::from_str(if fs_entry.is_empty() {
            "fs_main"
        } else {
            fs_entry
        }),
        constant_count: fs_constants.len(),
        constants: if fs_constants.is_empty() {
            std::ptr::null()
        } else {
            fs_constants.as_ptr()
        },
        target_count: targets.len(),
        targets: targets.as_ptr(),
    };
    let desc = RenderPipelineDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::from_str(label),
        layout: as_ptr(layout),
        vertex: VertexState {
            next_in_chain: std::ptr::null_mut(),
            module: as_ptr(vs),
            entry_point: StringView::from_str(if vs_entry.is_empty() {
                "vs_main"
            } else {
                vs_entry
            }),
            constant_count: vs_constants.len(),
            constants: if vs_constants.is_empty() {
                std::ptr::null()
            } else {
                vs_constants.as_ptr()
            },
            buffer_count: layouts.len(),
            buffers: layouts.as_ptr(),
        },
        primitive: PrimitiveState {
            next_in_chain: std::ptr::null_mut(),
            topology: primitive
                .first()
                .copied()
                .unwrap_or(TOPOLOGY_TRIANGLE_LIST as i32) as WgpuEnum,
            strip_index_format: primitive.get(1).copied().unwrap_or(0) as WgpuEnum,
            front_face: primitive.get(2).copied().unwrap_or(FRONT_CCW as i32) as WgpuEnum,
            cull_mode: primitive.get(3).copied().unwrap_or(CULL_BACK as i32) as WgpuEnum,
            unclipped_depth: 0,
        },
        depth_stencil: depth
            .as_ref()
            .map(|d| d as *const DepthStencilState)
            .unwrap_or(std::ptr::null()),
        multisample: parse_multisample(ctor.multisample),
        fragment: &fragment,
    };
    unsafe { from_ptr((api.create_rp)(as_ptr(device), &desc)) }
}

pub fn create_encoder(device: DawnSlot, label: &str) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if device == 0 {
        return 0;
    }
    let desc = CommandEncoderDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::from_str(label),
    };
    unsafe { from_ptr((api.create_encoder)(as_ptr(device), &desc)) }
}

pub fn begin_render_pass(
    encoder: DawnSlot,
    views: &[DawnSlot],
    loads: &[i32],
    stores: &[i32],
    has_clears: &[i32],
    clear_bits: &[i32],
    depth: DawnSlot,
) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if encoder == 0 {
        return 0;
    }
    let mut atts = Vec::with_capacity(views.len());
    for (i, &view) in views.iter().enumerate() {
        if view == 0 {
            continue;
        }
        let bits = |ch: usize| {
            let idx = i * 4 + ch;
            clear_bits
                .get(idx)
                .copied()
                .map(|b| f32::from_bits(b as u32) as f64)
                .unwrap_or(0.0)
        };
        let has = has_clears.get(i).copied().unwrap_or(1) != 0;
        atts.push(RenderPassColorAttachment {
            next_in_chain: std::ptr::null_mut(),
            view: as_ptr(view),
            depth_slice: WGPU_DEPTH_SLICE_UNDEFINED,
            resolve_target: std::ptr::null_mut(),
            load_op: loads
                .get(i)
                .copied()
                .filter(|&v| v > 0)
                .unwrap_or(LOAD_CLEAR as i32) as WgpuEnum,
            store_op: stores
                .get(i)
                .copied()
                .filter(|&v| v > 0)
                .unwrap_or(STORE_STORE as i32) as WgpuEnum,
            clear_value: if has {
                Color {
                    r: bits(0),
                    g: bits(1),
                    b: bits(2),
                    a: bits(3),
                }
            } else {
                Color {
                    r: 0.06,
                    g: 0.07,
                    b: 0.12,
                    a: 1.0,
                }
            },
        });
    }
    let depth_att = if depth != 0 {
        Some(DepthStencilAtt {
            next_in_chain: std::ptr::null_mut(),
            view: as_ptr(depth),
            depth_load_op: LOAD_CLEAR,
            depth_store_op: STORE_STORE,
            depth_clear: 1.0,
            depth_read_only: 0,
            stencil_load_op: 0,
            stencil_store_op: 0,
            stencil_clear: 0,
            stencil_read_only: 1,
        })
    } else {
        None
    };
    let desc = RenderPassDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::empty(),
        color_attachment_count: atts.len(),
        color_attachments: atts.as_ptr(),
        depth_stencil_attachment: depth_att
            .as_ref()
            .map(|d| d as *const DepthStencilAtt as *const c_void)
            .unwrap_or(std::ptr::null()),
        occlusion_query_set: std::ptr::null_mut(),
        timestamp_writes: std::ptr::null(),
    };
    unsafe { from_ptr((api.begin_render_pass)(as_ptr(encoder), &desc)) }
}

pub fn encoder_finish(encoder: DawnSlot, label: &str) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if encoder == 0 {
        return 0;
    }
    let desc = CommandBufferDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::from_str(label),
    };
    unsafe { from_ptr((api.encoder_finish)(as_ptr(encoder), &desc)) }
}

pub fn write_buffer(queue: DawnSlot, buffer: DawnSlot, offset: u64, bytes: &[u8]) {
    let Some(api) = api() else {
        return;
    };
    if queue == 0 || buffer == 0 || bytes.is_empty() {
        return;
    }
    unsafe {
        (api.write_buffer)(
            as_ptr(queue),
            as_ptr(buffer),
            offset,
            bytes.as_ptr(),
            bytes.len(),
        );
    }
}

pub fn queue_submit(queue: DawnSlot, commands: &[DawnSlot]) {
    let Some(api) = api() else {
        return;
    };
    if queue == 0 {
        return;
    }
    let objs: Vec<WgpuObj> = commands
        .iter()
        .filter(|s| **s != 0)
        .map(|&s| as_ptr(s))
        .collect();
    unsafe {
        (api.queue_submit)(as_ptr(queue), objs.len(), objs.as_ptr());
    }
}

pub fn create_surface(instance: DawnSlot, window: i64) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if instance == 0 || window == 0 {
        return 0;
    }
    let mut src = SurfaceAndroid {
        chain: Chained {
            next: std::ptr::null_mut(),
            s_type: STYPE_SURFACE_ANDROID,
        },
        window: window as *mut c_void,
    };
    let desc = SurfaceDesc {
        next_in_chain: &mut src.chain,
        label: StringView::empty(),
    };
    unsafe { from_ptr((api.create_surface)(as_ptr(instance), &desc)) }
}

pub fn surface_preferred_format(surface: DawnSlot, adapter: DawnSlot) -> WgpuEnum {
    let Some(api) = api() else {
        return FORMAT_RGBA8_UNORM;
    };
    if surface == 0 || adapter == 0 {
        return FORMAT_RGBA8_UNORM;
    }
    let mut caps = SurfaceCaps {
        next_in_chain: std::ptr::null_mut(),
        usages: 0,
        format_count: 0,
        formats: std::ptr::null(),
        present_mode_count: 0,
        present_modes: std::ptr::null(),
        alpha_mode_count: 0,
        alpha_modes: std::ptr::null(),
    };
    unsafe {
        let st = (api.surface_caps)(as_ptr(surface), as_ptr(adapter), &mut caps);
        let fmt = if st == STATUS_SUCCESS && caps.format_count > 0 && !caps.formats.is_null() {
            *caps.formats
        } else {
            FORMAT_RGBA8_UNORM
        };
        (api.caps_free)(caps);
        fmt
    }
}

/// Full surface capability lists (formats / present / alpha), copied out before
/// `caps_free`. Mirrors androidx `getCapabilities` so the Dawn C path can pick
/// the same `alphaModes[0]` / `formats[0]` as the smooth D24 control.
pub fn surface_caps_detail(
    surface: DawnSlot,
    adapter: DawnSlot,
) -> (Vec<WgpuEnum>, Vec<WgpuEnum>, Vec<WgpuEnum>) {
    let empty = (Vec::new(), Vec::new(), Vec::new());
    let Some(api) = api() else {
        return empty;
    };
    if surface == 0 || adapter == 0 {
        return empty;
    }
    let mut caps = SurfaceCaps {
        next_in_chain: std::ptr::null_mut(),
        usages: 0,
        format_count: 0,
        formats: std::ptr::null(),
        present_mode_count: 0,
        present_modes: std::ptr::null(),
        alpha_mode_count: 0,
        alpha_modes: std::ptr::null(),
    };
    unsafe {
        let st = (api.surface_caps)(as_ptr(surface), as_ptr(adapter), &mut caps);
        if st != STATUS_SUCCESS {
            (api.caps_free)(caps);
            return empty;
        }
        let copy = |count: usize, ptr: *const WgpuEnum| -> Vec<WgpuEnum> {
            if count > 0 && !ptr.is_null() {
                std::slice::from_raw_parts(ptr, count).to_vec()
            } else {
                Vec::new()
            }
        };
        let formats = copy(caps.format_count, caps.formats);
        let present = copy(caps.present_mode_count, caps.present_modes);
        let alpha = copy(caps.alpha_mode_count, caps.alpha_modes);
        (api.caps_free)(caps);
        (formats, present, alpha)
    }
}

pub fn surface_configure(
    surface: DawnSlot,
    device: DawnSlot,
    format: WgpuEnum,
    width: u32,
    height: u32,
    alpha_mode: WgpuEnum,
) {
    let Some(api) = api() else {
        return;
    };
    if surface == 0 || device == 0 || width == 0 || height == 0 {
        return;
    }
    let cfg = SurfaceConfig {
        next_in_chain: std::ptr::null_mut(),
        device: as_ptr(device),
        format: if format == 0 {
            FORMAT_RGBA8_UNORM
        } else {
            format
        },
        usage: USAGE_RENDER_ATTACHMENT,
        width,
        height,
        view_format_count: 0,
        view_formats: std::ptr::null(),
        alpha_mode,
        present_mode: PRESENT_FIFO,
    };
    unsafe {
        (api.surface_configure)(as_ptr(surface), &cfg);
    }
}

/// `(texture, WGPUSurfaceGetCurrentTextureStatus)`. Texture is `0` unless
/// Optimal/Suboptimal. Status `0` means the call did not run.
pub fn surface_current_texture(surface: DawnSlot) -> (DawnSlot, u32) {
    let Some(api) = api() else {
        return (0, 0);
    };
    if surface == 0 {
        return (0, 0);
    }
    let mut tex = SurfaceTexture {
        next_in_chain: std::ptr::null_mut(),
        texture: std::ptr::null_mut(),
        status: 0,
    };
    unsafe {
        (api.surface_get_current)(as_ptr(surface), &mut tex);
    }
    if tex.status != SURFACE_OK_OPTIMAL && tex.status != SURFACE_OK_SUBOPTIMAL {
        log_android(
            false,
            &format!("wgpuSurfaceGetCurrentTexture status={}", tex.status),
        );
        return (0, tex.status);
    }
    (from_ptr(tex.texture), tex.status)
}

pub fn surface_present(surface: DawnSlot) -> bool {
    let Some(api) = api() else {
        return false;
    };
    if surface == 0 {
        return false;
    }
    unsafe { (api.surface_present)(as_ptr(surface)) == STATUS_SUCCESS }
}

pub fn texture_size(texture: DawnSlot) -> (u32, u32) {
    let Some(api) = api() else {
        return (1, 1);
    };
    if texture == 0 {
        return (1, 1);
    }
    unsafe {
        (
            (api.texture_width)(as_ptr(texture)),
            (api.texture_height)(as_ptr(texture)),
        )
    }
}

pub fn create_view(
    texture: DawnSlot,
    dimension: u32,
    aspect: u32,
    format: u32,
    base_mip: u32,
    mip_count: u32,
    base_layer: u32,
    layer_count: u32,
) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if texture == 0 {
        return 0;
    }
    let desc = TextureViewDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::empty(),
        format,
        dimension,
        base_mip_level: base_mip,
        mip_level_count: if mip_count == 0 {
            WGPU_MIP_UNDEFINED
        } else {
            mip_count
        },
        base_array_layer: base_layer,
        array_layer_count: if layer_count == 0 {
            WGPU_ARRAY_UNDEFINED
        } else {
            layer_count
        },
        aspect,
        usage: 0,
    };
    unsafe { from_ptr((api.texture_create_view)(as_ptr(texture), &desc)) }
}

pub fn pass_set_pipeline(pass: DawnSlot, pipeline: DawnSlot) {
    if let Some(api) = api() {
        if pass != 0 && pipeline != 0 {
            unsafe { (api.pass_set_pipeline)(as_ptr(pass), as_ptr(pipeline)) }
        }
    }
}

pub fn pass_set_bind_group(pass: DawnSlot, index: u32, group: DawnSlot) {
    if let Some(api) = api() {
        if pass != 0 {
            unsafe {
                (api.pass_set_bind_group)(as_ptr(pass), index, as_ptr(group), 0, std::ptr::null())
            }
        }
    }
}

pub fn pass_set_vertex_buffer(pass: DawnSlot, slot: u32, buffer: DawnSlot, offset: u64, size: u64) {
    if let Some(api) = api() {
        if pass != 0 && buffer != 0 {
            let sz = if size == 0 { WGPU_WHOLE_SIZE } else { size };
            unsafe { (api.pass_set_vertex_buffer)(as_ptr(pass), slot, as_ptr(buffer), offset, sz) }
        }
    }
}

pub fn pass_draw(
    pass: DawnSlot,
    vertex_count: u32,
    instance_count: u32,
    first: u32,
    first_inst: u32,
) {
    if let Some(api) = api() {
        if pass != 0 {
            unsafe {
                (api.pass_draw)(
                    as_ptr(pass),
                    vertex_count,
                    instance_count,
                    first,
                    first_inst,
                )
            }
        }
    }
}

pub fn pass_end(pass: DawnSlot) {
    if let Some(api) = api() {
        if pass != 0 {
            unsafe { (api.pass_end)(as_ptr(pass)) }
        }
    }
}

pub fn create_texture(
    device: DawnSlot,
    width: u32,
    height: u32,
    depth: u32,
    format: u32,
    usage: u32,
    mip: u32,
    sample: u32,
    dimension: u32,
    view_formats: &[i32],
    label: &str,
) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if device == 0 || !proc_ok(api.create_texture) {
        return 0;
    }
    let views: Vec<WgpuEnum> = view_formats.iter().map(|f| *f as WgpuEnum).collect();
    let desc = TextureDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::from_str(label),
        usage: usage as WgpuFlags,
        dimension,
        size: Extent3D {
            width: width.max(1),
            height: height.max(1),
            depth: depth.max(1),
        },
        format,
        mip_level_count: mip.max(1),
        sample_count: sample.max(1),
        view_format_count: views.len(),
        view_formats: views.as_ptr(),
    };
    unsafe { from_ptr((api.create_texture)(as_ptr(device), &desc)) }
}

pub fn create_sampler(
    device: DawnSlot,
    mag_filter: u32,
    min_filter: u32,
    address_mode_u: u32,
    address_mode_v: u32,
    address_mode_w: u32,
    mipmap_filter: u32,
    compare: u32,
    lod_min: f32,
    lod_max: f32,
) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if device == 0 || !proc_ok(api.create_sampler) {
        return 0;
    }
    let desc = SamplerDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::empty(),
        address_mode_u,
        address_mode_v,
        address_mode_w,
        mag_filter,
        min_filter,
        mipmap_filter,
        lod_min_clamp: lod_min,
        lod_max_clamp: if lod_max <= 0.0 { 32.0 } else { lod_max },
        compare,
        max_anisotropy: 1,
    };
    unsafe { from_ptr((api.create_sampler)(as_ptr(device), &desc)) }
}

pub fn create_compute_pipeline(
    device: DawnSlot,
    layout: DawnSlot,
    shader: DawnSlot,
    entry: &str,
    label: &str,
    constants: &[(String, f64)],
) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if device == 0 || shader == 0 || !proc_ok(api.create_compute_pipeline) {
        return 0;
    }
    let entries = constant_entries(constants);
    let desc = ComputePipelineDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::from_str(label),
        layout: as_ptr(layout),
        compute: ComputeState {
            next_in_chain: std::ptr::null_mut(),
            module: as_ptr(shader),
            entry_point: StringView::from_str(if entry.is_empty() { "main" } else { entry }),
            constant_count: entries.len(),
            constants: if entries.is_empty() {
                std::ptr::null()
            } else {
                entries.as_ptr()
            },
        },
    };
    unsafe { from_ptr((api.create_compute_pipeline)(as_ptr(device), &desc)) }
}

pub fn create_query_set(device: DawnSlot, ty: u32, count: u32) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if device == 0 || !proc_ok(api.create_query_set) {
        return 0;
    }
    let desc = QuerySetDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::empty(),
        ty,
        count: count.max(1),
    };
    unsafe { from_ptr((api.create_query_set)(as_ptr(device), &desc)) }
}

pub fn create_render_bundle_encoder(device: DawnSlot, format: u32) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if device == 0 || !proc_ok(api.create_bundle_enc) {
        return 0;
    }
    let fmt = if format == 0 {
        FORMAT_RGBA8_UNORM
    } else {
        format
    };
    let desc = RenderBundleEncDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::empty(),
        color_format_count: 1,
        color_formats: &fmt,
        depth_stencil_format: 0,
        sample_count: 1,
        depth_read_only: 0,
        stencil_read_only: 0,
    };
    unsafe { from_ptr((api.create_bundle_enc)(as_ptr(device), &desc)) }
}

pub fn begin_compute_pass(
    encoder: DawnSlot,
    query: DawnSlot,
    begin_idx: u32,
    end_idx: u32,
) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if encoder == 0 || !proc_ok(api.begin_compute_pass) {
        return 0;
    }
    let ts = if query != 0 {
        Some(TimestampWrites {
            next_in_chain: std::ptr::null_mut(),
            query_set: as_ptr(query),
            beginning: if begin_idx == 0 {
                WGPU_QUERY_INDEX_UNDEFINED
            } else {
                begin_idx
            },
            end: if end_idx == 0 {
                WGPU_QUERY_INDEX_UNDEFINED
            } else {
                end_idx
            },
        })
    } else {
        None
    };
    let desc = ComputePassDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::empty(),
        timestamp_writes: ts
            .as_ref()
            .map(|t| t as *const TimestampWrites)
            .unwrap_or(std::ptr::null()),
    };
    unsafe { from_ptr((api.begin_compute_pass)(as_ptr(encoder), &desc)) }
}

pub fn copy_buffer_to_buffer(
    encoder: DawnSlot,
    src: DawnSlot,
    src_off: u64,
    dst: DawnSlot,
    dst_off: u64,
    size: u64,
) {
    if let Some(api) = api() {
        if encoder != 0 && src != 0 && dst != 0 && proc_ok(api.copy_b2b) {
            let sz = if size == 0 { WGPU_WHOLE_SIZE } else { size };
            unsafe {
                (api.copy_b2b)(
                    as_ptr(encoder),
                    as_ptr(src),
                    src_off,
                    as_ptr(dst),
                    dst_off,
                    sz,
                )
            }
        }
    }
}

fn texel_buf(buffer: DawnSlot, offset: u64, bpr: u32, rpi: u32) -> TexelCopyBufferInfo {
    TexelCopyBufferInfo {
        layout: TexelCopyBufferLayout {
            offset,
            bytes_per_row: if bpr == 0 {
                WGPU_COPY_STRIDE_UNDEFINED
            } else {
                bpr
            },
            rows_per_image: if rpi == 0 {
                WGPU_COPY_STRIDE_UNDEFINED
            } else {
                rpi
            },
        },
        buffer: as_ptr(buffer),
    }
}

fn texel_tex(
    texture: DawnSlot,
    mip: u32,
    x: u32,
    y: u32,
    z: u32,
    aspect: u32,
) -> TexelCopyTextureInfo {
    TexelCopyTextureInfo {
        texture: as_ptr(texture),
        mip_level: mip,
        origin: Origin3D { x, y, z },
        aspect,
    }
}

pub fn copy_buffer_to_texture(
    encoder: DawnSlot,
    buffer: DawnSlot,
    texture: DawnSlot,
    width: u32,
    height: u32,
    depth: u32,
) {
    if let Some(api) = api() {
        if encoder != 0 && buffer != 0 && texture != 0 && proc_ok(api.copy_b2t) {
            let src = texel_buf(buffer, 0, 0, 0);
            let dst = texel_tex(texture, 0, 0, 0, 0, 0);
            let size = Extent3D {
                width: width.max(1),
                height: height.max(1),
                depth: depth.max(1),
            };
            unsafe { (api.copy_b2t)(as_ptr(encoder), &src, &dst, &size) }
        }
    }
}

pub fn copy_texture_to_buffer(
    encoder: DawnSlot,
    texture: DawnSlot,
    buffer: DawnSlot,
    width: u32,
    height: u32,
    depth: u32,
) {
    if let Some(api) = api() {
        if encoder != 0 && buffer != 0 && texture != 0 && proc_ok(api.copy_t2b) {
            let src = texel_tex(texture, 0, 0, 0, 0, 0);
            let dst = texel_buf(buffer, 0, 0, 0);
            let size = Extent3D {
                width: width.max(1),
                height: height.max(1),
                depth: depth.max(1),
            };
            unsafe { (api.copy_t2b)(as_ptr(encoder), &src, &dst, &size) }
        }
    }
}

pub fn copy_texture_to_texture(
    encoder: DawnSlot,
    src: DawnSlot,
    dst: DawnSlot,
    width: u32,
    height: u32,
    depth: u32,
) {
    if let Some(api) = api() {
        if encoder != 0 && src != 0 && dst != 0 && proc_ok(api.copy_t2t) {
            let s = texel_tex(src, 0, 0, 0, 0, 0);
            let d = texel_tex(dst, 0, 0, 0, 0, 0);
            let size = Extent3D {
                width: width.max(1),
                height: height.max(1),
                depth: depth.max(1),
            };
            unsafe { (api.copy_t2t)(as_ptr(encoder), &s, &d, &size) }
        }
    }
}

pub fn clear_buffer(encoder: DawnSlot, buffer: DawnSlot, offset: u64, size: u64) {
    if let Some(api) = api() {
        if encoder != 0 && buffer != 0 && proc_ok(api.clear_buffer) {
            let sz = if size == 0 { WGPU_WHOLE_SIZE } else { size };
            unsafe { (api.clear_buffer)(as_ptr(encoder), as_ptr(buffer), offset, sz) }
        }
    }
}

pub fn resolve_query_set(
    encoder: DawnSlot,
    query: DawnSlot,
    first: u32,
    count: u32,
    dest: DawnSlot,
    dest_off: u64,
) {
    if let Some(api) = api() {
        if encoder != 0 && query != 0 && dest != 0 && proc_ok(api.resolve_query) {
            unsafe {
                (api.resolve_query)(
                    as_ptr(encoder),
                    as_ptr(query),
                    first,
                    count.max(1),
                    as_ptr(dest),
                    dest_off,
                )
            }
        }
    }
}

pub fn write_texture(
    queue: DawnSlot,
    texture: DawnSlot,
    bytes: &[u8],
    bytes_per_row: u32,
    width: u32,
    height: u32,
    depth: u32,
) {
    if let Some(api) = api() {
        if queue != 0 && texture != 0 && proc_ok(api.write_texture) {
            let dst = texel_tex(texture, 0, 0, 0, 0, 0);
            let layout = TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: if bytes_per_row == 0 {
                    WGPU_COPY_STRIDE_UNDEFINED
                } else {
                    bytes_per_row
                },
                rows_per_image: WGPU_COPY_STRIDE_UNDEFINED,
            };
            let size = Extent3D {
                width: width.max(1),
                height: height.max(1),
                depth: depth.max(1),
            };
            unsafe {
                (api.write_texture)(
                    as_ptr(queue),
                    &dst,
                    bytes.as_ptr(),
                    bytes.len(),
                    &layout,
                    &size,
                )
            }
        }
    }
}

unsafe extern "C" fn on_map(
    status: WgpuEnum,
    _msg: StringView,
    userdata1: *mut c_void,
    _userdata2: *mut c_void,
) {
    let out = userdata1 as *mut WgpuEnum;
    if !out.is_null() {
        *out = status;
    }
}

pub fn buffer_map_async(
    instance: DawnSlot,
    buffer: DawnSlot,
    mode: u32,
    offset: u64,
    size: u64,
) -> bool {
    let Some(api) = api() else {
        return false;
    };
    if instance == 0 || buffer == 0 || !proc_ok(api.buffer_map) {
        return false;
    }
    let mut status = 0u32;
    let info = CallbackInfo {
        next_in_chain: std::ptr::null_mut(),
        mode: CALLBACK_WAIT_ANY,
        callback: on_map as *const c_void,
        userdata1: (&mut status) as *mut _ as *mut c_void,
        userdata2: std::ptr::null_mut(),
    };
    let sz = if size == 0 { usize::MAX } else { size as usize };
    let future =
        unsafe { (api.buffer_map)(as_ptr(buffer), mode as WgpuFlags, offset as usize, sz, info) };
    let mut wait = FutureWaitInfo {
        future,
        completed: 0,
    };
    unsafe {
        let _ = (api.wait_any)(as_ptr(instance), 1, &mut wait, 2_000_000_000);
        if status == 0 {
            (api.process_events)(as_ptr(instance));
        }
    }
    status == STATUS_SUCCESS
}

pub fn buffer_unmap(buffer: DawnSlot) {
    if let Some(api) = api() {
        if buffer != 0 && proc_ok(api.buffer_unmap) {
            unsafe { (api.buffer_unmap)(as_ptr(buffer)) }
        }
    }
}

pub fn buffer_mapped_range(buffer: DawnSlot, offset: u64, size: u64) -> Vec<u8> {
    let Some(api) = api() else {
        return Vec::new();
    };
    if buffer == 0 || !proc_ok(api.buffer_mapped_range) {
        return Vec::new();
    }
    let sz = if size == 0 { 0 } else { size as usize };
    unsafe {
        let p = (api.buffer_mapped_range)(as_ptr(buffer), offset as usize, sz);
        if p.is_null() || sz == 0 {
            Vec::new()
        } else {
            std::slice::from_raw_parts(p, sz).to_vec()
        }
    }
}

pub fn work_done(instance: DawnSlot, queue: DawnSlot) {
    let Some(api) = api() else {
        return;
    };
    if instance == 0 || queue == 0 || !proc_ok(api.work_done) {
        return;
    }
    let mut status = 0u32;
    let info = CallbackInfo {
        next_in_chain: std::ptr::null_mut(),
        mode: CALLBACK_WAIT_ANY,
        callback: on_map as *const c_void,
        userdata1: (&mut status) as *mut _ as *mut c_void,
        userdata2: std::ptr::null_mut(),
    };
    let future = unsafe { (api.work_done)(as_ptr(queue), info) };
    let mut wait = FutureWaitInfo {
        future,
        completed: 0,
    };
    unsafe {
        let _ = (api.wait_any)(as_ptr(instance), 1, &mut wait, 2_000_000_000);
        if status == 0 {
            (api.process_events)(as_ptr(instance));
        }
    }
}

pub fn destroy(kind: ResourceKind, slot: DawnSlot) {
    let Some(api) = api() else {
        return;
    };
    if slot == 0 {
        return;
    }
    let p = as_ptr(slot);
    unsafe {
        match kind {
            ResourceKind::Buffer if proc_ok(api.buffer_destroy) => (api.buffer_destroy)(p),
            ResourceKind::Texture if proc_ok(api.texture_destroy) => (api.texture_destroy)(p),
            ResourceKind::QuerySet if proc_ok(api.query_destroy) => (api.query_destroy)(p),
            ResourceKind::Device if proc_ok(api.device_destroy) => (api.device_destroy)(p),
            _ => {}
        }
    }
}

pub fn adapter_has_feature(adapter: DawnSlot, feature: u32) -> bool {
    let Some(api) = api() else {
        return false;
    };
    if adapter == 0 || !proc_ok(api.adapter_has_feature) {
        return false;
    }
    unsafe { (api.adapter_has_feature)(as_ptr(adapter), feature) != 0 }
}

pub fn pass_set_viewport(pass: DawnSlot, x: f32, y: f32, w: f32, h: f32, min_d: f32, max_d: f32) {
    if let Some(api) = api() {
        if pass != 0 && proc_ok(api.pass_set_viewport) {
            unsafe { (api.pass_set_viewport)(as_ptr(pass), x, y, w, h, min_d, max_d) }
        }
    }
}

pub fn pass_set_scissor(pass: DawnSlot, x: u32, y: u32, w: u32, h: u32) {
    if let Some(api) = api() {
        if pass != 0 && proc_ok(api.pass_set_scissor) {
            unsafe { (api.pass_set_scissor)(as_ptr(pass), x, y, w, h) }
        }
    }
}

pub fn pass_set_blend_constant(pass: DawnSlot, r: f64, g: f64, b: f64, a: f64) {
    if let Some(api) = api() {
        if pass != 0 && proc_ok(api.pass_set_blend) {
            let c = Color { r, g, b, a };
            unsafe { (api.pass_set_blend)(as_ptr(pass), &c) }
        }
    }
}

pub fn pass_set_stencil_reference(pass: DawnSlot, reference: u32) {
    if let Some(api) = api() {
        if pass != 0 && proc_ok(api.pass_set_stencil) {
            unsafe { (api.pass_set_stencil)(as_ptr(pass), reference) }
        }
    }
}

pub fn pass_set_index_buffer(
    pass: DawnSlot,
    buffer: DawnSlot,
    format: u32,
    offset: u64,
    size: u64,
) {
    if let Some(api) = api() {
        if pass != 0 && buffer != 0 && proc_ok(api.pass_set_index) {
            let sz = if size == 0 { WGPU_WHOLE_SIZE } else { size };
            unsafe { (api.pass_set_index)(as_ptr(pass), as_ptr(buffer), format, offset, sz) }
        }
    }
}

pub fn pass_draw_indexed(
    pass: DawnSlot,
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
) {
    if let Some(api) = api() {
        if pass != 0 && proc_ok(api.pass_draw_indexed) {
            unsafe {
                (api.pass_draw_indexed)(
                    as_ptr(pass),
                    index_count,
                    instance_count,
                    first_index,
                    base_vertex,
                    first_instance,
                )
            }
        }
    }
}

pub fn pass_draw_indirect(pass: DawnSlot, buffer: DawnSlot, offset: u64) {
    if let Some(api) = api() {
        if pass != 0 && buffer != 0 && proc_ok(api.pass_draw_indirect) {
            unsafe { (api.pass_draw_indirect)(as_ptr(pass), as_ptr(buffer), offset) }
        }
    }
}

pub fn pass_draw_indexed_indirect(pass: DawnSlot, buffer: DawnSlot, offset: u64) {
    if let Some(api) = api() {
        if pass != 0 && buffer != 0 && proc_ok(api.pass_draw_indexed_indirect) {
            unsafe { (api.pass_draw_indexed_indirect)(as_ptr(pass), as_ptr(buffer), offset) }
        }
    }
}

pub fn pass_begin_occlusion_query(pass: DawnSlot, query_index: u32) {
    if let Some(api) = api() {
        if pass != 0 && proc_ok(api.pass_occlusion_begin) {
            unsafe { (api.pass_occlusion_begin)(as_ptr(pass), query_index) }
        }
    }
}

pub fn pass_end_occlusion_query(pass: DawnSlot) {
    if let Some(api) = api() {
        if pass != 0 && proc_ok(api.pass_occlusion_end) {
            unsafe { (api.pass_occlusion_end)(as_ptr(pass)) }
        }
    }
}

pub fn pass_execute_bundles(pass: DawnSlot, bundles: &[DawnSlot]) {
    if let Some(api) = api() {
        if pass != 0 && proc_ok(api.pass_execute_bundles) && !bundles.is_empty() {
            let objs: Vec<WgpuObj> = bundles.iter().map(|&s| as_ptr(s)).collect();
            unsafe { (api.pass_execute_bundles)(as_ptr(pass), objs.len(), objs.as_ptr()) }
        }
    }
}

pub fn compute_set_pipeline(pass: DawnSlot, pipeline: DawnSlot) {
    if let Some(api) = api() {
        if pass != 0 && pipeline != 0 && proc_ok(api.compute_set_pipeline) {
            unsafe { (api.compute_set_pipeline)(as_ptr(pass), as_ptr(pipeline)) }
        }
    }
}

pub fn compute_set_bind_group(pass: DawnSlot, index: u32, group: DawnSlot) {
    if let Some(api) = api() {
        if pass != 0 && proc_ok(api.compute_set_bind_group) {
            unsafe {
                (api.compute_set_bind_group)(
                    as_ptr(pass),
                    index,
                    as_ptr(group),
                    0,
                    std::ptr::null(),
                )
            }
        }
    }
}

pub fn compute_dispatch(pass: DawnSlot, x: u32, y: u32, z: u32) {
    if let Some(api) = api() {
        if pass != 0 && proc_ok(api.compute_dispatch) {
            unsafe { (api.compute_dispatch)(as_ptr(pass), x.max(1), y.max(1), z.max(1)) }
        }
    }
}

pub fn compute_dispatch_indirect(pass: DawnSlot, buffer: DawnSlot, offset: u64) {
    if let Some(api) = api() {
        if pass != 0 && buffer != 0 && proc_ok(api.compute_dispatch_indirect) {
            unsafe { (api.compute_dispatch_indirect)(as_ptr(pass), as_ptr(buffer), offset) }
        }
    }
}

pub fn compute_end(pass: DawnSlot) {
    if let Some(api) = api() {
        if pass != 0 && proc_ok(api.compute_end) {
            unsafe { (api.compute_end)(as_ptr(pass)) }
        }
    }
}

pub fn bundle_finish(encoder: DawnSlot) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if encoder == 0 || !proc_ok(api.bundle_finish) {
        return 0;
    }
    let desc = RenderBundleDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::empty(),
    };
    unsafe { from_ptr((api.bundle_finish)(as_ptr(encoder), &desc)) }
}

pub fn bundle_set_pipeline(enc: DawnSlot, pipeline: DawnSlot) {
    if let Some(api) = api() {
        if enc != 0 && pipeline != 0 && proc_ok(api.bundle_set_pipeline) {
            unsafe { (api.bundle_set_pipeline)(as_ptr(enc), as_ptr(pipeline)) }
        }
    }
}

pub fn bundle_set_bind_group(enc: DawnSlot, index: u32, group: DawnSlot) {
    if let Some(api) = api() {
        if enc != 0 && proc_ok(api.bundle_set_bind_group) {
            unsafe {
                (api.bundle_set_bind_group)(as_ptr(enc), index, as_ptr(group), 0, std::ptr::null())
            }
        }
    }
}

pub fn bundle_set_vertex_buffer(
    enc: DawnSlot,
    slot: u32,
    buffer: DawnSlot,
    offset: u64,
    size: u64,
) {
    if let Some(api) = api() {
        if enc != 0 && buffer != 0 && proc_ok(api.bundle_set_vb) {
            let sz = if size == 0 { WGPU_WHOLE_SIZE } else { size };
            unsafe { (api.bundle_set_vb)(as_ptr(enc), slot, as_ptr(buffer), offset, sz) }
        }
    }
}

pub fn bundle_set_index_buffer(
    enc: DawnSlot,
    buffer: DawnSlot,
    format: u32,
    offset: u64,
    size: u64,
) {
    if let Some(api) = api() {
        if enc != 0 && buffer != 0 && proc_ok(api.bundle_set_index) {
            let sz = if size == 0 { WGPU_WHOLE_SIZE } else { size };
            unsafe { (api.bundle_set_index)(as_ptr(enc), as_ptr(buffer), format, offset, sz) }
        }
    }
}

pub fn bundle_draw(
    enc: DawnSlot,
    vertex_count: u32,
    instance_count: u32,
    first: u32,
    first_inst: u32,
) {
    if let Some(api) = api() {
        if enc != 0 && proc_ok(api.bundle_draw) {
            unsafe {
                (api.bundle_draw)(as_ptr(enc), vertex_count, instance_count, first, first_inst)
            }
        }
    }
}

pub fn bundle_draw_indexed(
    enc: DawnSlot,
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
) {
    if let Some(api) = api() {
        if enc != 0 && proc_ok(api.bundle_draw_indexed) {
            unsafe {
                (api.bundle_draw_indexed)(
                    as_ptr(enc),
                    index_count,
                    instance_count,
                    first_index,
                    base_vertex,
                    first_instance,
                )
            }
        }
    }
}

pub fn bundle_draw_indirect(enc: DawnSlot, buffer: DawnSlot, offset: u64) {
    if let Some(api) = api() {
        if enc != 0 && buffer != 0 && proc_ok(api.bundle_draw_indirect) {
            unsafe { (api.bundle_draw_indirect)(as_ptr(enc), as_ptr(buffer), offset) }
        }
    }
}

pub fn pipeline_bind_group_layout(pipeline: DawnSlot, compute: bool, index: u32) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if pipeline == 0 {
        return 0;
    }
    let f = if compute {
        api.cp_get_bgl
    } else {
        api.rp_get_bgl
    };
    if !proc_ok(f) {
        return 0;
    }
    unsafe { from_ptr(f(as_ptr(pipeline), index)) }
}

pub fn push_error_scope(device: DawnSlot, filter: u32) {
    if let Some(api) = api() {
        if device != 0 && proc_ok(api.push_error) {
            unsafe { (api.push_error)(as_ptr(device), filter) }
        }
    }
}

pub fn pop_error_scope(instance: DawnSlot, device: DawnSlot) {
    let Some(api) = api() else {
        return;
    };
    if instance == 0 || device == 0 || !proc_ok(api.pop_error) {
        return;
    }
    let mut status = 0u32;
    let info = CallbackInfo {
        next_in_chain: std::ptr::null_mut(),
        mode: CALLBACK_WAIT_ANY,
        callback: on_map as *const c_void,
        userdata1: (&mut status) as *mut _ as *mut c_void,
        userdata2: std::ptr::null_mut(),
    };
    let future = unsafe { (api.pop_error)(as_ptr(device), info) };
    let mut wait = FutureWaitInfo {
        future,
        completed: 0,
    };
    unsafe {
        let _ = (api.wait_any)(as_ptr(instance), 1, &mut wait, 2_000_000_000);
    }
}

pub fn release(kind: ResourceKind, slot: DawnSlot) {
    if slot == 0 {
        return;
    }
    let Some(api) = api() else {
        return;
    };
    let p = as_ptr(slot);
    unsafe {
        match kind {
            ResourceKind::Adapter => (api.release_adapter)(p),
            ResourceKind::Device => (api.release_device)(p),
            ResourceKind::Queue => (api.release_queue)(p),
            ResourceKind::Buffer => (api.release_buffer)(p),
            ResourceKind::ShaderModule => (api.release_shader)(p),
            ResourceKind::BindGroupLayout => (api.release_bgl)(p),
            ResourceKind::PipelineLayout => (api.release_pl)(p),
            ResourceKind::BindGroup => (api.release_bg)(p),
            ResourceKind::RenderPipeline => (api.release_rp)(p),
            ResourceKind::CommandEncoder => (api.release_encoder)(p),
            ResourceKind::RenderPassEncoder => (api.release_pass)(p),
            ResourceKind::CommandBuffer => (api.release_cmd)(p),
            ResourceKind::Surface => (api.release_surface)(p),
            ResourceKind::Texture => (api.release_texture)(p),
            ResourceKind::TextureView => (api.release_view)(p),
            ResourceKind::Sampler if proc_ok(api.release_sampler) => (api.release_sampler)(p),
            ResourceKind::ComputePipeline if proc_ok(api.release_cp) => (api.release_cp)(p),
            ResourceKind::QuerySet if proc_ok(api.release_qs) => (api.release_qs)(p),
            ResourceKind::ComputePassEncoder if proc_ok(api.release_compute_pass) => {
                (api.release_compute_pass)(p)
            }
            ResourceKind::RenderBundleEncoder if proc_ok(api.release_bundle_enc) => {
                (api.release_bundle_enc)(p)
            }
            ResourceKind::RenderBundle if proc_ok(api.release_bundle) => (api.release_bundle)(p),
            _ => {}
        }
    }
}

pub fn release_instance(slot: DawnSlot) {
    if let Some(api) = api() {
        if slot != 0 {
            unsafe { (api.release_instance)(as_ptr(slot)) }
        }
    }
}

pub fn process_events(instance: DawnSlot) {
    if let Some(api) = api() {
        if instance != 0 {
            unsafe { (api.process_events)(as_ptr(instance)) }
        }
    }
}
