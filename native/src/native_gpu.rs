//! In-process Dawn C consume: [`NativeGpu`] trait + handle table.
//!
//! Kinds match Kotlin `ResourceKind` in
//! `host-dawn/.../experimental/host/Handles.kt`. This module must not import
//! `jni` — table insert/drop is the ND-HOST smoke that the GPU hot path does
//! not bounce through ART. Dawn C `u64` slots stay 0 until a consume lane
//! dlopens `libwebgpu_dawn.so`. Consume methods land in ND-BOOT+; until then
//! the table is exercised from `#[cfg(test)]` only (cdylib has no rlib).
#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::OnceLock;

use crate::dawn_c;

/// Dawn C object pointer/id. `0` until a later lane binds `webgpu.h`.
pub type DawnSlot = u64;

/// Opaque WASI-style resource handle (`u32`). Handle `0` is reserved as null.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GpuHandle(u32);

impl GpuHandle {
    pub const NULL: u32 = 0;

    pub fn from_raw(raw: u32) -> Result<Self, NativeGpuError> {
        if raw == Self::NULL {
            return Err(NativeGpuError::InvalidHandle {
                handle: raw,
                message: "handle 0 is reserved as null",
            });
        }
        Ok(Self(raw))
    }

    pub fn raw(self) -> u32 {
        self.0
    }
}

/// GPU resource kinds. Order matches `DawnWasiWebGpuHost` / Kotlin `ResourceKind`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ResourceKind {
    Adapter,
    Device,
    Buffer,
    ShaderModule,
    BindGroupLayout,
    BindGroup,
    PipelineLayout,
    ComputePipeline,
    CommandEncoder,
    ComputePassEncoder,
    CommandBuffer,
    Queue,
    Surface,
    CanvasContext,
    Texture,
    TextureView,
    Sampler,
    RenderPipeline,
    RenderPassEncoder,
    QuerySet,
    RenderBundleEncoder,
    RenderBundle,
}

/// Live table row: kind + optional Dawn C slot (0 = table-backed placeholder).
#[derive(Clone, Copy, Debug)]
pub struct HandleEntry {
    pub kind: ResourceKind,
    pub dawn: DawnSlot,
}

/// In-memory handle table (Kotlin `HandleTable` equivalent). Single-threaded
/// host calls (same model as P0 `docs/mapping/threading.md`).
#[derive(Debug)]
pub struct HandleTable {
    next_id: u32,
    entries: HashMap<u32, HandleEntry>,
}

impl Default for HandleTable {
    fn default() -> Self {
        Self::new()
    }
}

impl HandleTable {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, kind: ResourceKind, dawn: DawnSlot) -> GpuHandle {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        if self.next_id == GpuHandle::NULL {
            self.next_id = 1;
        }
        self.entries.insert(id, HandleEntry { kind, dawn });
        GpuHandle(id)
    }

    pub fn contains(&self, handle: GpuHandle) -> bool {
        self.entries.contains_key(&handle.0)
    }

    pub fn get(
        &self,
        handle: GpuHandle,
        kind: ResourceKind,
    ) -> Result<&HandleEntry, NativeGpuError> {
        let entry = self
            .entries
            .get(&handle.0)
            .ok_or(NativeGpuError::InvalidHandle {
                handle: handle.0,
                message: "unknown handle",
            })?;
        if entry.kind != kind {
            return Err(NativeGpuError::KindMismatch {
                handle: handle.0,
                expected: kind,
                found: entry.kind,
            });
        }
        Ok(entry)
    }

    pub fn drop_handle(&mut self, handle: GpuHandle) -> Result<HandleEntry, NativeGpuError> {
        self.try_drop(handle).ok_or(NativeGpuError::InvalidHandle {
            handle: handle.0,
            message: "already dropped or unknown",
        })
    }

    pub fn try_drop(&mut self, handle: GpuHandle) -> Option<HandleEntry> {
        self.entries.remove(&handle.0)
    }

    pub fn handles_of_kind(&self, kind: ResourceKind) -> Vec<GpuHandle> {
        let mut out: Vec<GpuHandle> = self
            .entries
            .iter()
            .filter_map(|(&id, entry)| {
                if entry.kind == kind {
                    Some(GpuHandle(id))
                } else {
                    None
                }
            })
            .collect();
        out.sort_by_key(|h| h.0);
        out
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn dawn_of(&self, handle: GpuHandle) -> DawnSlot {
        self.entries.get(&handle.0).map(|e| e.dawn).unwrap_or(0)
    }

    pub fn set_dawn(&mut self, handle: GpuHandle, dawn: DawnSlot) {
        if let Some(entry) = self.entries.get_mut(&handle.0) {
            entry.dawn = dawn;
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeGpuError {
    InvalidHandle {
        handle: u32,
        message: &'static str,
    },
    KindMismatch {
        handle: u32,
        expected: ResourceKind,
        found: ResourceKind,
    },
    AdapterUnavailable,
}

impl fmt::Display for NativeGpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NativeGpuError::InvalidHandle { handle, message } => {
                write!(f, "invalid GPU handle {handle}: {message}")
            }
            NativeGpuError::KindMismatch {
                handle,
                expected,
                found,
            } => write!(
                f,
                "invalid GPU handle {handle}: expected {expected:?} but found {found:?}"
            ),
            NativeGpuError::AdapterUnavailable => {
                write!(f, "NativeGpu request-adapter returned none")
            }
        }
    }
}

impl std::error::Error for NativeGpuError {}

/// In-process Dawn C consume (product path after ND-DEFAULT).
pub trait NativeGpu: Send {
    fn insert(&mut self, kind: ResourceKind, dawn: DawnSlot) -> GpuHandle;
    fn contains(&self, handle: GpuHandle) -> bool;
    fn get(&self, handle: GpuHandle, kind: ResourceKind) -> Result<&HandleEntry, NativeGpuError>;
    fn drop_handle(&mut self, handle: GpuHandle) -> Result<HandleEntry, NativeGpuError>;
    fn try_drop(&mut self, handle: GpuHandle) -> Option<HandleEntry>;
    fn handles_of_kind(&self, kind: ResourceKind) -> Vec<GpuHandle>;
    fn size(&self) -> usize;
    fn clear(&mut self);

    fn request_adapter(&mut self, options: &NativeRequestAdapterOptions<'_>) -> Option<GpuHandle>;
    fn request_device(
        &mut self,
        adapter: GpuHandle,
        desc: &NativeRequestDeviceDescriptor<'_>,
    ) -> Result<GpuHandle, NativeGpuError>;
    fn device_queue(&mut self, device: GpuHandle) -> Result<GpuHandle, NativeGpuError>;
    fn adapter_info(&self, adapter: GpuHandle) -> Result<NativeAdapterInfo, NativeGpuError>;
    fn adapter_has_feature(&self, adapter: GpuHandle, name: &str) -> Result<bool, NativeGpuError>;
}

/// Packed `[method]gpu.request-adapter` options (cm.rs lowering).
#[derive(Clone, Debug, Default)]
pub struct NativeRequestAdapterOptions<'a> {
    pub feature_level: &'a str,
    /// 0 = none, 1 = low-power, 2 = high-performance.
    pub power_preference: i32,
    pub force_fallback_adapter: bool,
    pub xr_compatible: Option<bool>,
}

/// Packed `[method]gpu-adapter.request-device` descriptor (cm.rs lowering).
#[derive(Clone, Debug, Default)]
pub struct NativeRequestDeviceDescriptor<'a> {
    pub required_features: &'a [i32],
    pub required_limits_rep: i32,
    pub label: &'a str,
    pub default_queue_label: &'a str,
}

/// Table-backed adapter info until Dawn C `wgpuAdapterGetInfo`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeAdapterInfo {
    pub vendor: String,
    pub architecture: String,
    pub device: String,
    pub description: String,
    pub subgroup_min_size: u32,
    pub subgroup_max_size: u32,
    pub is_fallback_adapter: bool,
}

/// Shader-module `compilation-hints` leftover. Dawn C has no ctor slot — Record only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeShaderHints {
    pub entries: String,
    pub layouts: Vec<i32>,
}

/// Pipeline `constants` map copied from WIT `record-gpu-pipeline-constant-value`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativePipelineConstants {
    pub compute: Vec<(String, f64)>,
    pub vertex: Vec<(String, f64)>,
    pub fragment: Vec<(String, f64)>,
}

/// Table-backed query-set descriptor (Dawn C slot still 0).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeQuerySet {
    pub ty: u32,
    pub count: u32,
}

/// One host copy of guest `write-*-with-copy` bytes (no JNI bounce).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeQueueWrite {
    pub target: u32,
    pub offset: u64,
    pub bytes: Vec<u8>,
}

/// Table-backed buffer leftover until Dawn C is bound.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeBuffer {
    pub size: u64,
    pub usage: u32,
    pub mapped: bool,
    pub mapped_bytes: Vec<u8>,
}

/// H9: `ANativeWindow_setBufferCount` before configure (3 = EINVAL on BLAST).
pub const SWAPCHAIN_BUFFER_COUNT: u32 = 4;
/// C2 / H26: keep last N presented images until GPU done **and** newer presents.
/// keep-6 (2026-09-01) raised hitch frequency; keep-8 starved the BLAST pool.
pub const CANVAS_FRAMES_TO_KEEP: usize = 3;

/// Table-backed present mode. Mailbox is not the product default (H6).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativePresentMode {
    Fifo,
}

/// Store / `bindCanvasNativeWindow` handle. Opaque `ANativeWindow*` as `i64`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeCanvasWindow {
    pub native_window: i64,
    pub width: u32,
    pub height: u32,
    pub buffer_count: u32,
    pub present_mode: NativePresentMode,
}

/// Table-backed `gpu-canvas-context` after configure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeCanvasContext {
    pub configured: bool,
    pub device: u32,
    pub format: u32,
    pub usage: u32,
    pub surface: Option<u32>,
    pub color_space: i32,
    pub tone_mapping: i32,
    pub alpha_mode: i32,
    pub view_formats: Vec<i32>,
}

/// Swapchain frame: present then recycle after GPU done + keep-3 (never same-present close).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeCanvasFrame {
    pub surface: u32,
    pub texture: u32,
    pub gpu_done: bool,
}

/// Vsync period + desired-present stamp (Fifo BLAST timestamp).
#[derive(Clone, Copy, Debug, Default)]
struct GfxHitchLog {
    vsync_ns: u64,
    /// Last measured Choreographer period (4–20 ms); else 8.333 ms.
    period_ns: u64,
    last_desired_ns: u64,
}

/// D3: beats of Choreographer vsync to put on the next queued BLAST buffer.
/// `0` disables. Default **2**. Device: `setprop debug.wasmtime.gfx.desired_present_beats`.
fn hitch_desired_present_beats() -> u32 {
    static N: OnceLock<u32> = OnceLock::new();
    *N.get_or_init(|| {
        if let Ok(s) = std::env::var("WASMTIME_GFX_DESIRED_PRESENT_BEATS") {
            if let Ok(n) = s.parse::<u32>() {
                return n.min(8);
            }
        }
        #[cfg(target_os = "android")]
        {
            let mut buf = [0u8; 92];
            unsafe extern "C" {
                fn __system_property_get(name: *const i8, value: *mut i8) -> i32;
            }
            let n = unsafe {
                __system_property_get(
                    c"debug.wasmtime.gfx.desired_present_beats".as_ptr() as *const i8,
                    buf.as_mut_ptr() as *mut i8,
                )
            };
            if n > 0 {
                if let Ok(s) = std::str::from_utf8(&buf[..n as usize]) {
                    if let Ok(v) = s.parse::<u32>() {
                        return v.min(8);
                    }
                }
            }
        }
        2
    })
}

/// Target present time = `vsync_ns + beats * period_ns`, monotonic vs last stamp.
fn desired_present_ns(
    vsync_ns: u64,
    period_ns: u64,
    beats: u32,
    last_desired_ns: u64,
) -> Option<i64> {
    if beats == 0 || vsync_ns == 0 || period_ns == 0 {
        return None;
    }
    let mut desired = vsync_ns.saturating_add(period_ns.saturating_mul(beats as u64));
    if last_desired_ns > 0 {
        let min_next = last_desired_ns.saturating_add(period_ns);
        if desired < min_next {
            desired = min_next;
        }
    }
    Some(desired as i64)
}

/// NDK omits this; `libnativewindow.so` still exports it (same as `setBufferCount`).
#[cfg(target_os = "android")]
fn native_window_set_buffers_timestamp(window: i64, timestamp: i64) -> i32 {
    if window == 0 {
        return -1;
    }
    type FnTs = unsafe extern "C" fn(*mut std::ffi::c_void, i64) -> i32;
    static FN: OnceLock<Option<FnTs>> = OnceLock::new();
    match *FN.get_or_init(|| unsafe {
        extern "C" {
            fn dlopen(filename: *const i8, flags: i32) -> *mut std::ffi::c_void;
            fn dlsym(handle: *mut std::ffi::c_void, symbol: *const i8) -> *mut std::ffi::c_void;
        }
        const RTLD_NOW: i32 = 2;
        let lib = dlopen(c"libnativewindow.so".as_ptr() as *const i8, RTLD_NOW);
        if lib.is_null() {
            return None;
        }
        let sym = dlsym(
            lib,
            c"ANativeWindow_setBuffersTimestamp".as_ptr() as *const i8,
        );
        if sym.is_null() {
            return None;
        }
        Some(std::mem::transmute(sym))
    }) {
        Some(f) => unsafe { f(window as *mut std::ffi::c_void, timestamp) },
        None => -2,
    }
}

/// Table-backed texture leftover until Dawn C is bound.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeTexture {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub format: u32,
    pub usage: u32,
    pub mip: u32,
    pub sample: u32,
    pub dimension: u32,
}

impl Default for NativeAdapterInfo {
    fn default() -> Self {
        Self {
            vendor: String::new(),
            architecture: String::new(),
            device: "native-gpu".into(),
            description: "table-backed NativeGpu (Dawn C slot 0)".into(),
            subgroup_min_size: 4,
            subgroup_max_size: 128,
            is_fallback_adapter: false,
        }
    }
}

/// Table-backed [`NativeGpu`]. Dawn C slots stay 0 until a consume lane dlopens.
#[derive(Debug)]
pub struct NativeGpuHost {
    table: HandleTable,
    interned_queues: HashMap<u32, u32>,
    adapter_info: HashMap<u32, NativeAdapterInfo>,
    /// Shader `compilation-hints` Record leftover (Dawn C has no slot).
    shader_hints: HashMap<u32, NativeShaderHints>,
    /// WIT `record-gpu-pipeline-constant-value` maps keyed by resource `rep`.
    pipeline_constant_records: HashMap<u32, Vec<(String, f64)>>,
    /// Constants copied onto compute/render pipelines at create (Dawn slot still 0).
    pipeline_constants: HashMap<u32, NativePipelineConstants>,
    /// `gpu-query-set` type/count leftover until Dawn C is bound.
    query_sets: HashMap<u32, NativeQuerySet>,
    /// Last `write-buffer-with-copy` / `write-texture-with-copy` per queue (one copy).
    queue_writes: HashMap<u32, NativeQueueWrite>,
    /// Last `queue.submit` command-buffer reps.
    last_submit: Vec<u32>,
    /// WIT `record-option-gpu-size64` maps keyed by resource `rep`.
    size64_records: HashMap<u32, Vec<(String, Option<u64>)>>,
    /// `.label` / `.set-label` leftover (Dawn C slot still 0).
    labels: HashMap<u32, String>,
    /// Buffer size / usage / map leftover until Dawn C is bound.
    buffers: HashMap<u32, NativeBuffer>,
    /// Texture getter leftover until Dawn C is bound.
    textures: HashMap<u32, NativeTexture>,
    /// `bindCanvasNativeWindow` / Store window handle (opaque `ANativeWindow*`).
    canvas_window: Option<NativeCanvasWindow>,
    /// Configured `gpu-canvas-context` leftover (color-space / tone-mapping Record).
    canvas_contexts: HashMap<u32, NativeCanvasContext>,
    /// Acquired, not yet presented (H8: present clears this).
    pending_present: Option<NativeCanvasFrame>,
    /// Presented images; close only after GPU done + keep-3 (C2).
    presented_ring: VecDeque<NativeCanvasFrame>,
    /// How many times `canvas_present` actually presented (not no-op).
    present_count: u32,
    /// Dawn C instance (0 = table-backed / `.so` missing).
    dawn_instance: DawnSlot,
    /// One `WGPUSurface` for the bound `ANativeWindow`.
    dawn_surface: DawnSlot,
    dawn_surface_format: u32,
    dawn_surface_configured: bool,
    /// `WGPUCompositeAlphaMode` chosen at configure (caps.alphaModes[0], matching D24).
    dawn_surface_alpha_mode: u32,
    hitch: GfxHitchLog,
}

impl Default for NativeGpuHost {
    fn default() -> Self {
        Self::new()
    }
}

fn feature_enum(name: &str) -> u32 {
    match name {
        "depth-clip-control" => 0x2,
        "depth32float-stencil8" => 0x3,
        "texture-compression-bc" => 0x4,
        "texture-compression-bc-sliced-3d" => 0x5,
        "texture-compression-etc2" => 0x6,
        "texture-compression-astc" => 0x7,
        "texture-compression-astc-sliced-3d" => 0x8,
        "timestamp-query" => 0x9,
        "indirect-first-instance" => 0xA,
        "shader-f16" => 0xB,
        "rg11b10ufloat-renderable" => 0xC,
        "bgra8unorm-storage" => 0xD,
        "float32-filterable" => 0xE,
        "float32-blendable" => 0xF,
        "clip-distances" => 0x10,
        "dual-source-blending" => 0x11,
        "subgroups" => 0x12,
        _ => 0,
    }
}

impl NativeGpuHost {
    /// Best-effort `dlopen` + `dlsym` of `libwebgpu_dawn.so`. Missing `.so`
    /// (Cloud / recipe not run) is not an error; Dawn C slots stay 0.
    pub fn try_load_dawn_c() -> bool {
        dawn_c::try_load()
    }

    fn dawn_of(&self, handle: GpuHandle) -> DawnSlot {
        self.table.dawn_of(handle)
    }

    fn ensure_instance(&mut self) -> DawnSlot {
        if self.dawn_instance == 0 && Self::try_load_dawn_c() {
            self.dawn_instance = dawn_c::create_instance();
        }
        self.dawn_instance
    }

    pub fn new() -> Self {
        Self {
            table: HandleTable::new(),
            interned_queues: HashMap::new(),
            adapter_info: HashMap::new(),
            shader_hints: HashMap::new(),
            pipeline_constant_records: HashMap::new(),
            pipeline_constants: HashMap::new(),
            query_sets: HashMap::new(),
            queue_writes: HashMap::new(),
            last_submit: Vec::new(),
            size64_records: HashMap::new(),
            labels: HashMap::new(),
            buffers: HashMap::new(),
            textures: HashMap::new(),
            canvas_window: None,
            canvas_contexts: HashMap::new(),
            pending_present: None,
            presented_ring: VecDeque::new(),
            present_count: 0,
            dawn_instance: 0,
            dawn_surface: 0,
            dawn_surface_format: 0,
            dawn_surface_configured: false,
            dawn_surface_alpha_mode: 0,
            hitch: GfxHitchLog::default(),
        }
    }

    /// Consumed Choreographer `frameTimeNanos` for vsync→present stamp.
    pub fn note_consumed_vsync(&mut self, vsync_ns: u64) {
        if self.hitch.vsync_ns > 0 && vsync_ns > self.hitch.vsync_ns {
            let d = vsync_ns - self.hitch.vsync_ns;
            if (4_000_000..=20_000_000).contains(&d) {
                self.hitch.period_ns = d;
            }
        }
        self.hitch.vsync_ns = vsync_ns;
    }

    fn hitch_period_ns(&self) -> u64 {
        if self.hitch.period_ns >= 4_000_000 {
            self.hitch.period_ns
        } else {
            8_333_333
        }
    }

    fn stamp_desired_present(&mut self) {
        let beats = hitch_desired_present_beats();
        let vsync = self.hitch.vsync_ns;
        let period = self.hitch_period_ns();
        let Some(ts) = desired_present_ns(vsync, period, beats, self.hitch.last_desired_ns) else {
            return;
        };
        self.hitch.last_desired_ns = ts as u64;
        #[cfg(target_os = "android")]
        {
            let window = self.canvas_window.map(|w| w.native_window).unwrap_or(0);
            let _ = native_window_set_buffers_timestamp(window, ts);
        }
    }

    fn forget_side(&mut self, handle: GpuHandle, kind: ResourceKind) {
        match kind {
            ResourceKind::Adapter => {
                self.adapter_info.remove(&handle.raw());
            }
            ResourceKind::Device => {
                self.interned_queues.remove(&handle.raw());
            }
            ResourceKind::ShaderModule => {
                self.shader_hints.remove(&handle.raw());
            }
            ResourceKind::ComputePipeline | ResourceKind::RenderPipeline => {
                self.pipeline_constants.remove(&handle.raw());
            }
            ResourceKind::QuerySet => {
                self.query_sets.remove(&handle.raw());
            }
            ResourceKind::Queue => {
                self.queue_writes.remove(&handle.raw());
            }
            ResourceKind::Buffer => {
                self.buffers.remove(&handle.raw());
            }
            ResourceKind::Texture => {
                self.textures.remove(&handle.raw());
            }
            ResourceKind::CanvasContext => {
                self.canvas_contexts.remove(&handle.raw());
            }
            _ => {}
        }
        self.labels.remove(&handle.raw());
    }

    /// C2: do not `wgpuTextureRelease` a just-presented swapchain image.
    fn is_live_swapchain_texture(&self, handle: GpuHandle) -> bool {
        let raw = handle.raw();
        self.pending_present.is_some_and(|f| f.texture == raw)
            || self.presented_ring.iter().any(|f| f.texture == raw)
    }

    fn release_dawn_entry(&mut self, handle: GpuHandle, entry: HandleEntry) {
        if entry.dawn == 0 {
            return;
        }
        if entry.kind == ResourceKind::Texture && self.is_live_swapchain_texture(handle) {
            return;
        }
        dawn_c::release(entry.kind, entry.dawn);
    }

    /// `rep == 0` is the fixture `get-adapter` stub; otherwise a live table id.
    pub fn resolve_adapter(&mut self, adapter_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if adapter_rep == GpuHandle::NULL {
            self.request_adapter(&NativeRequestAdapterOptions::default())
                .ok_or(NativeGpuError::AdapterUnavailable)
        } else {
            let handle = GpuHandle::from_raw(adapter_rep)?;
            self.get(handle, ResourceKind::Adapter)?;
            Ok(handle)
        }
    }

    /// `rep == 0` is the fixture `get-device` stub; otherwise a live table id.
    pub fn resolve_device(&mut self, device_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if device_rep == GpuHandle::NULL {
            let adapter = self.resolve_adapter(GpuHandle::NULL)?;
            self.request_device(adapter, &NativeRequestDeviceDescriptor::default())
        } else {
            let handle = GpuHandle::from_raw(device_rep)?;
            self.get(handle, ResourceKind::Device)?;
            Ok(handle)
        }
    }

    pub fn request_adapter(
        &mut self,
        options: &NativeRequestAdapterOptions<'_>,
    ) -> Option<GpuHandle> {
        let mut info = NativeAdapterInfo::default();
        info.is_fallback_adapter = options.force_fallback_adapter;
        let _ = options.feature_level;
        let _ = options.power_preference;
        let _ = options.xr_compatible;
        let instance = self.ensure_instance();
        let dawn = if instance != 0 {
            dawn_c::request_adapter_vulkan(instance)
        } else {
            0
        };
        if dawn != 0 {
            info.description = "Dawn C Vulkan adapter".into();
        }
        let handle = self.table.insert(ResourceKind::Adapter, dawn);
        self.adapter_info.insert(handle.raw(), info);
        Some(handle)
    }

    pub fn request_device(
        &mut self,
        adapter: GpuHandle,
        desc: &NativeRequestDeviceDescriptor<'_>,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(adapter, ResourceKind::Adapter)?;
        let _ = desc.required_features;
        let _ = desc.required_limits_rep;
        let _ = desc.label;
        let _ = desc.default_queue_label;
        let instance = self.ensure_instance();
        let dawn = dawn_c::request_device(instance, self.dawn_of(adapter));
        Ok(self.table.insert(ResourceKind::Device, dawn))
    }

    pub fn device_queue(&mut self, device: GpuHandle) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        if let Some(&raw) = self.interned_queues.get(&device.raw()) {
            if let Ok(existing) = GpuHandle::from_raw(raw) {
                if self.table.contains(existing) {
                    let _ = self.get(existing, ResourceKind::Queue)?;
                    return Ok(existing);
                }
            }
        }
        let dawn = dawn_c::device_queue(self.dawn_of(device));
        let queue = self.table.insert(ResourceKind::Queue, dawn);
        self.interned_queues.insert(device.raw(), queue.raw());
        Ok(queue)
    }

    pub fn adapter_info(&self, adapter: GpuHandle) -> Result<NativeAdapterInfo, NativeGpuError> {
        self.get(adapter, ResourceKind::Adapter)?;
        Ok(self
            .adapter_info
            .get(&adapter.raw())
            .cloned()
            .unwrap_or_default())
    }

    pub fn adapter_has_feature(
        &self,
        adapter: GpuHandle,
        name: &str,
    ) -> Result<bool, NativeGpuError> {
        self.get(adapter, ResourceKind::Adapter)?;
        let dawn = self.dawn_of(adapter);
        if dawn == 0 {
            return Ok(false);
        }
        Ok(dawn_c::adapter_has_feature(dawn, feature_enum(name)))
    }

    pub fn resolve_texture(&mut self, texture_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if texture_rep == GpuHandle::NULL {
            let device = self.resolve_device(GpuHandle::NULL)?;
            self.create_texture(device, 1, 1, 1, 0, 0, 1, 1, 2, &[], "")
        } else {
            let handle = GpuHandle::from_raw(texture_rep)?;
            self.get(handle, ResourceKind::Texture)?;
            Ok(handle)
        }
    }

    pub fn create_buffer(
        &mut self,
        device: GpuHandle,
        size: u64,
        usage: u32,
        mapped_at_creation: i32,
        label: &str,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let mapped = mapped_at_creation == 1;
        let dawn = dawn_c::create_buffer(self.dawn_of(device), size, usage, mapped, label);
        let handle = self.table.insert(ResourceKind::Buffer, dawn);
        if !label.is_empty() {
            self.labels.insert(handle.raw(), label.to_string());
        }
        self.buffers.insert(
            handle.raw(),
            NativeBuffer {
                size,
                usage,
                mapped,
                mapped_bytes: if mapped {
                    vec![0; size.min(4096) as usize]
                } else {
                    Vec::new()
                },
            },
        );
        Ok(handle)
    }

    pub fn create_texture(
        &mut self,
        device: GpuHandle,
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
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let dawn = dawn_c::create_texture(
            self.dawn_of(device),
            width,
            height,
            depth,
            format,
            usage,
            mip,
            sample,
            dimension,
            view_formats,
            label,
        );
        let handle = self.table.insert(ResourceKind::Texture, dawn);
        if !label.is_empty() {
            self.labels.insert(handle.raw(), label.to_string());
        }
        self.textures.insert(
            handle.raw(),
            NativeTexture {
                width,
                height,
                depth,
                format,
                usage,
                mip,
                sample,
                dimension,
            },
        );
        Ok(handle)
    }

    pub fn create_sampler(
        &mut self,
        device: GpuHandle,
        mag_filter: u32,
        min_filter: u32,
        address_mode_u: u32,
        address_mode_v: u32,
        address_mode_w: u32,
        mipmap_filter: u32,
        compare: u32,
        has_lod_min: i32,
        lod_min: f32,
        has_lod_max: i32,
        lod_max: f32,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let dawn = dawn_c::create_sampler(
            self.dawn_of(device),
            mag_filter,
            min_filter,
            address_mode_u,
            address_mode_v,
            address_mode_w,
            mipmap_filter,
            compare,
            if has_lod_min != 0 { lod_min } else { 0.0 },
            if has_lod_max != 0 { lod_max } else { 32.0 },
        );
        Ok(self.table.insert(ResourceKind::Sampler, dawn))
    }

    pub fn create_shader_module(
        &mut self,
        device: GpuHandle,
        code: &str,
        label: &str,
        hint_layouts: &[i32],
        hint_entries: &str,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let dawn = dawn_c::create_shader(self.dawn_of(device), code, label);
        let handle = self.table.insert(ResourceKind::ShaderModule, dawn);
        if !hint_layouts.is_empty() || !hint_entries.is_empty() {
            self.shader_hints.insert(
                handle.raw(),
                NativeShaderHints {
                    entries: hint_entries.to_string(),
                    layouts: hint_layouts.to_vec(),
                },
            );
        }
        Ok(handle)
    }

    pub fn shader_compilation_hints(
        &self,
        shader: GpuHandle,
    ) -> Result<Option<&NativeShaderHints>, NativeGpuError> {
        self.get(shader, ResourceKind::ShaderModule)?;
        Ok(self.shader_hints.get(&shader.raw()))
    }

    pub fn create_texture_view(
        &mut self,
        texture: GpuHandle,
        dimension: u32,
        aspect: u32,
        format: u32,
        base_mip: i32,
        mip_count: i32,
        base_layer: i32,
        layer_count: i32,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(texture, ResourceKind::Texture)?;
        let dawn = if self.dawn_of(texture) != 0 {
            dawn_c::create_view(
                self.dawn_of(texture),
                dimension,
                aspect,
                format,
                if base_mip < 0 { 0 } else { base_mip as u32 },
                if mip_count <= 0 { 0 } else { mip_count as u32 },
                if base_layer < 0 { 0 } else { base_layer as u32 },
                if layer_count <= 0 {
                    0
                } else {
                    layer_count as u32
                },
            )
        } else {
            0
        };
        Ok(self.table.insert(ResourceKind::TextureView, dawn))
    }

    pub fn resolve_shader(&mut self, shader_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if shader_rep == GpuHandle::NULL {
            let device = self.resolve_device(GpuHandle::NULL)?;
            self.create_shader_module(device, "", "", &[], "")
        } else {
            let handle = GpuHandle::from_raw(shader_rep)?;
            self.get(handle, ResourceKind::ShaderModule)?;
            Ok(handle)
        }
    }

    pub fn resolve_bind_group_layout(
        &mut self,
        layout_rep: u32,
    ) -> Result<GpuHandle, NativeGpuError> {
        if layout_rep == GpuHandle::NULL {
            let device = self.resolve_device(GpuHandle::NULL)?;
            self.create_bind_group_layout(device, &[], &[], &[], &[], &[])
        } else {
            let handle = GpuHandle::from_raw(layout_rep)?;
            self.get(handle, ResourceKind::BindGroupLayout)?;
            Ok(handle)
        }
    }

    pub fn resolve_buffer(&mut self, buffer_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if buffer_rep == GpuHandle::NULL {
            let device = self.resolve_device(GpuHandle::NULL)?;
            self.create_buffer(device, 0, 0, -1, "")
        } else {
            let handle = GpuHandle::from_raw(buffer_rep)?;
            self.get(handle, ResourceKind::Buffer)?;
            Ok(handle)
        }
    }

    pub fn create_bind_group_layout(
        &mut self,
        device: GpuHandle,
        bindings: &[i32],
        visibilities: &[i32],
        buffer_types: &[i32],
        sampler_types: &[i32],
        texture_sample_types: &[i32],
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let dawn = dawn_c::create_bind_group_layout(
            self.dawn_of(device),
            bindings,
            visibilities,
            buffer_types,
            sampler_types,
            texture_sample_types,
        );
        Ok(self.table.insert(ResourceKind::BindGroupLayout, dawn))
    }

    pub fn create_pipeline_layout(
        &mut self,
        device: GpuHandle,
        bind_group_layouts: &[i32],
        label: &str,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        for &raw in bind_group_layouts {
            if raw > 0 {
                let h = GpuHandle::from_raw(raw as u32)?;
                self.get(h, ResourceKind::BindGroupLayout)?;
            }
        }
        let _ = label;
        let slots: Vec<DawnSlot> = bind_group_layouts
            .iter()
            .filter_map(|&raw| {
                if raw > 0 {
                    GpuHandle::from_raw(raw as u32)
                        .ok()
                        .map(|h| self.dawn_of(h))
                } else {
                    None
                }
            })
            .collect();
        let dawn = dawn_c::create_pipeline_layout(self.dawn_of(device), &slots, label);
        Ok(self.table.insert(ResourceKind::PipelineLayout, dawn))
    }

    pub fn create_bind_group(
        &mut self,
        device: GpuHandle,
        layout: GpuHandle,
        label: &str,
        bindings: &[i32],
        kinds: &[i32],
        handles: &[i32],
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        self.get(layout, ResourceKind::BindGroupLayout)?;
        let slots: Vec<DawnSlot> = handles
            .iter()
            .filter_map(|&raw| {
                if raw > 0 {
                    GpuHandle::from_raw(raw as u32)
                        .ok()
                        .map(|h| self.dawn_of(h))
                } else {
                    Some(0)
                }
            })
            .collect();
        let dawn = dawn_c::create_bind_group(
            self.dawn_of(device),
            self.dawn_of(layout),
            bindings,
            kinds,
            &slots,
            label,
        );
        Ok(self.table.insert(ResourceKind::BindGroup, dawn))
    }

    fn copy_constant_record(&self, rep: i32) -> Vec<(String, f64)> {
        if rep <= 0 {
            return Vec::new();
        }
        self.pipeline_constant_records
            .get(&(rep as u32))
            .cloned()
            .unwrap_or_default()
    }

    pub fn pipeline_constant_add(&mut self, handle: u32, key: String, value: f64) {
        let rec = self.pipeline_constant_records.entry(handle).or_default();
        if let Some(slot) = rec.iter_mut().find(|(k, _)| *k == key) {
            slot.1 = value;
        } else {
            rec.push((key, value));
        }
    }

    pub fn pipeline_constant_get(&self, handle: u32, key: &str) -> Option<f64> {
        self.pipeline_constant_records
            .get(&handle)
            .and_then(|rec| rec.iter().find(|(k, _)| k == key).map(|(_, v)| *v))
    }

    pub fn pipeline_constant_has(&self, handle: u32, key: &str) -> bool {
        self.pipeline_constant_get(handle, key).is_some()
    }

    pub fn pipeline_constant_remove(&mut self, handle: u32, key: &str) {
        if let Some(rec) = self.pipeline_constant_records.get_mut(&handle) {
            rec.retain(|(k, _)| k != key);
        }
    }

    pub fn pipeline_constant_keys(&self, handle: u32) -> Vec<String> {
        self.pipeline_constant_records
            .get(&handle)
            .map(|rec| rec.iter().map(|(k, _)| k.clone()).collect())
            .unwrap_or_default()
    }

    pub fn pipeline_constant_values(&self, handle: u32) -> Vec<f64> {
        self.pipeline_constant_records
            .get(&handle)
            .map(|rec| rec.iter().map(|(_, v)| *v).collect())
            .unwrap_or_default()
    }

    pub fn pipeline_constant_entries(&self, handle: u32) -> Vec<(String, f64)> {
        self.pipeline_constant_records
            .get(&handle)
            .cloned()
            .unwrap_or_default()
    }

    pub fn create_compute_pipeline(
        &mut self,
        device: GpuHandle,
        shader_rep: u32,
        entry_point: &str,
        layout_rep: i32,
        label: &str,
        constants_rep: i32,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let shader = self.resolve_shader(shader_rep)?;
        let layout_dawn = if layout_rep > 0 {
            let h = GpuHandle::from_raw(layout_rep as u32)?;
            self.get(h, ResourceKind::PipelineLayout)?;
            self.dawn_of(h)
        } else {
            0
        };
        let dawn = dawn_c::create_compute_pipeline(
            self.dawn_of(device),
            layout_dawn,
            self.dawn_of(shader),
            entry_point,
            label,
        );
        let handle = self.table.insert(ResourceKind::ComputePipeline, dawn);
        self.pipeline_constants.insert(
            handle.raw(),
            NativePipelineConstants {
                compute: self.copy_constant_record(constants_rep),
                ..NativePipelineConstants::default()
            },
        );
        Ok(handle)
    }

    pub fn create_render_pipeline(
        &mut self,
        device: GpuHandle,
        vertex_shader: u32,
        vertex_entry: &str,
        fragment_shader: i32,
        fragment_entry: &str,
        format: i32,
        layout_rep: i32,
        label: &str,
        vertex_constants: i32,
        fragment_constants: i32,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.create_render_pipeline_described(
            device,
            vertex_shader,
            vertex_entry,
            fragment_shader,
            fragment_entry,
            format,
            layout_rep,
            label,
            vertex_constants,
            fragment_constants,
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
        )
    }

    pub fn create_render_pipeline_described(
        &mut self,
        device: GpuHandle,
        vertex_shader: u32,
        vertex_entry: &str,
        fragment_shader: i32,
        fragment_entry: &str,
        format: i32,
        layout_rep: i32,
        label: &str,
        vertex_constants: i32,
        fragment_constants: i32,
        vb_strides: &[i32],
        vb_step_modes: &[i32],
        attr_index: &[i32],
        attr_formats: &[i32],
        attr_offsets: &[i32],
        attr_locations: &[i32],
        primitive: &[i32],
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let vs = self.resolve_shader(vertex_shader)?;
        let fs = if fragment_shader > 0 {
            self.resolve_shader(fragment_shader as u32)?
        } else {
            vs
        };
        let layout_dawn = if layout_rep > 0 {
            let h = GpuHandle::from_raw(layout_rep as u32)?;
            self.get(h, ResourceKind::PipelineLayout)?;
            self.dawn_of(h)
        } else {
            0
        };
        let dawn = dawn_c::create_render_pipeline(
            self.dawn_of(device),
            layout_dawn,
            self.dawn_of(vs),
            vertex_entry,
            self.dawn_of(fs),
            fragment_entry,
            format as u32,
            primitive,
            dawn_c::VertexPack {
                strides: vb_strides,
                step_modes: vb_step_modes,
                attr_index,
                attr_formats,
                attr_offsets,
                attr_locations,
            },
            label,
        );
        let handle = self.table.insert(ResourceKind::RenderPipeline, dawn);
        self.pipeline_constants.insert(
            handle.raw(),
            NativePipelineConstants {
                vertex: self.copy_constant_record(vertex_constants),
                fragment: self.copy_constant_record(fragment_constants),
                ..NativePipelineConstants::default()
            },
        );
        Ok(handle)
    }

    pub fn pipeline_constants(
        &self,
        pipeline: GpuHandle,
    ) -> Result<Option<&NativePipelineConstants>, NativeGpuError> {
        let entry = self
            .table
            .get(pipeline, ResourceKind::ComputePipeline)
            .or_else(|_| self.table.get(pipeline, ResourceKind::RenderPipeline))?;
        let _ = entry;
        Ok(self.pipeline_constants.get(&pipeline.raw()))
    }

    pub fn resolve_texture_view(&mut self, view_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if view_rep == GpuHandle::NULL {
            let texture = self.resolve_texture(GpuHandle::NULL)?;
            self.create_texture_view(texture, 0, 0, 0, 0, 1, 0, 1)
        } else {
            let handle = GpuHandle::from_raw(view_rep)?;
            self.get(handle, ResourceKind::TextureView)?;
            Ok(handle)
        }
    }

    pub fn resolve_bind_group(&mut self, bind_group_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if bind_group_rep == GpuHandle::NULL {
            let device = self.resolve_device(GpuHandle::NULL)?;
            let layout = self.resolve_bind_group_layout(GpuHandle::NULL)?;
            self.create_bind_group(device, layout, "", &[], &[], &[])
        } else {
            let handle = GpuHandle::from_raw(bind_group_rep)?;
            self.get(handle, ResourceKind::BindGroup)?;
            Ok(handle)
        }
    }

    pub fn resolve_render_pipeline(
        &mut self,
        pipeline_rep: u32,
    ) -> Result<GpuHandle, NativeGpuError> {
        if pipeline_rep == GpuHandle::NULL {
            let device = self.resolve_device(GpuHandle::NULL)?;
            self.create_render_pipeline(device, 0, "", 0, "", 0, 0, "", 0, 0)
        } else {
            let handle = GpuHandle::from_raw(pipeline_rep)?;
            self.get(handle, ResourceKind::RenderPipeline)?;
            Ok(handle)
        }
    }

    pub fn resolve_compute_pipeline(
        &mut self,
        pipeline_rep: u32,
    ) -> Result<GpuHandle, NativeGpuError> {
        if pipeline_rep == GpuHandle::NULL {
            let device = self.resolve_device(GpuHandle::NULL)?;
            self.create_compute_pipeline(device, 0, "", 0, "", 0)
        } else {
            let handle = GpuHandle::from_raw(pipeline_rep)?;
            self.get(handle, ResourceKind::ComputePipeline)?;
            Ok(handle)
        }
    }

    pub fn create_command_encoder(
        &mut self,
        device: GpuHandle,
        label: &str,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let dawn = dawn_c::create_encoder(self.dawn_of(device), label);
        Ok(self.table.insert(ResourceKind::CommandEncoder, dawn))
    }

    pub fn resolve_encoder(&mut self, encoder_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if encoder_rep == GpuHandle::NULL {
            let device = self.resolve_device(GpuHandle::NULL)?;
            self.create_command_encoder(device, "")
        } else {
            let handle = GpuHandle::from_raw(encoder_rep)?;
            self.get(handle, ResourceKind::CommandEncoder)?;
            Ok(handle)
        }
    }

    pub fn create_query_set(
        &mut self,
        device: GpuHandle,
        ty: u32,
        count: u32,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let dawn = dawn_c::create_query_set(self.dawn_of(device), ty, count);
        let handle = self.table.insert(ResourceKind::QuerySet, dawn);
        self.query_sets
            .insert(handle.raw(), NativeQuerySet { ty, count });
        Ok(handle)
    }

    pub fn resolve_query_set(&mut self, query_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if query_rep == GpuHandle::NULL {
            let device = self.resolve_device(GpuHandle::NULL)?;
            self.create_query_set(device, 0, 1)
        } else {
            let handle = GpuHandle::from_raw(query_rep)?;
            self.get(handle, ResourceKind::QuerySet)?;
            Ok(handle)
        }
    }

    pub fn query_set_type(&self, query: GpuHandle) -> Result<u32, NativeGpuError> {
        self.get(query, ResourceKind::QuerySet)?;
        Ok(self.query_sets.get(&query.raw()).map(|q| q.ty).unwrap_or(0))
    }

    pub fn query_set_count(&self, query: GpuHandle) -> Result<u32, NativeGpuError> {
        self.get(query, ResourceKind::QuerySet)?;
        Ok(self
            .query_sets
            .get(&query.raw())
            .map(|q| q.count)
            .unwrap_or(1))
    }

    pub fn query_set_destroy(&mut self, query: GpuHandle) -> Result<(), NativeGpuError> {
        self.get(query, ResourceKind::QuerySet)?;
        dawn_c::destroy(ResourceKind::QuerySet, self.dawn_of(query));
        Ok(())
    }

    pub fn destroy_rep(&mut self, kind: ResourceKind, rep: u32) -> Result<(), NativeGpuError> {
        if rep == GpuHandle::NULL {
            return Ok(());
        }
        let handle = GpuHandle::from_raw(rep)?;
        self.get(handle, kind)?;
        dawn_c::destroy(kind, self.dawn_of(handle));
        Ok(())
    }

    pub fn push_error_scope(&mut self, device_rep: u32, filter: u32) -> Result<(), NativeGpuError> {
        let device = self.resolve_device(device_rep)?;
        dawn_c::push_error_scope(self.dawn_of(device), filter);
        Ok(())
    }

    pub fn pop_error_scope(&mut self, device_rep: u32) -> Result<(), NativeGpuError> {
        let device = self.resolve_device(device_rep)?;
        let instance = self.ensure_instance();
        dawn_c::pop_error_scope(instance, self.dawn_of(device));
        Ok(())
    }

    pub fn begin_render_pass(
        &mut self,
        encoder: GpuHandle,
        color_views: &[i32],
        depth_view: u32,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(encoder, ResourceKind::CommandEncoder)?;
        for &view in color_views {
            if view >= 0 {
                let _ = self.resolve_texture_view(view as u32)?;
            }
        }
        let _ = self.resolve_texture_view(depth_view)?;
        self.begin_render_pass_described(encoder, color_views, &[], &[], &[], &[], depth_view)
    }

    pub fn begin_render_pass_described(
        &mut self,
        encoder: GpuHandle,
        color_views: &[i32],
        color_loads: &[i32],
        color_stores: &[i32],
        color_has_clears: &[i32],
        color_clear_bits: &[i32],
        depth_view: u32,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(encoder, ResourceKind::CommandEncoder)?;
        let mut views = Vec::with_capacity(color_views.len());
        for &view in color_views {
            if view > 0 {
                let h = self.resolve_texture_view(view as u32)?;
                views.push(self.dawn_of(h));
            }
        }
        let depth_dawn = if depth_view != 0 {
            let h = self.resolve_texture_view(depth_view)?;
            self.dawn_of(h)
        } else {
            0
        };
        let dawn = dawn_c::begin_render_pass(
            self.dawn_of(encoder),
            &views,
            color_loads,
            color_stores,
            color_has_clears,
            color_clear_bits,
            depth_dawn,
        );
        Ok(self.table.insert(ResourceKind::RenderPassEncoder, dawn))
    }

    pub fn resolve_render_pass(&mut self, pass_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if pass_rep == GpuHandle::NULL {
            let encoder = self.resolve_encoder(GpuHandle::NULL)?;
            self.begin_render_pass(encoder, &[], 0)
        } else {
            let handle = GpuHandle::from_raw(pass_rep)?;
            self.get(handle, ResourceKind::RenderPassEncoder)?;
            Ok(handle)
        }
    }

    pub fn begin_compute_pass(
        &mut self,
        encoder: GpuHandle,
        query_rep: u32,
        begin_idx: u32,
        end_idx: u32,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(encoder, ResourceKind::CommandEncoder)?;
        let query_dawn = if query_rep != GpuHandle::NULL {
            let q = self.resolve_query_set(query_rep)?;
            self.dawn_of(q)
        } else {
            0
        };
        let dawn =
            dawn_c::begin_compute_pass(self.dawn_of(encoder), query_dawn, begin_idx, end_idx);
        Ok(self.table.insert(ResourceKind::ComputePassEncoder, dawn))
    }

    pub fn resolve_compute_pass(&mut self, pass_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if pass_rep == GpuHandle::NULL {
            let encoder = self.resolve_encoder(GpuHandle::NULL)?;
            self.begin_compute_pass(encoder, 0, 0, 0)
        } else {
            let handle = GpuHandle::from_raw(pass_rep)?;
            self.get(handle, ResourceKind::ComputePassEncoder)?;
            Ok(handle)
        }
    }

    pub fn encoder_finish(
        &mut self,
        encoder: GpuHandle,
        label: &str,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(encoder, ResourceKind::CommandEncoder)?;
        let dawn = dawn_c::encoder_finish(self.dawn_of(encoder), label);
        Ok(self.table.insert(ResourceKind::CommandBuffer, dawn))
    }

    pub fn encoder_copy(
        &mut self,
        encoder_rep: u32,
        src_buffer: Option<u32>,
        dst_buffer: Option<u32>,
        src_texture: Option<u32>,
        dst_texture: Option<u32>,
    ) -> Result<(), NativeGpuError> {
        self.encoder_copy_sized(
            encoder_rep,
            src_buffer,
            dst_buffer,
            src_texture,
            dst_texture,
            0,
            0,
            0,
            1,
            1,
            1,
        )
    }

    pub fn encoder_copy_sized(
        &mut self,
        encoder_rep: u32,
        src_buffer: Option<u32>,
        dst_buffer: Option<u32>,
        src_texture: Option<u32>,
        dst_texture: Option<u32>,
        src_off: u64,
        dst_off: u64,
        size: u64,
        width: u32,
        height: u32,
        depth: u32,
    ) -> Result<(), NativeGpuError> {
        let encoder = self.resolve_encoder(encoder_rep)?;
        let enc = self.dawn_of(encoder);
        match (src_buffer, dst_buffer, src_texture, dst_texture) {
            (Some(src), Some(dst), None, None) => {
                let s = self.resolve_buffer(src)?;
                let d = self.resolve_buffer(dst)?;
                dawn_c::copy_buffer_to_buffer(
                    enc,
                    self.dawn_of(s),
                    src_off,
                    self.dawn_of(d),
                    dst_off,
                    size,
                );
            }
            (Some(src), None, None, Some(dst)) => {
                let s = self.resolve_buffer(src)?;
                let d = self.resolve_texture(dst)?;
                dawn_c::copy_buffer_to_texture(
                    enc,
                    self.dawn_of(s),
                    self.dawn_of(d),
                    width,
                    height,
                    depth,
                );
            }
            (None, Some(dst), Some(src), None) => {
                let s = self.resolve_texture(src)?;
                let d = self.resolve_buffer(dst)?;
                dawn_c::copy_texture_to_buffer(
                    enc,
                    self.dawn_of(s),
                    self.dawn_of(d),
                    width,
                    height,
                    depth,
                );
            }
            (None, None, Some(src), Some(dst)) => {
                let s = self.resolve_texture(src)?;
                let d = self.resolve_texture(dst)?;
                dawn_c::copy_texture_to_texture(
                    enc,
                    self.dawn_of(s),
                    self.dawn_of(d),
                    width,
                    height,
                    depth,
                );
            }
            _ => {}
        }
        Ok(())
    }

    pub fn encoder_clear_buffer(
        &mut self,
        encoder_rep: u32,
        buffer_rep: u32,
    ) -> Result<(), NativeGpuError> {
        self.encoder_clear_buffer_range(encoder_rep, buffer_rep, 0, 0)
    }

    pub fn encoder_clear_buffer_range(
        &mut self,
        encoder_rep: u32,
        buffer_rep: u32,
        offset: u64,
        size: u64,
    ) -> Result<(), NativeGpuError> {
        let encoder = self.resolve_encoder(encoder_rep)?;
        let buffer = self.resolve_buffer(buffer_rep)?;
        dawn_c::clear_buffer(self.dawn_of(encoder), self.dawn_of(buffer), offset, size);
        Ok(())
    }

    pub fn encoder_resolve_query_set(
        &mut self,
        encoder_rep: u32,
        query_rep: u32,
        dest_rep: u32,
    ) -> Result<(), NativeGpuError> {
        self.encoder_resolve_query_set_range(encoder_rep, query_rep, 0, 1, dest_rep, 0)
    }

    pub fn encoder_resolve_query_set_range(
        &mut self,
        encoder_rep: u32,
        query_rep: u32,
        first: u32,
        count: u32,
        dest_rep: u32,
        dest_off: u64,
    ) -> Result<(), NativeGpuError> {
        let encoder = self.resolve_encoder(encoder_rep)?;
        let query = self.resolve_query_set(query_rep)?;
        let dest = self.resolve_buffer(dest_rep)?;
        dawn_c::resolve_query_set(
            self.dawn_of(encoder),
            self.dawn_of(query),
            first,
            count,
            self.dawn_of(dest),
            dest_off,
        );
        Ok(())
    }

    pub fn encoder_debug(&mut self, encoder_rep: u32) -> Result<(), NativeGpuError> {
        let _ = self.resolve_encoder(encoder_rep)?;
        Ok(())
    }

    pub fn render_pass_end(&mut self, pass_rep: u32) -> Result<(), NativeGpuError> {
        let pass = self.resolve_render_pass(pass_rep)?;
        dawn_c::pass_end(self.dawn_of(pass));
        Ok(())
    }

    pub fn render_pass_draw(&mut self, pass_rep: u32) -> Result<(), NativeGpuError> {
        self.render_pass_draw_counts(pass_rep, 3, 1, 0, 0)
    }

    pub fn render_pass_draw_counts(
        &mut self,
        pass_rep: u32,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_render_pass(pass_rep)?;
        dawn_c::pass_draw(
            self.dawn_of(pass),
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        );
        Ok(())
    }

    pub fn render_pass_set_pipeline(
        &mut self,
        pass_rep: u32,
        pipeline_rep: u32,
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_render_pass(pass_rep)?;
        if pipeline_rep != 0 {
            if let Ok(pipeline) = GpuHandle::from_raw(pipeline_rep) {
                if self.get(pipeline, ResourceKind::RenderPipeline).is_ok() {
                    dawn_c::pass_set_pipeline(self.dawn_of(pass), self.dawn_of(pipeline));
                }
            }
        }
        Ok(())
    }

    pub fn render_pass_set_bind_group(
        &mut self,
        pass_rep: u32,
        index: u32,
        bind_group_rep: u32,
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_render_pass(pass_rep)?;
        let group = if bind_group_rep == 0 {
            0
        } else {
            let h = GpuHandle::from_raw(bind_group_rep)?;
            self.get(h, ResourceKind::BindGroup)?;
            self.dawn_of(h)
        };
        dawn_c::pass_set_bind_group(self.dawn_of(pass), index, group);
        Ok(())
    }

    pub fn render_pass_set_vertex_buffer(
        &mut self,
        pass_rep: u32,
        slot: u32,
        buffer_rep: u32,
        offset: u64,
        size: u64,
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_render_pass(pass_rep)?;
        let buffer = if buffer_rep == 0 {
            0
        } else {
            let h = self.resolve_buffer(buffer_rep)?;
            self.dawn_of(h)
        };
        dawn_c::pass_set_vertex_buffer(self.dawn_of(pass), slot, buffer, offset, size);
        Ok(())
    }

    pub fn compute_pass_end(&mut self, pass_rep: u32) -> Result<(), NativeGpuError> {
        let pass = self.resolve_compute_pass(pass_rep)?;
        dawn_c::compute_end(self.dawn_of(pass));
        Ok(())
    }

    pub fn compute_pass_dispatch(&mut self, pass_rep: u32) -> Result<(), NativeGpuError> {
        self.compute_pass_dispatch_xyz(pass_rep, 1, 1, 1)
    }

    pub fn compute_pass_dispatch_xyz(
        &mut self,
        pass_rep: u32,
        x: u32,
        y: u32,
        z: u32,
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_compute_pass(pass_rep)?;
        dawn_c::compute_dispatch(self.dawn_of(pass), x, y, z);
        Ok(())
    }

    pub fn compute_pass_set_pipeline(
        &mut self,
        pass_rep: u32,
        pipeline_rep: u32,
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_compute_pass(pass_rep)?;
        if pipeline_rep != 0 {
            let pipeline = self.resolve_compute_pipeline(pipeline_rep)?;
            dawn_c::compute_set_pipeline(self.dawn_of(pass), self.dawn_of(pipeline));
        }
        Ok(())
    }

    pub fn compute_pass_set_bind_group(
        &mut self,
        pass_rep: u32,
        index: u32,
        bind_group_rep: u32,
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_compute_pass(pass_rep)?;
        let group = if bind_group_rep == 0 {
            0
        } else {
            let h = GpuHandle::from_raw(bind_group_rep)?;
            self.get(h, ResourceKind::BindGroup)?;
            self.dawn_of(h)
        };
        dawn_c::compute_set_bind_group(self.dawn_of(pass), index, group);
        Ok(())
    }

    pub fn compute_pass_dispatch_indirect(
        &mut self,
        pass_rep: u32,
        buffer_rep: u32,
        offset: u64,
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_compute_pass(pass_rep)?;
        let buffer = self.resolve_buffer(buffer_rep)?;
        dawn_c::compute_dispatch_indirect(self.dawn_of(pass), self.dawn_of(buffer), offset);
        Ok(())
    }

    pub fn render_pass_set_viewport(
        &mut self,
        pass_rep: u32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_render_pass(pass_rep)?;
        dawn_c::pass_set_viewport(
            self.dawn_of(pass),
            x,
            y,
            width,
            height,
            min_depth,
            max_depth,
        );
        Ok(())
    }

    pub fn render_pass_set_scissor(
        &mut self,
        pass_rep: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_render_pass(pass_rep)?;
        dawn_c::pass_set_scissor(self.dawn_of(pass), x, y, width, height);
        Ok(())
    }

    pub fn render_pass_set_blend_constant(
        &mut self,
        pass_rep: u32,
        r: f64,
        g: f64,
        b: f64,
        a: f64,
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_render_pass(pass_rep)?;
        dawn_c::pass_set_blend_constant(self.dawn_of(pass), r, g, b, a);
        Ok(())
    }

    pub fn render_pass_set_stencil_reference(
        &mut self,
        pass_rep: u32,
        reference: u32,
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_render_pass(pass_rep)?;
        dawn_c::pass_set_stencil_reference(self.dawn_of(pass), reference);
        Ok(())
    }

    pub fn render_pass_set_index_buffer(
        &mut self,
        pass_rep: u32,
        buffer_rep: u32,
        format: u32,
        offset: u64,
        size: u64,
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_render_pass(pass_rep)?;
        let buffer = self.resolve_buffer(buffer_rep)?;
        dawn_c::pass_set_index_buffer(
            self.dawn_of(pass),
            self.dawn_of(buffer),
            format,
            offset,
            size,
        );
        Ok(())
    }

    pub fn render_pass_draw_indexed(
        &mut self,
        pass_rep: u32,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        base_vertex: i32,
        first_instance: u32,
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_render_pass(pass_rep)?;
        dawn_c::pass_draw_indexed(
            self.dawn_of(pass),
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
        );
        Ok(())
    }

    pub fn render_pass_draw_indirect(
        &mut self,
        pass_rep: u32,
        buffer_rep: u32,
        offset: u64,
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_render_pass(pass_rep)?;
        let buffer = self.resolve_buffer(buffer_rep)?;
        dawn_c::pass_draw_indirect(self.dawn_of(pass), self.dawn_of(buffer), offset);
        Ok(())
    }

    pub fn render_pass_draw_indexed_indirect(
        &mut self,
        pass_rep: u32,
        buffer_rep: u32,
        offset: u64,
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_render_pass(pass_rep)?;
        let buffer = self.resolve_buffer(buffer_rep)?;
        dawn_c::pass_draw_indexed_indirect(self.dawn_of(pass), self.dawn_of(buffer), offset);
        Ok(())
    }

    pub fn render_pass_execute_bundles(
        &mut self,
        pass_rep: u32,
        bundle_reps: &[u32],
    ) -> Result<(), NativeGpuError> {
        let pass = self.resolve_render_pass(pass_rep)?;
        let mut dawn = Vec::with_capacity(bundle_reps.len());
        for &rep in bundle_reps {
            if rep != 0 {
                let h = GpuHandle::from_raw(rep)?;
                self.get(h, ResourceKind::RenderBundle)?;
                dawn.push(self.dawn_of(h));
            }
        }
        dawn_c::pass_execute_bundles(self.dawn_of(pass), &dawn);
        Ok(())
    }

    pub fn resolve_queue(&mut self, queue_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if queue_rep == GpuHandle::NULL {
            let device = self.resolve_device(GpuHandle::NULL)?;
            self.device_queue(device)
        } else {
            let handle = GpuHandle::from_raw(queue_rep)?;
            self.get(handle, ResourceKind::Queue)?;
            Ok(handle)
        }
    }

    pub fn resolve_command_buffer(
        &mut self,
        command_rep: u32,
    ) -> Result<GpuHandle, NativeGpuError> {
        if command_rep == GpuHandle::NULL {
            let encoder = self.resolve_encoder(GpuHandle::NULL)?;
            self.encoder_finish(encoder, "")
        } else {
            let handle = GpuHandle::from_raw(command_rep)?;
            self.get(handle, ResourceKind::CommandBuffer)?;
            Ok(handle)
        }
    }

    pub fn queue_submit(
        &mut self,
        queue_rep: u32,
        command_reps: &[u32],
    ) -> Result<(), NativeGpuError> {
        let queue = self.resolve_queue(queue_rep)?;
        let mut resolved = Vec::with_capacity(command_reps.len());
        let mut dawn_cmds = Vec::with_capacity(command_reps.len());
        for &rep in command_reps {
            let cmd = self.resolve_command_buffer(rep)?;
            dawn_cmds.push(self.dawn_of(cmd));
            resolved.push(cmd.raw());
        }
        dawn_c::queue_submit(self.dawn_of(queue), &dawn_cmds);
        self.last_submit = resolved;
        // H8: submit may auto-present; second guest present is a no-op.
        let _ = self.canvas_present();
        self.mark_canvas_gpu_done();
        self.retire_canvas_frames();
        Ok(())
    }

    pub fn last_submit(&self) -> &[u32] {
        &self.last_submit
    }

    /// Guest `list<u8>` is already one copy off linear memory; store it (no JNI).
    pub fn write_buffer_with_copy(
        &mut self,
        queue_rep: u32,
        buffer_rep: u32,
        offset: u64,
        bytes: Vec<u8>,
    ) -> Result<(), NativeGpuError> {
        let queue = self.resolve_queue(queue_rep)?;
        let buffer = self.resolve_buffer(buffer_rep)?;
        dawn_c::write_buffer(self.dawn_of(queue), self.dawn_of(buffer), offset, &bytes);
        self.queue_writes.insert(
            queue.raw(),
            NativeQueueWrite {
                target: buffer.raw(),
                offset,
                bytes,
            },
        );
        Ok(())
    }

    pub fn write_texture_with_copy(
        &mut self,
        queue_rep: u32,
        texture_rep: u32,
        bytes: Vec<u8>,
    ) -> Result<(), NativeGpuError> {
        self.write_texture_described(queue_rep, texture_rep, bytes, 0, 1, 1, 1)
    }

    pub fn write_texture_described(
        &mut self,
        queue_rep: u32,
        texture_rep: u32,
        bytes: Vec<u8>,
        bytes_per_row: u32,
        width: u32,
        height: u32,
        depth: u32,
    ) -> Result<(), NativeGpuError> {
        let queue = self.resolve_queue(queue_rep)?;
        let texture = self.resolve_texture(texture_rep)?;
        dawn_c::write_texture(
            self.dawn_of(queue),
            self.dawn_of(texture),
            &bytes,
            bytes_per_row,
            width,
            height,
            depth,
        );
        self.queue_writes.insert(
            queue.raw(),
            NativeQueueWrite {
                target: texture.raw(),
                offset: 0,
                bytes,
            },
        );
        Ok(())
    }

    pub fn last_queue_write(&self, queue: GpuHandle) -> Option<&NativeQueueWrite> {
        self.queue_writes.get(&queue.raw())
    }

    pub fn on_submitted_work_done(&mut self, queue_rep: u32) -> Result<(), NativeGpuError> {
        let queue = self.resolve_queue(queue_rep)?;
        let instance = self.ensure_instance();
        dawn_c::work_done(instance, self.dawn_of(queue));
        self.mark_canvas_gpu_done();
        self.retire_canvas_frames();
        Ok(())
    }

    pub fn set_label(&mut self, rep: u32, label: String) {
        self.labels.insert(rep, label);
    }

    pub fn label(&self, rep: u32) -> String {
        self.labels.get(&rep).cloned().unwrap_or_default()
    }

    pub fn size64_add(&mut self, handle: u32, key: String, value: Option<u64>) {
        let rec = self.size64_records.entry(handle).or_default();
        if let Some(slot) = rec.iter_mut().find(|(k, _)| *k == key) {
            slot.1 = value;
        } else {
            rec.push((key, value));
        }
    }

    pub fn size64_get(&self, handle: u32, key: &str) -> Option<Option<u64>> {
        self.size64_records
            .get(&handle)
            .and_then(|rec| rec.iter().find(|(k, _)| k == key).map(|(_, v)| *v))
    }

    pub fn size64_has(&self, handle: u32, key: &str) -> bool {
        self.size64_get(handle, key).is_some()
    }

    pub fn size64_remove(&mut self, handle: u32, key: &str) {
        if let Some(rec) = self.size64_records.get_mut(&handle) {
            rec.retain(|(k, _)| k != key);
        }
    }

    pub fn size64_keys(&self, handle: u32) -> Vec<String> {
        self.size64_records
            .get(&handle)
            .map(|rec| rec.iter().map(|(k, _)| k.clone()).collect())
            .unwrap_or_default()
    }

    pub fn size64_values(&self, handle: u32) -> Vec<Option<u64>> {
        self.size64_records
            .get(&handle)
            .map(|rec| rec.iter().map(|(_, v)| *v).collect())
            .unwrap_or_default()
    }

    pub fn size64_entries(&self, handle: u32) -> Vec<(String, Option<u64>)> {
        self.size64_records
            .get(&handle)
            .cloned()
            .unwrap_or_default()
    }

    pub fn buffer_size(&mut self, buffer_rep: u32) -> Result<u64, NativeGpuError> {
        let handle = self.resolve_buffer(buffer_rep)?;
        Ok(self.buffers.get(&handle.raw()).map(|b| b.size).unwrap_or(0))
    }

    pub fn buffer_usage(&mut self, buffer_rep: u32) -> Result<u32, NativeGpuError> {
        let handle = self.resolve_buffer(buffer_rep)?;
        Ok(self
            .buffers
            .get(&handle.raw())
            .map(|b| b.usage)
            .unwrap_or(0))
    }

    pub fn buffer_mapped(&mut self, buffer_rep: u32) -> Result<bool, NativeGpuError> {
        let handle = self.resolve_buffer(buffer_rep)?;
        Ok(self
            .buffers
            .get(&handle.raw())
            .map(|b| b.mapped)
            .unwrap_or(false))
    }

    pub fn buffer_map_async(&mut self, buffer_rep: u32) -> Result<(), NativeGpuError> {
        self.buffer_map_async_range(buffer_rep, 1, 0, 0)
    }

    pub fn buffer_map_async_range(
        &mut self,
        buffer_rep: u32,
        mode: u32,
        offset: u64,
        size: u64,
    ) -> Result<(), NativeGpuError> {
        let handle = self.resolve_buffer(buffer_rep)?;
        let instance = self.ensure_instance();
        let _ = dawn_c::buffer_map_async(instance, self.dawn_of(handle), mode, offset, size);
        if let Some(buf) = self.buffers.get_mut(&handle.raw()) {
            buf.mapped = true;
            if buf.mapped_bytes.is_empty() {
                buf.mapped_bytes = vec![0; buf.size.min(4096) as usize];
            }
        }
        Ok(())
    }

    pub fn buffer_unmap(&mut self, buffer_rep: u32) -> Result<(), NativeGpuError> {
        let handle = self.resolve_buffer(buffer_rep)?;
        dawn_c::buffer_unmap(self.dawn_of(handle));
        if let Some(buf) = self.buffers.get_mut(&handle.raw()) {
            buf.mapped = false;
        }
        Ok(())
    }

    pub fn buffer_mapped_range(&mut self, buffer_rep: u32) -> Result<Vec<u8>, NativeGpuError> {
        let handle = self.resolve_buffer(buffer_rep)?;
        Ok(self
            .buffers
            .get(&handle.raw())
            .map(|b| b.mapped_bytes.clone())
            .unwrap_or_default())
    }

    pub fn buffer_set_mapped_range(
        &mut self,
        buffer_rep: u32,
        data: Vec<u8>,
    ) -> Result<(), NativeGpuError> {
        let handle = self.resolve_buffer(buffer_rep)?;
        if let Some(buf) = self.buffers.get_mut(&handle.raw()) {
            buf.mapped_bytes = data;
            buf.mapped = true;
        }
        Ok(())
    }

    pub fn texture_meta(&mut self, texture_rep: u32) -> Result<NativeTexture, NativeGpuError> {
        let handle = self.resolve_texture(texture_rep)?;
        Ok(self
            .textures
            .get(&handle.raw())
            .cloned()
            .unwrap_or(NativeTexture {
                width: 1,
                height: 1,
                depth: 1,
                format: 0,
                usage: 0,
                mip: 1,
                sample: 1,
                dimension: 2,
            }))
    }

    pub fn create_render_bundle_encoder(
        &mut self,
        device: GpuHandle,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let dawn = dawn_c::create_render_bundle_encoder(self.dawn_of(device), 0);
        Ok(self.table.insert(ResourceKind::RenderBundleEncoder, dawn))
    }

    pub fn resolve_render_bundle_encoder(
        &mut self,
        encoder_rep: u32,
    ) -> Result<GpuHandle, NativeGpuError> {
        if encoder_rep == GpuHandle::NULL {
            let device = self.resolve_device(GpuHandle::NULL)?;
            self.create_render_bundle_encoder(device)
        } else {
            let handle = GpuHandle::from_raw(encoder_rep)?;
            self.get(handle, ResourceKind::RenderBundleEncoder)?;
            Ok(handle)
        }
    }

    pub fn finish_render_bundle(&mut self, encoder_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        let enc = self.resolve_render_bundle_encoder(encoder_rep)?;
        let dawn = dawn_c::bundle_finish(self.dawn_of(enc));
        Ok(self.table.insert(ResourceKind::RenderBundle, dawn))
    }

    pub fn pipeline_bind_group_layout(
        &mut self,
        pipeline_rep: u32,
    ) -> Result<GpuHandle, NativeGpuError> {
        let dawn = if pipeline_rep != 0 {
            if let Ok(h) = GpuHandle::from_raw(pipeline_rep) {
                let compute = self.get(h, ResourceKind::ComputePipeline).is_ok();
                if compute || self.get(h, ResourceKind::RenderPipeline).is_ok() {
                    dawn_c::pipeline_bind_group_layout(self.dawn_of(h), compute, 0)
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        };
        if dawn != 0 {
            return Ok(self.table.insert(ResourceKind::BindGroupLayout, dawn));
        }
        let device = self.resolve_device(GpuHandle::NULL)?;
        self.create_bind_group_layout(device, &[], &[], &[], &[], &[])
    }

    pub fn bind_canvas_native_window(
        &mut self,
        native_window: i64,
        width: u32,
        height: u32,
    ) -> Result<(), NativeGpuError> {
        if native_window == 0 {
            return Err(NativeGpuError::InvalidHandle {
                handle: 0,
                message: "window-handle is null",
            });
        }
        if width == 0 || height == 0 {
            return Err(NativeGpuError::InvalidHandle {
                handle: 0,
                message: "invalid canvas size",
            });
        }
        self.canvas_window = Some(NativeCanvasWindow {
            native_window,
            width,
            height,
            buffer_count: SWAPCHAIN_BUFFER_COUNT,
            present_mode: NativePresentMode::Fifo,
        });
        self.hitch.last_desired_ns = 0;
        Ok(())
    }

    pub fn canvas_window(&self) -> Option<NativeCanvasWindow> {
        self.canvas_window
    }

    pub fn canvas_present_count(&self) -> u32 {
        self.present_count
    }

    pub fn canvas_ring_len(&self) -> usize {
        self.presented_ring.len()
    }

    pub fn canvas_pending(&self) -> Option<NativeCanvasFrame> {
        self.pending_present
    }

    fn mark_canvas_gpu_done(&mut self) {
        for frame in &mut self.presented_ring {
            frame.gpu_done = true;
        }
    }

    /// Recycle only GPU-done frames older than keep-3. Never close the just-presented image.
    fn retire_canvas_frames(&mut self) {
        while self.presented_ring.len() > CANVAS_FRAMES_TO_KEEP {
            let oldest = self.presented_ring.front().copied();
            match oldest {
                Some(frame) if frame.gpu_done => {
                    self.presented_ring.pop_front();
                    if let Ok(tex) = GpuHandle::from_raw(frame.texture) {
                        let slot = self.dawn_of(tex);
                        if slot != 0 {
                            dawn_c::release(ResourceKind::Texture, slot);
                            self.table.set_dawn(tex, 0);
                        }
                    }
                }
                _ => break,
            }
        }
    }

    fn discard_unpresented_canvas_frame(&mut self) {
        if let Some(frame) = self.pending_present.take() {
            if let Ok(tex) = GpuHandle::from_raw(frame.texture) {
                let slot = self.dawn_of(tex);
                if slot != 0 {
                    dawn_c::release(ResourceKind::Texture, slot);
                    self.table.set_dawn(tex, 0);
                }
            }
        }
    }

    fn ensure_dawn_surface(&mut self, device: GpuHandle) -> Option<GpuHandle> {
        let win = self.canvas_window?;
        let instance = self.ensure_instance();
        if instance == 0 {
            return Some(self.table.insert(ResourceKind::Surface, 0));
        }
        if self.dawn_surface == 0 {
            self.dawn_surface = dawn_c::create_surface(instance, win.native_window);
        }
        if self.dawn_surface_format == 0 {
            let adapter = self
                .table
                .handles_of_kind(ResourceKind::Adapter)
                .first()
                .copied()
                .and_then(|h| {
                    let slot = self.dawn_of(h);
                    if slot != 0 {
                        Some(slot)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            self.dawn_surface_format = dawn_c::surface_preferred_format(self.dawn_surface, adapter);
            // Align to D24 (androidx) which configures `caps.alphaModes[0]`,
            // not a hard-coded Auto.
            let (_formats, _present_modes, alpha_modes) =
                dawn_c::surface_caps_detail(self.dawn_surface, adapter);
            if let Some(a) = alpha_modes.first() {
                self.dawn_surface_alpha_mode = *a;
            }
        }
        let _ = device;
        Some(self.table.insert(ResourceKind::Surface, self.dawn_surface))
    }

    pub fn preferred_canvas_format(&mut self) -> u32 {
        if self.dawn_surface_format != 0 {
            return self.dawn_surface_format;
        }
        if let Some(device) = self
            .table
            .handles_of_kind(ResourceKind::Device)
            .first()
            .copied()
        {
            let _ = self.ensure_dawn_surface(device);
        } else if let Ok(device) = self.resolve_device(0) {
            let _ = self.ensure_dawn_surface(device);
        }
        if self.dawn_surface_format != 0 {
            self.dawn_surface_format
        } else {
            0x1B // BGRA8Unorm — table-backed leftover
        }
    }

    pub fn canvas_configure(
        &mut self,
        ctx_rep: u32,
        device_rep: u32,
        format: u32,
        usage: u32,
        color_space: i32,
        tone_mapping: i32,
        alpha_mode: i32,
        view_formats: &[i32],
    ) -> Result<GpuHandle, NativeGpuError> {
        let device = self.resolve_device(device_rep)?;
        let handle = if ctx_rep == GpuHandle::NULL {
            self.table.insert(ResourceKind::CanvasContext, 0)
        } else if let Ok(existing) = GpuHandle::from_raw(ctx_rep) {
            if self.table.contains(existing) {
                existing
            } else {
                self.table.insert(ResourceKind::CanvasContext, 0)
            }
        } else {
            self.table.insert(ResourceKind::CanvasContext, 0)
        };
        let surface = self.ensure_dawn_surface(device).map(|h| h.raw());
        if let (Some(win), Some(_surf)) = (self.canvas_window, surface) {
            let fmt = if format == 0 {
                self.dawn_surface_format
            } else {
                format
            };
            dawn_c::surface_configure(
                self.dawn_surface,
                self.dawn_of(device),
                fmt,
                win.width,
                win.height,
                // D24 (androidx) configures caps.alphaModes[0]; match it. Guest
                // alpha_mode is WIT opaque/premultiplied and is recorded but not
                // yet lowered to a WGPUCompositeAlphaMode (cube passes None).
                if self.dawn_surface_alpha_mode != 0 {
                    self.dawn_surface_alpha_mode
                } else {
                    dawn_c::ALPHA_AUTO
                },
            );
            self.dawn_surface_configured = self.dawn_surface != 0 && self.dawn_of(device) != 0;
            self.dawn_surface_format = fmt;
        }
        self.canvas_contexts.insert(
            handle.raw(),
            NativeCanvasContext {
                configured: true,
                device: device.raw(),
                format,
                usage,
                surface,
                color_space,
                tone_mapping,
                alpha_mode,
                view_formats: view_formats.to_vec(),
            },
        );
        Ok(handle)
    }

    pub fn canvas_unconfigure(&mut self, ctx_rep: u32) {
        self.discard_unpresented_canvas_frame();
        if let Some(state) = self.canvas_contexts.get_mut(&ctx_rep) {
            state.configured = false;
            state.surface = None;
        }
    }

    pub fn canvas_configuration(&self, ctx_rep: u32) -> Option<&NativeCanvasContext> {
        self.canvas_contexts.get(&ctx_rep).filter(|c| c.configured)
    }

    pub fn canvas_current_texture(&mut self, ctx_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        // H1: do not wait the previous GPU fence on acquire.
        self.discard_unpresented_canvas_frame();
        let device_rep = self
            .canvas_contexts
            .get(&ctx_rep)
            .map(|c| c.device)
            .unwrap_or(0);
        let device = self.resolve_device(device_rep)?;
        let (width, height, format, usage) = match self.canvas_contexts.get(&ctx_rep) {
            Some(c) if c.configured => {
                let (w, h) = self
                    .canvas_window
                    .map(|win| (win.width, win.height))
                    .unwrap_or((1, 1));
                (w, h, c.format, c.usage)
            }
            _ => (1, 1, 0, 0),
        };
        dawn_c::process_events(self.dawn_instance);
        let (dawn_tex, _status) = if self.dawn_surface_configured {
            dawn_c::surface_current_texture(self.dawn_surface)
        } else {
            (0, 0)
        };
        let (width, height) = if dawn_tex != 0 {
            dawn_c::texture_size(dawn_tex)
        } else {
            (width, height)
        };
        let texture =
            self.create_texture(device, width, height, 1, format, usage, 1, 1, 2, &[], "")?;
        if dawn_tex != 0 {
            self.table.set_dawn(texture, dawn_tex);
        }
        let surface = self
            .canvas_contexts
            .get(&ctx_rep)
            .and_then(|c| c.surface)
            .unwrap_or(0);
        self.pending_present = Some(NativeCanvasFrame {
            surface,
            texture: texture.raw(),
            gpu_done: false,
        });
        Ok(texture)
    }

    /// H8: idempotent. Second present with no pending acquire is a no-op.
    pub fn canvas_present(&mut self) -> bool {
        let Some(frame) = self.pending_present.take() else {
            return false;
        };
        self.stamp_desired_present();
        if self.dawn_surface != 0 {
            let _ = dawn_c::surface_present(self.dawn_surface);
        }
        // C2: do not close() the just-presented texture here.
        self.presented_ring.push_back(NativeCanvasFrame {
            gpu_done: false,
            ..frame
        });
        self.present_count = self.present_count.saturating_add(1);
        self.retire_canvas_frames();
        true
    }
}

impl NativeGpu for NativeGpuHost {
    fn insert(&mut self, kind: ResourceKind, dawn: DawnSlot) -> GpuHandle {
        self.table.insert(kind, dawn)
    }

    fn contains(&self, handle: GpuHandle) -> bool {
        self.table.contains(handle)
    }

    fn get(&self, handle: GpuHandle, kind: ResourceKind) -> Result<&HandleEntry, NativeGpuError> {
        self.table.get(handle, kind)
    }

    fn drop_handle(&mut self, handle: GpuHandle) -> Result<HandleEntry, NativeGpuError> {
        let entry = self.table.drop_handle(handle)?;
        self.release_dawn_entry(handle, entry);
        self.forget_side(handle, entry.kind);
        Ok(entry)
    }

    fn try_drop(&mut self, handle: GpuHandle) -> Option<HandleEntry> {
        let entry = self.table.try_drop(handle)?;
        self.release_dawn_entry(handle, entry);
        self.forget_side(handle, entry.kind);
        Some(entry)
    }

    fn handles_of_kind(&self, kind: ResourceKind) -> Vec<GpuHandle> {
        self.table.handles_of_kind(kind)
    }

    fn size(&self) -> usize {
        self.table.size()
    }

    fn clear(&mut self) {
        self.table.clear();
        self.interned_queues.clear();
        self.adapter_info.clear();
        self.shader_hints.clear();
        self.pipeline_constant_records.clear();
        self.pipeline_constants.clear();
        self.query_sets.clear();
        self.queue_writes.clear();
        self.last_submit.clear();
        self.size64_records.clear();
        self.labels.clear();
        self.buffers.clear();
        self.textures.clear();
        self.canvas_window = None;
        self.canvas_contexts.clear();
        self.pending_present = None;
        self.presented_ring.clear();
        self.present_count = 0;
    }

    fn request_adapter(&mut self, options: &NativeRequestAdapterOptions<'_>) -> Option<GpuHandle> {
        NativeGpuHost::request_adapter(self, options)
    }

    fn request_device(
        &mut self,
        adapter: GpuHandle,
        desc: &NativeRequestDeviceDescriptor<'_>,
    ) -> Result<GpuHandle, NativeGpuError> {
        NativeGpuHost::request_device(self, adapter, desc)
    }

    fn device_queue(&mut self, device: GpuHandle) -> Result<GpuHandle, NativeGpuError> {
        NativeGpuHost::device_queue(self, device)
    }

    fn adapter_info(&self, adapter: GpuHandle) -> Result<NativeAdapterInfo, NativeGpuError> {
        NativeGpuHost::adapter_info(self, adapter)
    }

    fn adapter_has_feature(&self, adapter: GpuHandle, name: &str) -> Result<bool, NativeGpuError> {
        NativeGpuHost::adapter_has_feature(self, adapter, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_no_jni_in_this_module() {
        // Compile-time: this file has no `use jni`. Runtime smoke: insert/drop
        // never constructs a GlobalRef / JVM attach.
        let _gpu: NativeGpuHost = NativeGpuHost::new();
    }

    #[test]
    fn try_load_dawn_c_is_best_effort_no_jni() {
        // Host/Cloud: Android `.so` is absent; must not panic or JNI.
        assert!(!NativeGpuHost::try_load_dawn_c());
    }

    #[test]
    fn insert_drop_does_not_jni() {
        assert_no_jni_in_this_module();
        let mut gpu: NativeGpuHost = NativeGpuHost::new();
        let h = NativeGpu::insert(&mut gpu, ResourceKind::Buffer, 0);
        assert_ne!(h.raw(), GpuHandle::NULL);
        assert!(gpu.contains(h));
        assert_eq!(gpu.size(), 1);
        let entry = NativeGpu::drop_handle(&mut gpu, h).expect("drop live handle");
        assert_eq!(entry.kind, ResourceKind::Buffer);
        assert_eq!(entry.dawn, 0);
        assert_eq!(gpu.size(), 0);
        assert!(NativeGpu::try_drop(&mut gpu, h).is_none());
    }

    #[test]
    fn handle_zero_reserved() {
        assert!(GpuHandle::from_raw(0).is_err());
        let mut table = HandleTable::new();
        let h = table.insert(ResourceKind::Adapter, 0);
        assert_eq!(h.raw(), 1);
        assert_ne!(h.raw(), GpuHandle::NULL);
    }

    #[test]
    fn kind_mismatch_and_unknown() {
        let mut gpu = NativeGpuHost::new();
        let h = gpu.insert(ResourceKind::Device, 0);
        let err = gpu.get(h, ResourceKind::Queue).unwrap_err();
        assert!(matches!(
            err,
            NativeGpuError::KindMismatch {
                expected: ResourceKind::Queue,
                found: ResourceKind::Device,
                ..
            }
        ));
        gpu.drop_handle(h).unwrap();
        assert!(matches!(
            gpu.drop_handle(h),
            Err(NativeGpuError::InvalidHandle { .. })
        ));
    }

    #[test]
    fn handles_of_kind_and_clear() {
        let mut gpu = NativeGpuHost::new();
        let a = gpu.insert(ResourceKind::Adapter, 0);
        let _d = gpu.insert(ResourceKind::Device, 0);
        let adapters = gpu.handles_of_kind(ResourceKind::Adapter);
        assert_eq!(adapters, vec![a]);
        gpu.clear();
        assert_eq!(gpu.size(), 0);
        assert!(gpu.handles_of_kind(ResourceKind::Adapter).is_empty());
    }

    #[test]
    fn all_kotlin_resource_kinds_exist() {
        // Keep in lockstep with Handles.kt ResourceKind.
        let kinds = [
            ResourceKind::Adapter,
            ResourceKind::Device,
            ResourceKind::Buffer,
            ResourceKind::ShaderModule,
            ResourceKind::BindGroupLayout,
            ResourceKind::BindGroup,
            ResourceKind::PipelineLayout,
            ResourceKind::ComputePipeline,
            ResourceKind::CommandEncoder,
            ResourceKind::ComputePassEncoder,
            ResourceKind::CommandBuffer,
            ResourceKind::Queue,
            ResourceKind::Surface,
            ResourceKind::CanvasContext,
            ResourceKind::Texture,
            ResourceKind::TextureView,
            ResourceKind::Sampler,
            ResourceKind::RenderPipeline,
            ResourceKind::RenderPassEncoder,
            ResourceKind::QuerySet,
            ResourceKind::RenderBundleEncoder,
            ResourceKind::RenderBundle,
        ];
        assert_eq!(kinds.len(), 22);
        let mut gpu = NativeGpuHost::new();
        for kind in kinds {
            let _ = gpu.insert(kind, 0);
        }
        assert_eq!(gpu.size(), 22);
    }

    #[test]
    fn request_adapter_device_queue_boot_no_jni() {
        let mut gpu = NativeGpuHost::new();
        let adapter = gpu
            .request_adapter(&NativeRequestAdapterOptions {
                xr_compatible: Some(true),
                ..Default::default()
            })
            .expect("table-backed adapter");
        assert_ne!(adapter.raw(), GpuHandle::NULL);
        let device = gpu
            .request_device(
                adapter,
                &NativeRequestDeviceDescriptor {
                    label: "l2",
                    default_queue_label: "l2",
                    required_features: &[0, 1],
                    required_limits_rep: 0,
                },
            )
            .expect("table-backed device");
        let q1 = gpu.device_queue(device).expect("queue");
        let q2 = gpu.device_queue(device).expect("interned queue");
        assert_eq!(q1, q2, "device.queue interned like DawnWasiWebGpuHost");
        let info = gpu.adapter_info(adapter).expect("info");
        assert_eq!(info.device, "native-gpu");
        assert!(!info.is_fallback_adapter);
        assert!(!gpu.adapter_has_feature(adapter, "timestamp-query").unwrap());
        let via_zero = gpu.resolve_device(0).expect("fixture get-device");
        assert_ne!(via_zero.raw(), GpuHandle::NULL);
    }

    #[test]
    fn handle_table_default_skips_null() {
        let mut table = HandleTable::default();
        let h = table.insert(ResourceKind::Queue, 0);
        assert_ne!(h.raw(), GpuHandle::NULL);
    }

    #[test]
    fn create_resources_and_shader_hints_record_no_jni() {
        let mut gpu = NativeGpuHost::new();
        let device = gpu.resolve_device(0).expect("boot device");
        let buf = gpu.create_buffer(device, 4, 0x28, 1, "l2").expect("buffer");
        let tex = gpu
            .create_texture(device, 1, 1, 1, 0, 0, 2, 1, 2, &[1], "l2")
            .expect("texture");
        let samp = gpu
            .create_sampler(device, 0, 0, 0, 0, 0, 0, 0, 0, 0.0, 0, 0.0)
            .expect("sampler");
        let shader = gpu
            .create_shader_module(device, "fn l2() {}", "l2", &[-1], "l2")
            .expect("shader");
        let view = gpu
            .create_texture_view(tex, 0, 0, 0, 0, 1, 0, 1)
            .expect("view");
        assert_ne!(buf.raw(), GpuHandle::NULL);
        assert_ne!(samp.raw(), GpuHandle::NULL);
        assert_ne!(view.raw(), GpuHandle::NULL);
        let entry = gpu.get(shader, ResourceKind::ShaderModule).unwrap();
        assert_eq!(entry.dawn, 0, "compilation-hints stay Record, not Dawn C");
        let hints = gpu
            .shader_compilation_hints(shader)
            .unwrap()
            .expect("hints recorded");
        assert_eq!(hints.entries, "l2");
        assert_eq!(hints.layouts, vec![-1]);
    }

    #[test]
    fn create_layouts_pipelines_and_constants_no_jni() {
        let mut gpu = NativeGpuHost::new();
        let device = gpu.resolve_device(0).expect("boot device");
        let bgl = gpu
            .create_bind_group_layout(device, &[0], &[4], &[0], &[-1], &[-1])
            .expect("bgl");
        let pl = gpu
            .create_pipeline_layout(device, &[bgl.raw() as i32], "l2")
            .expect("pipeline-layout");
        let bg = gpu
            .create_bind_group(device, bgl, "l2", &[0], &[0], &[0])
            .expect("bind-group");
        gpu.pipeline_constant_add(7, "c".into(), 1.0);
        assert!(gpu.pipeline_constant_has(7, "c"));
        assert_eq!(gpu.pipeline_constant_get(7, "c"), Some(1.0));
        let shader = gpu
            .create_shader_module(device, "fn main() {}", "", &[], "")
            .expect("shader");
        let compute = gpu
            .create_compute_pipeline(device, shader.raw(), "main", 0, "l2", 7)
            .expect("compute");
        let render = gpu
            .create_render_pipeline(device, shader.raw(), "vs_main", 0, "", 0, 0, "l2", 0, 0)
            .expect("render");
        assert_ne!(pl.raw(), GpuHandle::NULL);
        assert_ne!(bg.raw(), GpuHandle::NULL);
        let constants = gpu
            .pipeline_constants(compute)
            .unwrap()
            .expect("constants copied");
        assert_eq!(constants.compute, vec![("c".into(), 1.0)]);
        assert_eq!(
            gpu.get(render, ResourceKind::RenderPipeline).unwrap().dawn,
            0
        );
    }

    #[test]
    fn create_encoder_passes_draws_copies_no_jni() {
        let mut gpu = NativeGpuHost::new();
        let device = gpu.resolve_device(0).expect("boot device");
        let encoder = gpu.create_command_encoder(device, "l2").expect("encoder");
        let query = gpu.create_query_set(device, 0, 1).expect("query");
        let pass = gpu
            .begin_render_pass(encoder, &[0], 0)
            .expect("render-pass");
        gpu.render_pass_draw(pass.raw()).expect("draw");
        gpu.render_pass_end(pass.raw()).expect("end");
        let compute = gpu
            .begin_compute_pass(encoder, query.raw(), 0, 1)
            .expect("compute-pass");
        gpu.compute_pass_dispatch(compute.raw()).expect("dispatch");
        gpu.compute_pass_end(compute.raw()).expect("compute-end");
        gpu.encoder_copy(encoder.raw(), Some(0), Some(0), None, None)
            .expect("copy");
        gpu.encoder_debug(encoder.raw()).expect("debug");
        let buf = gpu.encoder_finish(encoder, "l2").expect("finish");
        assert_ne!(encoder.raw(), GpuHandle::NULL);
        assert_ne!(pass.raw(), GpuHandle::NULL);
        assert_ne!(buf.raw(), GpuHandle::NULL);
        assert_eq!(gpu.query_set_type(query).unwrap(), 0);
        assert_eq!(gpu.get(buf, ResourceKind::CommandBuffer).unwrap().dawn, 0);
    }

    #[test]
    fn queue_submit_write_one_copy_no_jni() {
        let mut gpu = NativeGpuHost::new();
        let device = gpu.resolve_device(0).expect("boot device");
        let queue = gpu.device_queue(device).expect("queue");
        let buffer = gpu.create_buffer(device, 4, 0x8, -1, "").expect("buffer");
        gpu.write_buffer_with_copy(queue.raw(), buffer.raw(), 0, b"l2\0\0".to_vec())
            .expect("write-buffer");
        let write = gpu.last_queue_write(queue).expect("one copy stored");
        assert_eq!(write.bytes, b"l2\0\0");
        assert_eq!(write.offset, 0);
        gpu.write_texture_with_copy(queue.raw(), 0, b"l2\0\0".to_vec())
            .expect("write-texture");
        gpu.queue_submit(queue.raw(), &[0]).expect("submit");
        assert_eq!(gpu.last_submit().len(), 1);
        gpu.on_submitted_work_done(queue.raw()).expect("work-done");
        assert_eq!(gpu.get(queue, ResourceKind::Queue).unwrap().dawn, 0);
    }

    #[test]
    fn rest_labels_size64_buffer_bundle_no_jni() {
        let mut gpu = NativeGpuHost::new();
        let device = gpu.resolve_device(0).expect("boot device");
        gpu.set_label(device.raw(), "l2".into());
        assert_eq!(gpu.label(device.raw()), "l2");
        gpu.size64_add(3, "w".into(), Some(4));
        assert!(gpu.size64_has(3, "w"));
        assert_eq!(gpu.size64_get(3, "w"), Some(Some(4)));
        assert_eq!(gpu.size64_keys(3), vec!["w".to_string()]);
        let buf = gpu.create_buffer(device, 8, 0x4, 1, "buf").expect("buffer");
        assert_eq!(gpu.buffer_size(buf.raw()).unwrap(), 8);
        assert!(gpu.buffer_mapped(buf.raw()).unwrap());
        gpu.buffer_unmap(buf.raw()).unwrap();
        assert!(!gpu.buffer_mapped(buf.raw()).unwrap());
        let enc = gpu
            .create_render_bundle_encoder(device)
            .expect("bundle-encoder");
        let bundle = gpu.finish_render_bundle(enc.raw()).expect("bundle");
        assert_eq!(gpu.get(bundle, ResourceKind::RenderBundle).unwrap().dawn, 0);
        let tex = gpu.canvas_current_texture(0).expect("canvas texture");
        assert_eq!(gpu.texture_meta(tex.raw()).unwrap().width, 1);
    }

    #[test]
    fn canvas_window_hitch_invariants_no_jni() {
        let mut gpu = NativeGpuHost::new();
        assert!(gpu.bind_canvas_native_window(0, 64, 64).is_err());
        gpu.bind_canvas_native_window(0x1000, 64, 48)
            .expect("bind window");
        let win = gpu.canvas_window().expect("window stored");
        assert_eq!(win.buffer_count, SWAPCHAIN_BUFFER_COUNT);
        assert_eq!(win.present_mode, NativePresentMode::Fifo);
        assert_eq!(win.width, 64);
        let device = gpu.resolve_device(0).expect("device");
        let ctx = gpu
            .canvas_configure(0, device.raw(), 0x16, 0x10, -1, -1, -1, &[])
            .expect("configure");
        let cfg = gpu.canvas_configuration(ctx.raw()).expect("configured");
        assert!(cfg.surface.is_some(), "windowed configure inserts Surface");
        let tex = gpu.canvas_current_texture(ctx.raw()).expect("acquire");
        assert_eq!(gpu.texture_meta(tex.raw()).unwrap().width, 64);
        assert!(gpu.canvas_pending().is_some());
        assert!(gpu.canvas_present(), "first present");
        assert!(!gpu.canvas_present(), "H8 second present is no-op");
        assert_eq!(gpu.canvas_present_count(), 1);
        assert_eq!(gpu.canvas_ring_len(), 1);
        for _ in 0..5 {
            let _ = gpu.canvas_current_texture(ctx.raw()).expect("acquire");
            assert!(gpu.canvas_present());
        }
        let queue = gpu.device_queue(device).expect("interned queue");
        gpu.queue_submit(queue.raw(), &[]).expect("submit");
        assert!(
            gpu.canvas_ring_len() <= CANVAS_FRAMES_TO_KEEP,
            "keep-3 after GPU done, ring={}",
            gpu.canvas_ring_len()
        );
        gpu.canvas_unconfigure(ctx.raw());
        assert!(gpu.canvas_configuration(ctx.raw()).is_none());
    }

    #[test]
    fn desired_present_ns_beats_and_monotonic() {
        assert_eq!(desired_present_ns(0, 8_333_333, 2, 0), None);
        assert_eq!(desired_present_ns(1_000, 100, 0, 0), None);
        assert_eq!(desired_present_ns(1_000, 100, 2, 0), Some(1_200));
        // Cadence must not go backwards if vsync is late relative to last stamp.
        assert_eq!(desired_present_ns(1_000, 100, 2, 1_200), Some(1_300));
        assert_eq!(desired_present_ns(1_200, 100, 2, 1_200), Some(1_400));
    }

    #[test]
    fn canvas_present_stamps_desired_from_vsync() {
        let mut gpu = NativeGpuHost::new();
        gpu.bind_canvas_native_window(0x1000, 64, 48)
            .expect("bind window");
        let device = gpu.resolve_device(0).expect("device");
        let ctx = gpu
            .canvas_configure(0, device.raw(), 0x16, 0x10, -1, -1, -1, &[])
            .expect("configure");
        gpu.note_consumed_vsync(1_000_000_000);
        let _ = gpu.canvas_current_texture(ctx.raw()).expect("acquire");
        assert!(gpu.canvas_present());
        let beats = hitch_desired_present_beats();
        let expect = desired_present_ns(1_000_000_000, 8_333_333, beats, 0)
            .map(|t| t as u64)
            .unwrap_or(0);
        assert_eq!(gpu.hitch.last_desired_ns, expect);
        gpu.note_consumed_vsync(1_008_333_333);
        let _ = gpu.canvas_current_texture(ctx.raw()).expect("acquire");
        assert!(gpu.canvas_present());
        let expect2 = desired_present_ns(1_008_333_333, 8_333_333, beats, expect)
            .map(|t| t as u64)
            .unwrap_or(0);
        assert_eq!(gpu.hitch.last_desired_ns, expect2);
        if beats >= 1 {
            assert!(gpu.hitch.last_desired_ns > expect);
        }
    }

    #[test]
    fn vsync_present_beats_are_1_to_1() {
        use crate::host::{GfxOnFrameGate, GfxOnFrameTake};
        use std::thread;
        use std::time::Duration;

        let gate = GfxOnFrameGate::new();
        let mut gpu = NativeGpuHost::new();
        gpu.bind_canvas_native_window(0x1000, 64, 48)
            .expect("bind window");
        let device = gpu.resolve_device(0).expect("device");
        let ctx = gpu
            .canvas_configure(0, device.raw(), 0x16, 0x10, -1, -1, -1, &[])
            .expect("configure");
        let queue = gpu.device_queue(device).expect("queue");
        let buffer = gpu.create_buffer(device, 4, 0x8, -1, "").expect("buffer");

        const PERIOD: u64 = 8_333_333;
        const T0: u64 = 1_000_000_000;
        const BEATS: u32 = 12;

        for i in 0..BEATS {
            let ts = (T0 + u64::from(i) * PERIOD) as i64;
            if i == 0 {
                let waiter = gate.clone();
                let take = thread::spawn(move || waiter.wait_take(false));
                thread::sleep(Duration::from_millis(20));
                gate.post(ts);
                take.join().expect("first take");
            } else {
                gate.post(ts);
                assert!(
                    matches!(gate.wait_take(false), GfxOnFrameTake::Item),
                    "beat {i} must take without waiting an extra vsync"
                );
            }
            assert_eq!(
                gate.last_take_vsync_ns(),
                ts as u64,
                "gate consumed Choreographer ts beat {i}"
            );
            gpu.note_consumed_vsync(gate.last_take_vsync_ns());
            let _ = gpu.canvas_current_texture(ctx.raw()).expect("acquire");
            gpu.write_buffer_with_copy(queue.raw(), buffer.raw(), 0, b"l2\0\0".to_vec())
                .expect("write");
            gpu.queue_submit(queue.raw(), &[]).expect("submit");
            assert!(
                !gpu.canvas_present(),
                "H8 no-op after auto-present beat {i}"
            );
            assert_eq!(gpu.canvas_present_count(), i + 1);
        }

        assert!(
            gpu.canvas_ring_len() <= CANVAS_FRAMES_TO_KEEP,
            "keep-3, ring={}",
            gpu.canvas_ring_len()
        );

        let stamp_beats = hitch_desired_present_beats();
        if stamp_beats >= 1 {
            let last_vsync = T0 + u64::from(BEATS - 1) * PERIOD;
            let expect = last_vsync.saturating_add(PERIOD.saturating_mul(stamp_beats as u64));
            assert_eq!(
                gpu.hitch.last_desired_ns, expect,
                "desired-present stays 1:1 with vsync + {stamp_beats} beats"
            );
        }
    }
}
