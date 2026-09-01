//! Dawn C `webgpu.h` loader + cube-path wrappers.
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
const ALPHA_AUTO: WgpuEnum = 0x0000_0000;
const FORMAT_RGBA8_UNORM: WgpuEnum = 0x0000_0016;
const USAGE_RENDER_ATTACHMENT: WgpuFlags = 1 << 4;
const COLOR_WRITE_ALL: WgpuFlags = 0xF;
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
    constants: *const c_void,
    buffer_count: usize,
    buffers: *const VertexBufferLayout,
}

#[repr(C)]
struct ColorTargetState {
    next_in_chain: *mut Chained,
    format: WgpuEnum,
    blend: *const c_void,
    write_mask: WgpuFlags,
}

#[repr(C)]
struct FragmentState {
    next_in_chain: *mut Chained,
    module: WgpuObj,
    entry_point: StringView,
    constant_count: usize,
    constants: *const c_void,
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
    depth_stencil: *const c_void,
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
            pass_set_vertex_buffer: std::mem::transmute(need(c"wgpuRenderPassEncoderSetVertexBuffer")),
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

fn wait_future(api: &Api, instance: WgpuObj, future: Future, out: &mut (WgpuEnum, WgpuObj)) -> bool {
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

fn request_adapter_backend(api: &Api, instance: WgpuObj, backend: WgpuEnum) -> DawnSlot {
    let mut out = (0u32, std::ptr::null_mut());
    let opts = RequestAdapterOptions {
        next_in_chain: std::ptr::null_mut(),
        feature_level: 0,
        power_preference: 0,
        force_fallback_adapter: 0,
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

pub fn request_adapter_vulkan(instance: DawnSlot) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if instance == 0 {
        log_android(false, "wgpuCreateInstance returned null");
        return 0;
    }
    let vulkan = request_adapter_backend(api, as_ptr(instance), BACKEND_VULKAN);
    if vulkan != 0 {
        return vulkan;
    }
    request_adapter_backend(api, as_ptr(instance), 0)
}

pub fn request_device(instance: DawnSlot, adapter: DawnSlot) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if adapter == 0 {
        return 0;
    }
    let mut out = (0u32, std::ptr::null_mut());
    let desc = DeviceDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::empty(),
        required_feature_count: 0,
        required_features: std::ptr::null(),
        required_limits: std::ptr::null(),
        default_queue: QueueDesc {
            next_in_chain: std::ptr::null_mut(),
            label: StringView::empty(),
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

pub fn create_buffer(device: DawnSlot, size: u64, usage: u32, mapped: bool, label: &str) -> DawnSlot {
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

/// JNI leftover pack: 0=uniform 1=storage 2=read-only-storage −1=unused.
pub fn create_bind_group_layout(
    device: DawnSlot,
    bindings: &[i32],
    visibilities: &[i32],
    buffer_types: &[i32],
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
            step_mode: pack.step_modes.get(i).copied().unwrap_or(STEP_VERTEX as i32) as WgpuEnum,
            array_stride: pack.strides[i] as u64,
            attribute_count: attrs[i].len(),
            attributes: attrs[i].as_ptr(),
        })
        .collect();
    let target = ColorTargetState {
        next_in_chain: std::ptr::null_mut(),
        format: if format == 0 { FORMAT_RGBA8_UNORM } else { format },
        blend: std::ptr::null(),
        write_mask: COLOR_WRITE_ALL,
    };
    let fs_mod = if fs == 0 { vs } else { fs };
    let fragment = FragmentState {
        next_in_chain: std::ptr::null_mut(),
        module: as_ptr(fs_mod),
        entry_point: StringView::from_str(if fs_entry.is_empty() {
            "fs_main"
        } else {
            fs_entry
        }),
        constant_count: 0,
        constants: std::ptr::null(),
        target_count: 1,
        targets: &target,
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
            constant_count: 0,
            constants: std::ptr::null(),
            buffer_count: layouts.len(),
            buffers: layouts.as_ptr(),
        },
        primitive: PrimitiveState {
            next_in_chain: std::ptr::null_mut(),
            topology: primitive.first().copied().unwrap_or(TOPOLOGY_TRIANGLE_LIST as i32) as WgpuEnum,
            strip_index_format: primitive.get(1).copied().unwrap_or(0) as WgpuEnum,
            front_face: primitive.get(2).copied().unwrap_or(FRONT_CCW as i32) as WgpuEnum,
            cull_mode: primitive.get(3).copied().unwrap_or(CULL_BACK as i32) as WgpuEnum,
            unclipped_depth: 0,
        },
        depth_stencil: std::ptr::null(),
        multisample: MultisampleState {
            next_in_chain: std::ptr::null_mut(),
            count: 1,
            mask: u32::MAX,
            alpha_to_coverage_enabled: 0,
        },
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
            load_op: loads.get(i).copied().filter(|&v| v > 0).unwrap_or(LOAD_CLEAR as i32) as WgpuEnum,
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
    let desc = RenderPassDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::empty(),
        color_attachment_count: atts.len(),
        color_attachments: atts.as_ptr(),
        depth_stencil_attachment: std::ptr::null(),
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
        (api.write_buffer)(as_ptr(queue), as_ptr(buffer), offset, bytes.as_ptr(), bytes.len());
    }
}

pub fn queue_submit(queue: DawnSlot, commands: &[DawnSlot]) {
    let Some(api) = api() else {
        return;
    };
    if queue == 0 {
        return;
    }
    let objs: Vec<WgpuObj> = commands.iter().filter(|s| **s != 0).map(|&s| as_ptr(s)).collect();
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

pub fn surface_configure(
    surface: DawnSlot,
    device: DawnSlot,
    format: WgpuEnum,
    width: u32,
    height: u32,
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
        format: if format == 0 { FORMAT_RGBA8_UNORM } else { format },
        usage: USAGE_RENDER_ATTACHMENT,
        width,
        height,
        view_format_count: 0,
        view_formats: std::ptr::null(),
        alpha_mode: ALPHA_AUTO,
        present_mode: PRESENT_FIFO,
    };
    unsafe {
        (api.surface_configure)(as_ptr(surface), &cfg);
    }
}

pub fn surface_current_texture(surface: DawnSlot) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if surface == 0 {
        return 0;
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
        log_android(false, &format!("wgpuSurfaceGetCurrentTexture status={}", tex.status));
        return 0;
    }
    from_ptr(tex.texture)
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
    unsafe { ((api.texture_width)(as_ptr(texture)), (api.texture_height)(as_ptr(texture))) }
}

pub fn create_view(texture: DawnSlot) -> DawnSlot {
    let Some(api) = api() else {
        return 0;
    };
    if texture == 0 {
        return 0;
    }
    let desc = TextureViewDesc {
        next_in_chain: std::ptr::null_mut(),
        label: StringView::empty(),
        format: 0,
        dimension: 0,
        base_mip_level: 0,
        mip_level_count: WGPU_MIP_UNDEFINED,
        base_array_layer: 0,
        array_layer_count: WGPU_ARRAY_UNDEFINED,
        aspect: 0,
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

pub fn pass_draw(pass: DawnSlot, vertex_count: u32, instance_count: u32, first: u32, first_inst: u32) {
    if let Some(api) = api() {
        if pass != 0 {
            unsafe { (api.pass_draw)(as_ptr(pass), vertex_count, instance_count, first, first_inst) }
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
