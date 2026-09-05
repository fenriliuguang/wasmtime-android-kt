//! Component Model JNI (M1 sync + M2 concurrent/async + P3 stream).

use crate::engine::new_engine;
use crate::error::{throw, throw_compile, throw_err, throw_link};
use crate::gpu_dispatch::GpuBackend;
use crate::handles::{drop_handle, from_handle, to_handle};
use crate::host::{
    gfx_input_lookup, gfx_input_register, gfx_input_unregister, gfx_on_frame_lookup,
    gfx_on_frame_register, gfx_on_frame_unregister, wasi_monotonic_now_ns, GfxInputTake,
    GfxKeyGate, GfxKeySample, GfxOnFrameGate, GfxOnFrameTake, GfxOnResizeGate, GfxOnResizeTake,
    GfxPointerGate, Gpu, GpuAdapter, GpuBindGroup, GpuBindGroupLayout, GpuBuffer, GpuCommandBuffer,
    GpuCommandEncoder, GpuComputePassEncoder, GpuComputePipeline, GpuDevice, GpuPipelineLayout,
    GpuQuerySet, GpuQueue, GpuRenderBundle, GpuRenderBundleEncoder, GpuRenderPassEncoder,
    GpuRenderPipeline, GpuSampler, GpuShaderModule, GpuTexture, GpuTextureView, HostState, Widget,
};
use crate::jvm;
use crate::native_gpu::{NativeRequestAdapterOptions, NativeRequestDeviceDescriptor};
use crate::webgpu_abi::{
    CreatePipelineError, CreatePipelineErrorKind, CreateQuerySetError, GetMappedRangeError,
    GpuAdapterInfo, GpuBindGroupDescriptor, GpuBindGroupLayoutDescriptor, GpuBindingResource,
    GpuBlendFactor, GpuBlendOperation, GpuBufferBindingType, GpuBufferDescriptor,
    GpuBufferMapState, GpuBufferUsage, GpuCanvasConfiguration, GpuCanvasConfigurationOwned,
    GpuCanvasContext, GpuColor, GpuColorWrite, GpuCommandBufferDescriptor,
    GpuCommandEncoderDescriptor, GpuCompareFunction, GpuCompilationInfo, GpuCompilationMessage,
    GpuCompilationMessageType, GpuComputePassDescriptor, GpuComputePipelineDescriptor, GpuCullMode,
    GpuDeviceDescriptor, GpuDeviceLostInfo, GpuDeviceLostReason, GpuError, GpuErrorFilter,
    GpuErrorKind, GpuExtent3D, GpuFrontFace, GpuIndexFormat, GpuLayoutMode, GpuMapMode,
    GpuMipmapFilterMode, GpuPipelineErrorReason, GpuPipelineLayoutDescriptor, GpuPrimitiveTopology,
    GpuQuerySetDescriptor, GpuQueryType, GpuRenderBundleDescriptor,
    GpuRenderBundleEncoderDescriptor, GpuRenderPassDescriptor, GpuRenderPipelineDescriptor,
    GpuRequestAdapterOptions, GpuSamplerBindingType, GpuSamplerDescriptor,
    GpuShaderModuleDescriptor, GpuShaderStage, GpuStencilFaceState, GpuStencilOperation,
    GpuSupportedFeatures, GpuSupportedLimits, GpuTexelCopyBufferInfo, GpuTexelCopyBufferLayout,
    GpuTexelCopyTextureInfo, GpuTextureDescriptor, GpuTextureDimension, GpuTextureFormat,
    GpuTextureSampleType, GpuTextureUsage, GpuTextureViewDescriptor, GpuTextureViewDimension,
    GpuUncapturedErrorEvent, GpuVertexFormat, GpuVertexStepMode, MapAsyncError, PopErrorScopeError,
    RecordGpuPipelineConstantValue, RecordOptionGpuSize64, RequestDeviceError,
    RequestDeviceErrorKind, SetBindGroupError, UnmapError, WgslLanguageFeatures, WriteBufferError,
};
use futures::channel::oneshot;
use jni::objects::{JByteArray, JClass, JObject, JString};
use jni::sys::{jboolean, jdouble, jint, jlong};
use jni::JNIEnv;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use wasmtime::component::{
    Component, ComponentType, Destination, FutureReader, Lift, Linker, Lower, Resource,
    ResourceType, Source, StreamConsumer, StreamProducer, StreamReader, StreamResult,
};
use wasmtime::{Engine, Store, StoreContextMut};

type HostStore = Store<HostState>;

/// P3: pack guest `vertex.buffers` into parallel JNI int arrays (Dawn step/format values).
fn pack_vertex_buffers(
    buffers: &Option<Vec<Option<crate::webgpu_abi::GpuVertexBufferLayout>>>,
) -> (Vec<i32>, Vec<i32>, Vec<i32>, Vec<i32>, Vec<i32>, Vec<i32>) {
    let mut strides = Vec::new();
    let mut step_modes = Vec::new();
    let mut attr_index = Vec::new();
    let mut attr_formats = Vec::new();
    let mut attr_offsets = Vec::new();
    let mut attr_locations = Vec::new();
    let Some(slots) = buffers else {
        return (
            strides,
            step_modes,
            attr_index,
            attr_formats,
            attr_offsets,
            attr_locations,
        );
    };
    for slot in slots {
        let Some(layout) = slot else {
            continue;
        };
        let buf_i = strides.len() as i32;
        strides.push(layout.array_stride as i32);
        step_modes.push(match layout.step_mode {
            Some(GpuVertexStepMode::Instance) => 2,
            Some(GpuVertexStepMode::Vertex) | None => 1,
        });
        for attr in &layout.attributes {
            attr_index.push(buf_i);
            attr_formats.push(match attr.format {
                GpuVertexFormat::Float32x2 => 0x1d,
                GpuVertexFormat::Float32x3 => 0x1e,
                GpuVertexFormat::Float32x4 => 0x1f,
                GpuVertexFormat::Float32 => 0x1c,
                other => other as i32,
            });
            attr_offsets.push(attr.offset as i32);
            attr_locations.push(attr.shader_location as i32);
        }
    }
    (
        strides,
        step_modes,
        attr_index,
        attr_formats,
        attr_offsets,
        attr_locations,
    )
}

fn first_fragment_target_format(fragment: &Option<crate::webgpu_abi::GpuFragmentState>) -> i32 {
    fragment
        .as_ref()
        .and_then(|fs| fs.targets.iter().flatten().next())
        .map(|target| target.format.to_dawn_u32() as i32)
        .unwrap_or(0)
}

fn dawn_topology(t: GpuPrimitiveTopology) -> i32 {
    match t {
        GpuPrimitiveTopology::PointList => 1,
        GpuPrimitiveTopology::LineList => 2,
        GpuPrimitiveTopology::LineStrip => 3,
        GpuPrimitiveTopology::TriangleList => 4,
        GpuPrimitiveTopology::TriangleStrip => 5,
    }
}

fn dawn_cull(c: GpuCullMode) -> i32 {
    match c {
        GpuCullMode::None => 1,
        GpuCullMode::Front => 2,
        GpuCullMode::Back => 3,
    }
}

fn dawn_front_face(f: GpuFrontFace) -> i32 {
    match f {
        GpuFrontFace::Ccw => 1,
        GpuFrontFace::Cw => 2,
    }
}

fn dawn_index_format(f: GpuIndexFormat) -> i32 {
    match f {
        GpuIndexFormat::Uint16 => 1,
        GpuIndexFormat::Uint32 => 2,
    }
}

fn dawn_blend_op(op: GpuBlendOperation) -> i32 {
    match op {
        GpuBlendOperation::Add => 1,
        GpuBlendOperation::Subtract => 2,
        GpuBlendOperation::ReverseSubtract => 3,
        GpuBlendOperation::Min => 4,
        GpuBlendOperation::Max => 5,
    }
}

fn dawn_blend_factor(f: GpuBlendFactor) -> i32 {
    (f as i32) + 1
}

fn dawn_compare(c: GpuCompareFunction) -> i32 {
    (c as i32) + 1
}

fn dawn_stencil_op(op: GpuStencilOperation) -> i32 {
    (op as i32) + 1
}

fn pack_stencil_face(face: &Option<GpuStencilFaceState>) -> [i32; 5] {
    match face {
        None => [0, 0, 0, 0, 0],
        Some(f) => [
            1,
            f.compare.map(dawn_compare).unwrap_or(0),
            f.fail_op.map(dawn_stencil_op).unwrap_or(0),
            f.depth_fail_op.map(dawn_stencil_op).unwrap_or(0),
            f.pass_op.map(dawn_stencil_op).unwrap_or(0),
        ],
    }
}

/// Empty vec = absent. Packed ints: format, depth-write, compare, front/back 5-tuples,
/// then (has, value) for read-mask / write-mask / bias / slope-bits / clamp-bits.
fn pack_depth_stencil(depth_stencil: &Option<crate::webgpu_abi::GpuDepthStencilState>) -> Vec<i32> {
    let Some(ds) = depth_stencil else {
        return Vec::new();
    };
    let mut v = Vec::with_capacity(23);
    v.push((ds.format as i32) + 1);
    v.push(match ds.depth_write_enabled {
        Some(true) => 1,
        Some(false) => 0,
        None => -1,
    });
    v.push(ds.depth_compare.map(dawn_compare).unwrap_or(0));
    v.extend_from_slice(&pack_stencil_face(&ds.stencil_front));
    v.extend_from_slice(&pack_stencil_face(&ds.stencil_back));
    match ds.stencil_read_mask {
        Some(m) => {
            v.push(1);
            v.push(m as i32);
        }
        None => v.extend_from_slice(&[0, 0]),
    }
    match ds.stencil_write_mask {
        Some(m) => {
            v.push(1);
            v.push(m as i32);
        }
        None => v.extend_from_slice(&[0, 0]),
    }
    match ds.depth_bias {
        Some(b) => {
            v.push(1);
            v.push(b);
        }
        None => v.extend_from_slice(&[0, 0]),
    }
    match ds.depth_bias_slope_scale {
        Some(s) => {
            v.push(1);
            v.push(s.to_bits() as i32);
        }
        None => v.extend_from_slice(&[0, 0]),
    }
    match ds.depth_bias_clamp {
        Some(c) => {
            v.push(1);
            v.push(c.to_bits() as i32);
        }
        None => v.extend_from_slice(&[0, 0]),
    }
    v
}

/// WIT `gpu-color-write` bits as i32; `-1` = absent (Dawn All). `0` = explicit none.
fn pack_color_write(mask: Option<GpuColorWrite>) -> i32 {
    let Some(m) = mask else {
        return -1;
    };
    let mut bits = 0i32;
    if m.contains(GpuColorWrite::RED) {
        bits |= 1 << 0;
    }
    if m.contains(GpuColorWrite::GREEN) {
        bits |= 1 << 1;
    }
    if m.contains(GpuColorWrite::BLUE) {
        bits |= 1 << 2;
    }
    if m.contains(GpuColorWrite::ALPHA) {
        bits |= 1 << 3;
    }
    if m.contains(GpuColorWrite::ALL) {
        bits |= 1 << 4;
    }
    bits
}

/// F1: primitive (topology/strip/front/cull) + multisample + per-target blend 7-tuples
/// + per-target write-mask (`-1` absent) + depth-stencil leftovers.
fn pack_render_pipeline_semantics(
    descriptor: &GpuRenderPipelineDescriptor,
) -> (Vec<i32>, Vec<i32>, Vec<i32>, Vec<i32>, Vec<i32>) {
    let mut primitive = vec![0, 0, 0, 0];
    if let Some(p) = &descriptor.primitive {
        primitive[0] = p.topology.map(dawn_topology).unwrap_or(0);
        primitive[1] = p.strip_index_format.map(dawn_index_format).unwrap_or(0);
        primitive[2] = p.front_face.map(dawn_front_face).unwrap_or(0);
        primitive[3] = p.cull_mode.map(dawn_cull).unwrap_or(0);
    }
    let mut multisample = Vec::new();
    if let Some(ms) = &descriptor.multisample {
        let count = ms.count.unwrap_or(0) as i32;
        let has_mask = if ms.mask.is_some() { 1 } else { 0 };
        let mask = ms.mask.unwrap_or(0) as i32;
        let alpha = match ms.alpha_to_coverage_enabled {
            Some(true) => 1,
            Some(false) => 0,
            None => -1,
        };
        multisample.extend_from_slice(&[count, has_mask, mask, alpha]);
    }
    let mut blend = Vec::new();
    let mut write_mask = Vec::new();
    if let Some(fragment) = &descriptor.fragment {
        for target in fragment.targets.iter().flatten() {
            match &target.blend {
                None => blend.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0]),
                Some(b) => {
                    blend.push(1);
                    blend.push(b.color.operation.map(dawn_blend_op).unwrap_or(0));
                    blend.push(b.color.src_factor.map(dawn_blend_factor).unwrap_or(0));
                    blend.push(b.color.dst_factor.map(dawn_blend_factor).unwrap_or(0));
                    blend.push(b.alpha.operation.map(dawn_blend_op).unwrap_or(0));
                    blend.push(b.alpha.src_factor.map(dawn_blend_factor).unwrap_or(0));
                    blend.push(b.alpha.dst_factor.map(dawn_blend_factor).unwrap_or(0));
                }
            }
            write_mask.push(pack_color_write(target.write_mask));
        }
    }
    let depth_stencil = pack_depth_stencil(&descriptor.depth_stencil);
    (primitive, multisample, blend, write_mask, depth_stencil)
}

fn pack_color_clear_bits(c: &GpuColor) -> [i32; 4] {
    [
        (c.r as f32).to_bits() as i32,
        (c.g as f32).to_bits() as i32,
        (c.b as f32).to_bits() as i32,
        (c.a as f32).to_bits() as i32,
    ]
}

/// Guest `option<record-gpu-pipeline-constant-value>` → host handle (0 = none).
fn pipeline_constant_rep(rec: &Option<Resource<RecordGpuPipelineConstantValue>>) -> i32 {
    rec.as_ref().map(|r| r.rep() as i32).unwrap_or(0)
}

/// Official WASI 0.3.0 `wasi:clocks/system-clock` `instant` record
/// (`seconds: s64`, `nanoseconds: u32`). Not a timezone type.
#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct SystemClockInstant {
    seconds: i64,
    nanoseconds: u32,
}

/// WASI 0.3.0 `wasi:cli/types` `error-code` (official: io / illegal-byte-sequence / pipe).
#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum CliErrorCode {
    #[component(name = "io")]
    Io,
    #[component(name = "illegal-byte-sequence")]
    IllegalByteSequence,
    #[component(name = "pipe")]
    Pipe,
}

/// WASI 0.3.0 `wasi:filesystem` `error-code` (official variant; last case `other`).
#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum FsErrorCode {
    #[component(name = "access")]
    Access,
    #[component(name = "already")]
    Already,
    #[component(name = "bad-descriptor")]
    BadDescriptor,
    #[component(name = "busy")]
    Busy,
    #[component(name = "deadlock")]
    Deadlock,
    #[component(name = "quota")]
    Quota,
    #[component(name = "exist")]
    Exist,
    #[component(name = "file-too-large")]
    FileTooLarge,
    #[component(name = "illegal-byte-sequence")]
    IllegalByteSequence,
    #[component(name = "in-progress")]
    InProgress,
    #[component(name = "interrupted")]
    Interrupted,
    #[component(name = "invalid")]
    Invalid,
    #[component(name = "io")]
    Io,
    #[component(name = "is-directory")]
    IsDirectory,
    #[component(name = "loop")]
    Loop,
    #[component(name = "too-many-links")]
    TooManyLinks,
    #[component(name = "message-size")]
    MessageSize,
    #[component(name = "name-too-long")]
    NameTooLong,
    #[component(name = "no-device")]
    NoDevice,
    #[component(name = "no-entry")]
    NoEntry,
    #[component(name = "no-lock")]
    NoLock,
    #[component(name = "insufficient-memory")]
    InsufficientMemory,
    #[component(name = "insufficient-space")]
    InsufficientSpace,
    #[component(name = "not-directory")]
    NotDirectory,
    #[component(name = "not-empty")]
    NotEmpty,
    #[component(name = "not-recoverable")]
    NotRecoverable,
    #[component(name = "unsupported")]
    Unsupported,
    #[component(name = "no-tty")]
    NoTty,
    #[component(name = "no-such-device")]
    NoSuchDevice,
    #[component(name = "overflow")]
    Overflow,
    #[component(name = "not-permitted")]
    NotPermitted,
    #[component(name = "pipe")]
    Pipe,
    #[component(name = "read-only")]
    ReadOnly,
    #[component(name = "invalid-seek")]
    InvalidSeek,
    #[component(name = "text-file-busy")]
    TextFileBusy,
    #[component(name = "cross-device")]
    CrossDevice,
    #[component(name = "other")]
    Other(Option<String>),
}

fn fs_error_from_io(err: &std::io::Error) -> FsErrorCode {
    use std::io::ErrorKind::*;
    match err.kind() {
        NotFound => FsErrorCode::NoEntry,
        PermissionDenied => FsErrorCode::Access,
        AlreadyExists => FsErrorCode::Exist,
        InvalidInput => FsErrorCode::Invalid,
        Interrupted => FsErrorCode::Interrupted,
        OutOfMemory => FsErrorCode::InsufficientMemory,
        BrokenPipe => FsErrorCode::Pipe,
        Unsupported => FsErrorCode::Unsupported,
        IsADirectory => FsErrorCode::IsDirectory,
        NotADirectory => FsErrorCode::NotDirectory,
        DirectoryNotEmpty => FsErrorCode::NotEmpty,
        ReadOnlyFilesystem => FsErrorCode::ReadOnly,
        StorageFull => FsErrorCode::InsufficientSpace,
        FileTooLarge => FsErrorCode::FileTooLarge,
        QuotaExceeded => FsErrorCode::Quota,
        InvalidFilename => FsErrorCode::IllegalByteSequence,
        NotSeekable => FsErrorCode::InvalidSeek,
        _ => FsErrorCode::Io,
    }
}

/// Host `resource descriptor` for the W6 preopen smoke. Path is under the
/// process sandbox root (see `filesystem_sandbox_join`).
/// `writer` joins before read so guests can drop the write future (official
/// `error-code` has `other(option<string>)`; `future.read` BLOCKS under sync lift).
struct FsDescriptor {
    path: std::path::PathBuf,
    writer: Option<std::thread::JoinHandle<std::io::Result<()>>>,
}

fn filesystem_sandbox_root() -> std::path::PathBuf {
    std::env::temp_dir().join("wasmtime-android-kt-wasi-fs")
}

/// Relative path only: reject empty, NUL, `..`, `.`, and absolute/prefix paths.
/// Not `/sdcard` or other shared storage — root is `temp_dir()` (Android:
/// app-private cache via `TMPDIR`).
fn filesystem_sandbox_join(rel: &str) -> Result<std::path::PathBuf, FsErrorCode> {
    if rel.is_empty() {
        return Err(FsErrorCode::Invalid);
    }
    if rel.contains('\0') {
        return Err(FsErrorCode::IllegalByteSequence);
    }
    let p = std::path::Path::new(rel);
    if p.components()
        .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err(FsErrorCode::Access);
    }
    Ok(filesystem_sandbox_root().join(p))
}

/// Splice `bytes` into `path` at `offset` (non-zero). Does not truncate prefix.
fn fs_write_at(path: &std::path::Path, offset: u64, bytes: &[u8]) -> std::io::Result<()> {
    let start = offset as usize;
    let mut existing = std::fs::read(path).unwrap_or_default();
    let end = start.saturating_add(bytes.len());
    if existing.len() < end {
        existing.resize(end, 0);
    }
    existing[start..end].copy_from_slice(bytes);
    std::fs::write(path, existing)
}

fn fs_read_from(path: &std::path::Path, offset: u64) -> Vec<u8> {
    let bytes = std::fs::read(path).unwrap_or_default();
    let start = (offset as usize).min(bytes.len());
    bytes[start..].to_vec()
}

fn fs_open_child(
    table: &mut wasmtime::component::ResourceTable,
    parent: &wasmtime::component::Resource<FsDescriptor>,
    rel: &str,
) -> Result<wasmtime::component::Resource<FsDescriptor>, FsErrorCode> {
    let _ = table.get(parent).map_err(|_| FsErrorCode::BadDescriptor)?;
    let child = filesystem_sandbox_join(rel)?;
    if !child.exists() {
        std::fs::write(&child, b"").map_err(|e| fs_error_from_io(&e))?;
    }
    table
        .push(FsDescriptor {
            path: child,
            writer: None,
        })
        .map_err(|_| FsErrorCode::InsufficientMemory)
}

/// Host `resource tcp-socket` for the W7 loopback smoke + P010 outbound dial.
struct TcpSocket {
    client: Option<std::net::TcpStream>,
    server: Option<std::thread::JoinHandle<std::io::Result<()>>>,
    writer: Option<std::thread::JoinHandle<std::io::Result<()>>>,
}

struct TcpConnected {
    client: std::net::TcpStream,
    server: Option<std::thread::JoinHandle<std::io::Result<()>>>,
}

/// WASI 0.3.0 `ip-address-family` (P1-SK1). Smoke uses `ipv4`.
#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum IpAddressFamily {
    #[component(name = "ipv4")]
    Ipv4,
    #[component(name = "ipv6")]
    Ipv6,
}

/// WASI 0.3.0 sockets `error-code` (official variant; last case `other`).
#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum SockErrorCode {
    #[component(name = "access-denied")]
    AccessDenied,
    #[component(name = "not-supported")]
    NotSupported,
    #[component(name = "invalid-argument")]
    InvalidArgument,
    #[component(name = "out-of-memory")]
    OutOfMemory,
    #[component(name = "timeout")]
    Timeout,
    #[component(name = "invalid-state")]
    InvalidState,
    #[component(name = "address-not-bindable")]
    AddressNotBindable,
    #[component(name = "address-in-use")]
    AddressInUse,
    #[component(name = "remote-unreachable")]
    RemoteUnreachable,
    #[component(name = "connection-refused")]
    ConnectionRefused,
    #[component(name = "connection-broken")]
    ConnectionBroken,
    #[component(name = "connection-reset")]
    ConnectionReset,
    #[component(name = "connection-aborted")]
    ConnectionAborted,
    #[component(name = "datagram-too-large")]
    DatagramTooLarge,
    #[component(name = "other")]
    Other(Option<String>),
}

fn sock_error_from_io(err: &std::io::Error) -> SockErrorCode {
    use std::io::ErrorKind::*;
    match err.kind() {
        PermissionDenied => SockErrorCode::AccessDenied,
        InvalidInput => SockErrorCode::InvalidArgument,
        OutOfMemory => SockErrorCode::OutOfMemory,
        TimedOut => SockErrorCode::Timeout,
        AddrNotAvailable => SockErrorCode::AddressNotBindable,
        AddrInUse => SockErrorCode::AddressInUse,
        HostUnreachable | NetworkUnreachable | NetworkDown => SockErrorCode::RemoteUnreachable,
        ConnectionRefused => SockErrorCode::ConnectionRefused,
        BrokenPipe => SockErrorCode::ConnectionBroken,
        ConnectionReset => SockErrorCode::ConnectionReset,
        ConnectionAborted => SockErrorCode::ConnectionAborted,
        Unsupported => SockErrorCode::NotSupported,
        _ => SockErrorCode::Other(None),
    }
}

/// WASI 0.3.0 `ipv4-socket-address` (P1-SK2 / P010-TCP).
/// Loopback: host still ignores port (W7 echo pair). Non-loopback: host dials.
#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct Ipv4SocketAddress {
    port: u16,
    address: (u8, u8, u8, u8),
}

/// WASI 0.3.0 `ip-socket-address` subset (`ipv4` only in this smoke).
#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum IpSocketAddress {
    #[component(name = "ipv4")]
    Ipv4(Ipv4SocketAddress),
}

/// Bind `127.0.0.1:0`, spawn an echo accept thread, return the client stream.
/// Loopback only — not WAN. Blocking IO stays off the CM executor.
fn tcp_loopback_pair() -> std::io::Result<(
    std::net::TcpStream,
    std::thread::JoinHandle<std::io::Result<()>>,
)> {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::time::Duration;

    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
    let addr = listener.local_addr()?;
    let server = std::thread::spawn(move || -> std::io::Result<()> {
        let (mut sock, _) = listener.accept()?;
        sock.set_read_timeout(Some(Duration::from_secs(2)))?;
        sock.set_write_timeout(Some(Duration::from_secs(2)))?;
        let mut buf = Vec::new();
        sock.read_to_end(&mut buf)?;
        sock.write_all(&buf)?;
        Ok(())
    });
    let client = TcpStream::connect_timeout(&addr, Duration::from_secs(2))?;
    client.set_read_timeout(Some(Duration::from_secs(2)))?;
    client.set_write_timeout(Some(Duration::from_secs(2)))?;
    Ok((client, server))
}

/// Guest `connect(ip-socket-address)`: loopback keeps the W7 echo pair;
/// non-loopback **dials that IPv4:port** (P010-TCP). No listen / UDP.
fn tcp_connect_guest(addr: IpSocketAddress) -> std::io::Result<TcpConnected> {
    use std::net::{Ipv4Addr, SocketAddr, TcpStream};
    use std::time::Duration;

    match addr {
        IpSocketAddress::Ipv4(a) => {
            let ip = Ipv4Addr::new(a.address.0, a.address.1, a.address.2, a.address.3);
            if ip.is_loopback() {
                let (client, server) = tcp_loopback_pair()?;
                return Ok(TcpConnected {
                    client,
                    server: Some(server),
                });
            }
            let sock_addr = SocketAddr::from((ip, a.port));
            let client = TcpStream::connect_timeout(&sock_addr, Duration::from_secs(2))?;
            client.set_read_timeout(Some(Duration::from_secs(2)))?;
            client.set_write_timeout(Some(Duration::from_secs(2)))?;
            Ok(TcpConnected {
                client,
                server: None,
            })
        }
    }
}

/// Host `resource request` / `response` for the W8 incoming-handler smoke + P010 body.
struct HttpRequest {
    body: Vec<u8>,
    authority: String,
}

struct HttpResponse {
    status: u16,
    body: Arc<Mutex<Vec<u8>>>,
}

/// P010-GFXH/L: host `wasi-gfx:surface` (pin `v0.2.0`).
struct GfxSurface {
    desc_height: Option<u32>,
    desc_width: Option<u32>,
}

/// P010-GFXL: `wasi-gfx:surface/surface-webgpu` `context` (reuses canvas JNI).
struct GfxWebGpuContext {
    canvas_rep: u32,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GfxSurfaceCreateDesc {
    height: Option<u32>,
    width: Option<u32>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GfxFrameEvent {
    nothing: bool,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GfxResizeEvent {
    height: u32,
    width: u32,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GfxPointerEvent {
    x: f64,
    y: f64,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GfxKey {
    #[component(name = "backquote")]
    Backquote,
    #[component(name = "backslash")]
    Backslash,
    #[component(name = "bracket-left")]
    BracketLeft,
    #[component(name = "bracket-right")]
    BracketRight,
    #[component(name = "comma")]
    Comma,
    #[component(name = "digit0")]
    Digit0,
    #[component(name = "digit1")]
    Digit1,
    #[component(name = "digit2")]
    Digit2,
    #[component(name = "digit3")]
    Digit3,
    #[component(name = "digit4")]
    Digit4,
    #[component(name = "digit5")]
    Digit5,
    #[component(name = "digit6")]
    Digit6,
    #[component(name = "digit7")]
    Digit7,
    #[component(name = "digit8")]
    Digit8,
    #[component(name = "digit9")]
    Digit9,
    #[component(name = "equal")]
    Equal,
    #[component(name = "intl-backslash")]
    IntlBackslash,
    #[component(name = "intl-ro")]
    IntlRo,
    #[component(name = "intl-yen")]
    IntlYen,
    #[component(name = "key-a")]
    KeyA,
    #[component(name = "key-b")]
    KeyB,
    #[component(name = "key-c")]
    KeyC,
    #[component(name = "key-d")]
    KeyD,
    #[component(name = "key-e")]
    KeyE,
    #[component(name = "key-f")]
    KeyF,
    #[component(name = "key-g")]
    KeyG,
    #[component(name = "key-h")]
    KeyH,
    #[component(name = "key-i")]
    KeyI,
    #[component(name = "key-j")]
    KeyJ,
    #[component(name = "key-k")]
    KeyK,
    #[component(name = "key-l")]
    KeyL,
    #[component(name = "key-m")]
    KeyM,
    #[component(name = "key-n")]
    KeyN,
    #[component(name = "key-o")]
    KeyO,
    #[component(name = "key-p")]
    KeyP,
    #[component(name = "key-q")]
    KeyQ,
    #[component(name = "key-r")]
    KeyR,
    #[component(name = "key-s")]
    KeyS,
    #[component(name = "key-t")]
    KeyT,
    #[component(name = "key-u")]
    KeyU,
    #[component(name = "key-v")]
    KeyV,
    #[component(name = "key-w")]
    KeyW,
    #[component(name = "key-x")]
    KeyX,
    #[component(name = "key-y")]
    KeyY,
    #[component(name = "key-z")]
    KeyZ,
    #[component(name = "minus")]
    Minus,
    #[component(name = "period")]
    Period,
    #[component(name = "quote")]
    Quote,
    #[component(name = "semicolon")]
    Semicolon,
    #[component(name = "slash")]
    Slash,
    #[component(name = "alt-left")]
    AltLeft,
    #[component(name = "alt-right")]
    AltRight,
    #[component(name = "backspace")]
    Backspace,
    #[component(name = "caps-lock")]
    CapsLock,
    #[component(name = "context-menu")]
    ContextMenu,
    #[component(name = "control-left")]
    ControlLeft,
    #[component(name = "control-right")]
    ControlRight,
    #[component(name = "enter")]
    Enter,
    #[component(name = "meta-left")]
    MetaLeft,
    #[component(name = "meta-right")]
    MetaRight,
    #[component(name = "shift-left")]
    ShiftLeft,
    #[component(name = "shift-right")]
    ShiftRight,
    #[component(name = "space")]
    Space,
    #[component(name = "tab")]
    Tab,
    #[component(name = "convert")]
    Convert,
    #[component(name = "kana-mode")]
    KanaMode,
    #[component(name = "lang1")]
    Lang1,
    #[component(name = "lang2")]
    Lang2,
    #[component(name = "lang3")]
    Lang3,
    #[component(name = "lang4")]
    Lang4,
    #[component(name = "lang5")]
    Lang5,
    #[component(name = "non-convert")]
    NonConvert,
    #[component(name = "delete")]
    Delete,
    #[component(name = "end")]
    End,
    #[component(name = "help")]
    Help,
    #[component(name = "home")]
    Home,
    #[component(name = "insert")]
    Insert,
    #[component(name = "page-down")]
    PageDown,
    #[component(name = "page-up")]
    PageUp,
    #[component(name = "arrow-down")]
    ArrowDown,
    #[component(name = "arrow-left")]
    ArrowLeft,
    #[component(name = "arrow-right")]
    ArrowRight,
    #[component(name = "arrow-up")]
    ArrowUp,
    #[component(name = "num-lock")]
    NumLock,
    #[component(name = "numpad0")]
    Numpad0,
    #[component(name = "numpad1")]
    Numpad1,
    #[component(name = "numpad2")]
    Numpad2,
    #[component(name = "numpad3")]
    Numpad3,
    #[component(name = "numpad4")]
    Numpad4,
    #[component(name = "numpad5")]
    Numpad5,
    #[component(name = "numpad6")]
    Numpad6,
    #[component(name = "numpad7")]
    Numpad7,
    #[component(name = "numpad8")]
    Numpad8,
    #[component(name = "numpad9")]
    Numpad9,
    #[component(name = "numpad-add")]
    NumpadAdd,
    #[component(name = "numpad-backspace")]
    NumpadBackspace,
    #[component(name = "numpad-clear")]
    NumpadClear,
    #[component(name = "numpad-clear-entry")]
    NumpadClearEntry,
    #[component(name = "numpad-comma")]
    NumpadComma,
    #[component(name = "numpad-decimal")]
    NumpadDecimal,
    #[component(name = "numpad-divide")]
    NumpadDivide,
    #[component(name = "numpad-enter")]
    NumpadEnter,
    #[component(name = "numpad-equal")]
    NumpadEqual,
    #[component(name = "numpad-hash")]
    NumpadHash,
    #[component(name = "numpad-memory-add")]
    NumpadMemoryAdd,
    #[component(name = "numpad-memory-clear")]
    NumpadMemoryClear,
    #[component(name = "numpad-memory-recall")]
    NumpadMemoryRecall,
    #[component(name = "numpad-memory-store")]
    NumpadMemoryStore,
    #[component(name = "numpad-memory-subtract")]
    NumpadMemorySubtract,
    #[component(name = "numpad-multiply")]
    NumpadMultiply,
    #[component(name = "numpad-paren-left")]
    NumpadParenLeft,
    #[component(name = "numpad-paren-right")]
    NumpadParenRight,
    #[component(name = "numpad-star")]
    NumpadStar,
    #[component(name = "numpad-subtract")]
    NumpadSubtract,
    #[component(name = "escape")]
    Escape,
    #[component(name = "f1")]
    F1,
    #[component(name = "f2")]
    F2,
    #[component(name = "f3")]
    F3,
    #[component(name = "f4")]
    F4,
    #[component(name = "f5")]
    F5,
    #[component(name = "f6")]
    F6,
    #[component(name = "f7")]
    F7,
    #[component(name = "f8")]
    F8,
    #[component(name = "f9")]
    F9,
    #[component(name = "f10")]
    F10,
    #[component(name = "f11")]
    F11,
    #[component(name = "f12")]
    F12,
    #[component(name = "fn")]
    Fn,
    #[component(name = "fn-lock")]
    FnLock,
    #[component(name = "print-screen")]
    PrintScreen,
    #[component(name = "scroll-lock")]
    ScrollLock,
    #[component(name = "pause")]
    Pause,
    #[component(name = "browser-back")]
    BrowserBack,
    #[component(name = "browser-favorites")]
    BrowserFavorites,
    #[component(name = "browser-forward")]
    BrowserForward,
    #[component(name = "browser-home")]
    BrowserHome,
    #[component(name = "browser-refresh")]
    BrowserRefresh,
    #[component(name = "browser-search")]
    BrowserSearch,
    #[component(name = "browser-stop")]
    BrowserStop,
    #[component(name = "eject")]
    Eject,
    #[component(name = "launch-app1")]
    LaunchApp1,
    #[component(name = "launch-app2")]
    LaunchApp2,
    #[component(name = "launch-mail")]
    LaunchMail,
    #[component(name = "media-play-pause")]
    MediaPlayPause,
    #[component(name = "media-select")]
    MediaSelect,
    #[component(name = "media-stop")]
    MediaStop,
    #[component(name = "media-track-next")]
    MediaTrackNext,
    #[component(name = "media-track-previous")]
    MediaTrackPrevious,
    #[component(name = "power")]
    Power,
    #[component(name = "sleep")]
    Sleep,
    #[component(name = "audio-volume-down")]
    AudioVolumeDown,
    #[component(name = "audio-volume-mute")]
    AudioVolumeMute,
    #[component(name = "audio-volume-up")]
    AudioVolumeUp,
    #[component(name = "wake-up")]
    WakeUp,
    #[component(name = "hyper")]
    Hyper,
    #[component(name = "super")]
    Super,
    #[component(name = "turbo")]
    Turbo,
    #[component(name = "abort")]
    Abort,
    #[component(name = "resume")]
    Resume,
    #[component(name = "suspend")]
    Suspend,
    #[component(name = "again")]
    Again,
    #[component(name = "copy")]
    Copy,
    #[component(name = "cut")]
    Cut,
    #[component(name = "find")]
    Find,
    #[component(name = "open")]
    Open,
    #[component(name = "paste")]
    Paste,
    #[component(name = "props")]
    Props,
    #[component(name = "select")]
    Select,
    #[component(name = "undo")]
    Undo,
    #[component(name = "hiragana")]
    Hiragana,
    #[component(name = "katakana")]
    Katakana,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GfxKeyEvent {
    key: Option<GfxKey>,
    text: Option<String>,
    #[component(name = "alt-key")]
    alt_key: bool,
    #[component(name = "ctrl-key")]
    ctrl_key: bool,
    #[component(name = "meta-key")]
    meta_key: bool,
    #[component(name = "shift-key")]
    shift_key: bool,
}

/// P010-GFXV: Choreographer / helper vsync fills a 1-slot gate; `poll_produce`
/// on the CM driver (GpuThread) writes one `frame-event`. Unconsumed beats drop.
/// Pin `on-frame` is a sync `func`; no stackful CM async — wait on the gate
/// instead of `Poll::Pending` (guest WAT traps on stream.read BLOCKED).
struct GfxOnFrameProducer {
    gate: Arc<GfxOnFrameGate>,
}

impl<D> StreamProducer<D> for GfxOnFrameProducer {
    type Item = GfxFrameEvent;
    type Buffer = Option<GfxFrameEvent>;

    fn poll_produce<'a>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut store: StoreContextMut<'a, D>,
        mut destination: Destination<'a, Self::Item, Self::Buffer>,
        finish: bool,
    ) -> Poll<wasmtime::Result<StreamResult>> {
        let _ = cx;
        // remaining==0 is a readiness poll. Returning Dropped here marks the
        // writable end closed *before* the guest's first stream.read, which
        // traps (`cannot read after being notified that the writable end dropped`).
        // SurfaceDestroyed / closeGfxOnFrame during configure used to hit that.
        if destination.remaining(&mut store) == Some(0) {
            return Poll::Ready(Ok(match self.gate.wait_ready(finish) {
                GfxOnFrameTake::Item | GfxOnFrameTake::Eof => StreamResult::Completed,
                GfxOnFrameTake::Cancelled => StreamResult::Cancelled,
            }));
        }
        match self.gate.wait_take(finish) {
            GfxOnFrameTake::Item => {
                destination.set_buffer(Some(GfxFrameEvent { nothing: true }));
                Poll::Ready(Ok(StreamResult::Completed))
            }
            GfxOnFrameTake::Eof => Poll::Ready(Ok(StreamResult::Dropped)),
            GfxOnFrameTake::Cancelled => Poll::Ready(Ok(StreamResult::Cancelled)),
        }
    }
}

/// GFX-SIZE: `poll_produce` writes one `resize-event` from the 1-slot gate.
struct GfxOnResizeProducer {
    gate: Arc<GfxOnResizeGate>,
}

impl<D> StreamProducer<D> for GfxOnResizeProducer {
    type Item = GfxResizeEvent;
    type Buffer = Option<GfxResizeEvent>;

    fn poll_produce<'a>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut store: StoreContextMut<'a, D>,
        mut destination: Destination<'a, Self::Item, Self::Buffer>,
        finish: bool,
    ) -> Poll<wasmtime::Result<StreamResult>> {
        let _ = cx;
        if destination.remaining(&mut store) == Some(0) {
            return Poll::Ready(Ok(match self.gate.wait_ready(finish) {
                GfxOnResizeTake::Item(_) | GfxOnResizeTake::Eof => StreamResult::Completed,
                GfxOnResizeTake::Cancelled => StreamResult::Cancelled,
            }));
        }
        match self.gate.wait_take(finish) {
            GfxOnResizeTake::Item(sz) => {
                destination.set_buffer(Some(GfxResizeEvent {
                    height: sz.height,
                    width: sz.width,
                }));
                Poll::Ready(Ok(StreamResult::Completed))
            }
            GfxOnResizeTake::Eof => Poll::Ready(Ok(StreamResult::Dropped)),
            GfxOnResizeTake::Cancelled => Poll::Ready(Ok(StreamResult::Cancelled)),
        }
    }
}

struct GfxPointerProducer {
    gate: Arc<GfxPointerGate>,
}

impl<D> StreamProducer<D> for GfxPointerProducer {
    type Item = GfxPointerEvent;
    type Buffer = Option<GfxPointerEvent>;

    fn poll_produce<'a>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut store: StoreContextMut<'a, D>,
        mut destination: Destination<'a, Self::Item, Self::Buffer>,
        finish: bool,
    ) -> Poll<wasmtime::Result<StreamResult>> {
        let _ = cx;
        if destination.remaining(&mut store) == Some(0) {
            return Poll::Ready(Ok(match self.gate.wait_ready(finish) {
                GfxInputTake::Item(_) | GfxInputTake::Eof => StreamResult::Completed,
                GfxInputTake::Cancelled => StreamResult::Cancelled,
            }));
        }
        match self.gate.wait_take(finish) {
            GfxInputTake::Item(ev) => {
                destination.set_buffer(Some(GfxPointerEvent { x: ev.x, y: ev.y }));
                Poll::Ready(Ok(StreamResult::Completed))
            }
            GfxInputTake::Eof => Poll::Ready(Ok(StreamResult::Dropped)),
            GfxInputTake::Cancelled => Poll::Ready(Ok(StreamResult::Cancelled)),
        }
    }
}

struct GfxKeyProducer {
    gate: Arc<GfxKeyGate>,
}

impl<D> StreamProducer<D> for GfxKeyProducer {
    type Item = GfxKeyEvent;
    type Buffer = Option<GfxKeyEvent>;

    fn poll_produce<'a>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut store: StoreContextMut<'a, D>,
        mut destination: Destination<'a, Self::Item, Self::Buffer>,
        finish: bool,
    ) -> Poll<wasmtime::Result<StreamResult>> {
        let _ = cx;
        if destination.remaining(&mut store) == Some(0) {
            return Poll::Ready(Ok(match self.gate.wait_ready(finish) {
                GfxInputTake::Item(_) | GfxInputTake::Eof => StreamResult::Completed,
                GfxInputTake::Cancelled => StreamResult::Cancelled,
            }));
        }
        match self.gate.wait_take(finish) {
            GfxInputTake::Item(ev) => {
                destination.set_buffer(Some(GfxKeyEvent {
                    key: gfx_key_from_disc(ev.key),
                    text: ev.text,
                    alt_key: ev.alt_key,
                    ctrl_key: ev.ctrl_key,
                    meta_key: ev.meta_key,
                    shift_key: ev.shift_key,
                }));
                Poll::Ready(Ok(StreamResult::Completed))
            }
            GfxInputTake::Eof => Poll::Ready(Ok(StreamResult::Dropped)),
            GfxInputTake::Cancelled => Poll::Ready(Ok(StreamResult::Cancelled)),
        }
    }
}

fn gfx_key_from_disc(disc: Option<u8>) -> Option<GfxKey> {
    let disc = disc?;
    if disc > GfxKey::Katakana as u8 {
        return None;
    }
    // Sequential `#[repr(u8)]` pin enum.
    Some(unsafe { std::mem::transmute::<u8, GfxKey>(disc) })
}

/// Android `KeyEvent.keyCode` → WIT `key`. Unmapped → `none` (caller may still
/// fill `text`).
fn gfx_key_from_android(code: i32) -> Option<GfxKey> {
    use GfxKey::*;
    Some(match code {
        7 => Digit0,
        8 => Digit1,
        9 => Digit2,
        10 => Digit3,
        11 => Digit4,
        12 => Digit5,
        13 => Digit6,
        14 => Digit7,
        15 => Digit8,
        16 => Digit9,
        19 => ArrowUp,
        20 => ArrowDown,
        21 => ArrowLeft,
        22 => ArrowRight,
        29 => KeyA,
        30 => KeyB,
        31 => KeyC,
        32 => KeyD,
        33 => KeyE,
        34 => KeyF,
        35 => KeyG,
        36 => KeyH,
        37 => KeyI,
        38 => KeyJ,
        39 => KeyK,
        40 => KeyL,
        41 => KeyM,
        42 => KeyN,
        43 => KeyO,
        44 => KeyP,
        45 => KeyQ,
        46 => KeyR,
        47 => KeyS,
        48 => KeyT,
        49 => KeyU,
        50 => KeyV,
        51 => KeyW,
        52 => KeyX,
        53 => KeyY,
        54 => KeyZ,
        55 => Comma,
        56 => Period,
        57 => AltLeft,
        58 => AltRight,
        59 => ShiftLeft,
        60 => ShiftRight,
        61 => Tab,
        62 => Space,
        66 => Enter,
        67 => Backspace,
        68 => Backquote,
        69 => Minus,
        70 => Equal,
        71 => BracketLeft,
        72 => BracketRight,
        73 => Backslash,
        74 => Semicolon,
        75 => Quote,
        76 => Slash,
        82 => ContextMenu,
        84 => BrowserSearch,
        92 => PageUp,
        93 => PageDown,
        111 => Escape,
        112 => Delete,
        113 => ControlLeft,
        114 => ControlRight,
        115 => CapsLock,
        116 => ScrollLock,
        117 => MetaLeft,
        118 => MetaRight,
        119 => Fn,
        120 => PrintScreen,
        121 => Pause,
        122 => Home,
        123 => End,
        124 => Insert,
        131 => F1,
        132 => F2,
        133 => F3,
        134 => F4,
        135 => F5,
        136 => F6,
        137 => F7,
        138 => F8,
        139 => F9,
        140 => F10,
        141 => F11,
        142 => F12,
        143 => NumLock,
        144 => Numpad0,
        145 => Numpad1,
        146 => Numpad2,
        147 => Numpad3,
        148 => Numpad4,
        149 => Numpad5,
        150 => Numpad6,
        151 => Numpad7,
        152 => Numpad8,
        153 => Numpad9,
        154 => NumpadDivide,
        155 => NumpadMultiply,
        156 => NumpadSubtract,
        157 => NumpadAdd,
        158 => NumpadDecimal,
        160 => NumpadEnter,
        161 => NumpadEqual,
        162 => NumpadParenLeft,
        163 => NumpadParenRight,
        164 => AudioVolumeMute,
        24 => AudioVolumeUp,
        25 => AudioVolumeDown,
        26 => Power,
        _ => return None,
    })
}

/// WASI 0.3.0 `wasi:http` `error-code` (official variant + payload records).
#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct DnsErrorPayload {
    rcode: Option<String>,
    #[component(name = "info-code")]
    info_code: Option<u16>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct TlsAlertReceivedPayload {
    #[component(name = "alert-id")]
    alert_id: Option<u8>,
    #[component(name = "alert-message")]
    alert_message: Option<String>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct FieldSizePayload {
    #[component(name = "field-name")]
    field_name: Option<String>,
    #[component(name = "field-size")]
    field_size: Option<u32>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum HttpErrorCode {
    #[component(name = "DNS-timeout")]
    DnsTimeout,
    #[component(name = "DNS-error")]
    DnsError(DnsErrorPayload),
    #[component(name = "destination-not-found")]
    DestinationNotFound,
    #[component(name = "destination-unavailable")]
    DestinationUnavailable,
    #[component(name = "destination-IP-prohibited")]
    DestinationIpProhibited,
    #[component(name = "destination-IP-unroutable")]
    DestinationIpUnroutable,
    #[component(name = "connection-refused")]
    ConnectionRefused,
    #[component(name = "connection-terminated")]
    ConnectionTerminated,
    #[component(name = "connection-timeout")]
    ConnectionTimeout,
    #[component(name = "connection-read-timeout")]
    ConnectionReadTimeout,
    #[component(name = "connection-write-timeout")]
    ConnectionWriteTimeout,
    #[component(name = "connection-limit-reached")]
    ConnectionLimitReached,
    #[component(name = "TLS-protocol-error")]
    TlsProtocolError,
    #[component(name = "TLS-certificate-error")]
    TlsCertificateError,
    #[component(name = "TLS-alert-received")]
    TlsAlertReceived(TlsAlertReceivedPayload),
    #[component(name = "HTTP-request-denied")]
    HttpRequestDenied,
    #[component(name = "HTTP-request-length-required")]
    HttpRequestLengthRequired,
    #[component(name = "HTTP-request-body-size")]
    HttpRequestBodySize(Option<u64>),
    #[component(name = "HTTP-request-method-invalid")]
    HttpRequestMethodInvalid,
    #[component(name = "HTTP-request-URI-invalid")]
    HttpRequestUriInvalid,
    #[component(name = "HTTP-request-URI-too-long")]
    HttpRequestUriTooLong,
    #[component(name = "HTTP-request-header-section-size")]
    HttpRequestHeaderSectionSize(Option<u32>),
    #[component(name = "HTTP-request-header-size")]
    HttpRequestHeaderSize(Option<FieldSizePayload>),
    #[component(name = "HTTP-request-trailer-section-size")]
    HttpRequestTrailerSectionSize(Option<u32>),
    #[component(name = "HTTP-request-trailer-size")]
    HttpRequestTrailerSize(FieldSizePayload),
    #[component(name = "HTTP-response-incomplete")]
    HttpResponseIncomplete,
    #[component(name = "HTTP-response-header-section-size")]
    HttpResponseHeaderSectionSize(Option<u32>),
    #[component(name = "HTTP-response-header-size")]
    HttpResponseHeaderSize(FieldSizePayload),
    #[component(name = "HTTP-response-body-size")]
    HttpResponseBodySize(Option<u64>),
    #[component(name = "HTTP-response-trailer-section-size")]
    HttpResponseTrailerSectionSize(Option<u32>),
    #[component(name = "HTTP-response-trailer-size")]
    HttpResponseTrailerSize(FieldSizePayload),
    #[component(name = "HTTP-response-transfer-coding")]
    HttpResponseTransferCoding(Option<String>),
    #[component(name = "HTTP-response-content-coding")]
    HttpResponseContentCoding(Option<String>),
    #[component(name = "HTTP-response-timeout")]
    HttpResponseTimeout,
    #[component(name = "HTTP-upgrade-failed")]
    HttpUpgradeFailed,
    #[component(name = "HTTP-protocol-error")]
    HttpProtocolError,
    #[component(name = "loop-detected")]
    LoopDetected,
    #[component(name = "configuration-error")]
    ConfigurationError,
    #[component(name = "internal-error")]
    InternalError(Option<String>),
}

fn http_authority_reject(authority: &str) -> Option<HttpErrorCode> {
    if authority.to_ascii_lowercase().starts_with("https:") {
        return Some(HttpErrorCode::TlsProtocolError);
    }
    if authority.is_empty() || authority.contains('/') {
        return Some(HttpErrorCode::HttpRequestUriInvalid);
    }
    None
}

fn http_error_from_io(err: &std::io::Error) -> HttpErrorCode {
    use std::io::ErrorKind::*;
    match err.kind() {
        InvalidInput => HttpErrorCode::HttpRequestUriInvalid,
        ConnectionRefused => HttpErrorCode::ConnectionRefused,
        TimedOut => HttpErrorCode::ConnectionTimeout,
        ConnectionReset | ConnectionAborted => HttpErrorCode::ConnectionTerminated,
        _ => HttpErrorCode::InternalError(None),
    }
}

/// HTTP/1.1 GET to `authority` (`host:port`). Wire send — not in-process 200.
/// No TLS crate this lane (size); https is `TLS-protocol-error`. Helper-thread caller.
fn http_send_get(authority: &str) -> std::io::Result<(u16, Vec<u8>)> {
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    if authority.is_empty() || authority.contains('/') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "authority",
        ));
    }
    let (host, port) = authority
        .rsplit_once(':')
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "host:port"))?;
    let ip: std::net::Ipv4Addr = host
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{e}")))?;
    if ip.is_unspecified() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "unspecified",
        ));
    }
    let port: u16 = port
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{e}")))?;
    let addr = SocketAddr::from((ip, port));
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(2))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let req = format!("GET / HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;
    let split = buf
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "no header end"))?;
    let headers = std::str::from_utf8(&buf[..split])
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{e}")))?;
    let status = headers
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "status"))?;
    let body = buf[split + 4..].to_vec();
    Ok((status, body))
}

/// P3-PRIM-5 / W1: collect guest `stream.write` bytes; complete oneshot on drop.
/// `max_per_poll` caps items taken per `poll_consume` (backpressure). Use
/// `usize::MAX` for the original 4-byte `take` / cli stdio path.
struct CollectConsumer {
    buf: Arc<Mutex<Vec<u8>>>,
    done: Option<oneshot::Sender<u32>>,
    max_per_poll: usize,
}

impl Drop for CollectConsumer {
    fn drop(&mut self) {
        if let Some(tx) = self.done.take() {
            let n = self.buf.lock().map(|b| b.len() as u32).unwrap_or(0);
            let _ = tx.send(n);
        }
    }
}

impl StreamConsumer<HostState> for CollectConsumer {
    type Item = u8;

    fn poll_consume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        store: StoreContextMut<HostState>,
        src: Source<'_, Self::Item>,
        finish: bool,
    ) -> Poll<wasmtime::Result<StreamResult>> {
        let this = self.get_mut();
        let mut src = src.as_direct(store);
        let chunk = src.remaining();
        if chunk.is_empty() {
            if finish {
                return Poll::Ready(Ok(StreamResult::Cancelled));
            }
            // Zero-length readiness probe (component-model#561). Completed-on-empty
            // traps. Do not wake_by_ref: that marks the task runnable while guest
            // stream.write is still on the stack, so the executor re-polls until
            // ART's ~1MiB instrument thread overflows (Vivo SIGSEGV).
            // Wasmtime keeps the waker and polls again when the guest writes.
            let _ = cx;
            return Poll::Pending;
        }
        let n = chunk.len().min(this.max_per_poll);
        this.buf.lock().unwrap().extend_from_slice(&chunk[..n]);
        src.mark_read(n);
        Poll::Ready(Ok(StreamResult::Completed))
    }
}

fn l2_supported_limits_handles(
    caller: &mut StoreContextMut<'_, HostState>,
    limits: &Resource<GpuSupportedLimits>,
) -> wasmtime::Result<(jni::objects::GlobalRef, u32, u32)> {
    let (adapter, device) = {
        let entry = caller.data_mut().table.get(limits)?;
        (entry.adapter, entry.device)
    };
    let cb = caller.data().require_webgpu_jni_cb()?;
    let l2_adapter = if adapter == 0 && device == 0 {
        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
    } else {
        adapter
    };
    Ok((cb, l2_adapter, device))
}

fn native_gpu_error(err: crate::native_gpu::NativeGpuError) -> wasmtime::Error {
    wasmtime::Error::msg(err.to_string())
}

fn native_adapter_info_for(
    caller: &mut StoreContextMut<'_, HostState>,
    info: &Resource<GpuAdapterInfo>,
) -> wasmtime::Result<crate::native_gpu::NativeAdapterInfo> {
    let info_adapter = caller.data_mut().table.get(info)?.adapter;
    let gpu = caller.data_mut().require_native_gpu()?;
    let handle = gpu
        .resolve_adapter(info_adapter)
        .map_err(native_gpu_error)?;
    gpu.adapter_info(handle).map_err(native_gpu_error)
}

pub(crate) fn define_host(
    linker: &mut Linker<HostState>,
    fixture_ctors: bool,
) -> Result<(), String> {
    linker
        .root()
        .resource(
            "widget",
            ResourceType::host::<Widget>(),
            |mut store, rep| {
                let resource = Resource::<Widget>::new_own(rep);
                store.data_mut().table.delete(resource)?;
                Ok(())
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .root()
        .func_wrap("make-widget", |mut store, (rep,): (u32,)| {
            let resource = store.data_mut().table.push(Widget { rep })?;
            Ok((resource,))
        })
        .map_err(|e| e.to_string())?;

    linker
        .root()
        .func_wrap("echo-widget", |mut store, (r,): (Resource<Widget>,)| {
            let w = store.data_mut().table.get(&r)?;
            Ok((w.rep,))
        })
        .map_err(|e| e.to_string())?;

    linker
        .root()
        .func_wrap("add", |caller, (a, b): (u32, u32)| {
            let cb = caller
                .data()
                .add_cb
                .as_ref()
                .ok_or_else(|| wasmtime::Error::msg("host add callback not set"))?
                .clone();
            let result = jvm::call_u32_u32_to_u32(&cb, a, b).map_err(wasmtime::Error::msg)?;
            Ok((result,))
        })
        .map_err(|e| e.to_string())?;

    // M2: true CM async host import via official concurrent API + FutureReader complete.
    linker
        .root()
        .func_wrap_concurrent("get", |accessor, ()| {
            Box::pin(async move {
                let (tx, rx) = oneshot::channel::<u32>();
                let mut reader = accessor.with(|mut access| {
                    FutureReader::new(&mut access, async move {
                        match rx.await {
                            Ok(v) => Ok(Some(v)),
                            Err(_) => Err(wasmtime::Error::msg("future rejected/canceled")),
                        }
                    })
                })?;
                // Complete then close so the producer is observed (not left pending).
                tx.send(42)
                    .map_err(|_| wasmtime::Error::msg("no future consumer"))?;
                accessor.with(|mut access| reader.close(&mut access))?;
                Ok((42u32,))
            })
        })
        .map_err(|e| e.to_string())?;

    // WASI 0.3: wasi:random/random@0.3.0 (get-random-u64 + get-random-bytes).
    {
        let mut random = linker
            .instance("wasi:random/random@0.3.0")
            .map_err(|e| e.to_string())?;
        random
            .func_wrap("get-random-u64", |_store, ()| {
                let mut bytes = [0u8; 8];
                getrandom::fill(&mut bytes).map_err(|e| wasmtime::Error::msg(e.to_string()))?;
                Ok((u64::from_ne_bytes(bytes),))
            })
            .map_err(|e| e.to_string())?;
        random
            .func_wrap("get-random-bytes", |_store, (len,): (u64,)| {
                let n = (len as usize).min(4096);
                let mut bytes = vec![0u8; n];
                if n > 0 {
                    getrandom::fill(&mut bytes).map_err(|e| wasmtime::Error::msg(e.to_string()))?;
                }
                Ok((bytes,))
            })
            .map_err(|e| e.to_string())?;
    }

    // WASI 0.3: wasi:clocks/monotonic-clock@0.3.0 (now + resolution + wait-for + wait-until).
    {
        let mut clocks = linker
            .instance("wasi:clocks/monotonic-clock@0.3.0")
            .map_err(|e| e.to_string())?;
        clocks
            .func_wrap("now", |store, ()| {
                // H3: during on-frame, vsync instant of the consumed beat (not wakeup).
                Ok((store
                    .data()
                    .gfx_on_frame
                    .in_frame_instant_ns(wasi_monotonic_now_ns()),))
            })
            .map_err(|e| e.to_string())?;
        clocks
            .func_wrap("resolution", |_store, ()| {
                // Instant is nanosecond-granularity on this host.
                Ok((1u64,))
            })
            .map_err(|e| e.to_string())?;
        // True CM async: yield on oneshot while a helper thread sleeps (no tokio).
        clocks
            .func_wrap_concurrent("wait-for", |_accessor, (ns,): (u64,)| {
                Box::pin(async move {
                    let capped = ns.min(1_000_000_000); // 1s host cap
                    let (tx, rx) = oneshot::channel::<()>();
                    std::thread::spawn(move || {
                        if capped > 0 {
                            std::thread::sleep(std::time::Duration::from_nanos(capped));
                        }
                        let _ = tx.send(());
                    });
                    let _ = rx.await;
                    Ok(())
                })
            })
            .map_err(|e| e.to_string())?;
        clocks
            .func_wrap_concurrent("wait-until", |_accessor, (when,): (u64,)| {
                Box::pin(async move {
                    let now = wasi_monotonic_now_ns();
                    let sleep_ns = when.saturating_sub(now).min(1_000_000_000); // 1s host cap
                    let (tx, rx) = oneshot::channel::<()>();
                    std::thread::spawn(move || {
                        if sleep_ns > 0 {
                            std::thread::sleep(std::time::Duration::from_nanos(sleep_ns));
                        }
                        let _ = tx.send(());
                    });
                    let _ = rx.await;
                    Ok(())
                })
            })
            .map_err(|e| e.to_string())?;
    }

    // WASI 0.3: wasi:clocks/system-clock@0.3.0 (now + resolution).
    // Official instant record {seconds: s64, nanoseconds: u32}. No timezone in 0.3.0 pin.
    {
        let mut clock = linker
            .instance("wasi:clocks/system-clock@0.3.0")
            .map_err(|e| e.to_string())?;
        clock
            .func_wrap("now", |_store, ()| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let d = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default();
                Ok((SystemClockInstant {
                    seconds: d.as_secs() as i64,
                    nanoseconds: d.subsec_nanos(),
                },))
            })
            .map_err(|e| e.to_string())?;
        clock
            .func_wrap("resolution", |_store, ()| {
                Ok((SystemClockInstant {
                    seconds: 0,
                    nanoseconds: 1,
                },))
            })
            .map_err(|e| e.to_string())?;
    }

    // Pipe guest stream<u8> into CollectConsumer; complete future with byte count.
    // Shared by root `take` / `take-chunks`.
    fn pipe_stream_byte_count(
        store: &mut StoreContextMut<HostState>,
        reader: StreamReader<u8>,
        max_per_poll: usize,
    ) -> wasmtime::Result<FutureReader<u32>> {
        let (tx, rx) = oneshot::channel::<u32>();
        let buf = Arc::new(Mutex::new(Vec::new()));
        reader.pipe(
            &mut *store,
            CollectConsumer {
                buf: buf.clone(),
                done: Some(tx),
                max_per_poll,
            },
        )?;
        let fut = FutureReader::new(store, async move {
            let n = match rx.await {
                Ok(n) => n,
                Err(_) => 0,
            };
            Ok::<_, wasmtime::Error>(n)
        })?;
        let _ = buf;
        Ok(fut)
    }

    fn pipe_stream_write_result(
        store: &mut StoreContextMut<HostState>,
        reader: StreamReader<u8>,
    ) -> wasmtime::Result<FutureReader<Result<(), CliErrorCode>>> {
        let (tx, rx) = oneshot::channel::<u32>();
        let buf = Arc::new(Mutex::new(Vec::new()));
        reader.pipe(
            &mut *store,
            CollectConsumer {
                buf: buf.clone(),
                done: Some(tx),
                max_per_poll: usize::MAX,
            },
        )?;
        let fut = FutureReader::new(store, async move {
            let n = match rx.await {
                Ok(n) => n,
                Err(_) => return Ok::<_, wasmtime::Error>(Err(CliErrorCode::Pipe)),
            };
            let bytes = buf.lock().map(|b| b.clone()).unwrap_or_default();
            let _ = n;
            if bytes.iter().any(|&b| b == 0) {
                Ok(Err(CliErrorCode::IllegalByteSequence))
            } else if std::str::from_utf8(&bytes).is_err() {
                Ok(Err(CliErrorCode::Io))
            } else {
                Ok(Ok(()))
            }
        })?;
        Ok(fut)
    }

    // P3-PRIM-5: host consumes guest stream; returns future<u32> byte count.
    linker
        .root()
        .func_wrap(
            "take",
            |mut store: StoreContextMut<HostState>, (reader,): (StreamReader<u8>,)| {
                let fut = pipe_stream_byte_count(&mut store, reader, usize::MAX)?;
                Ok((fut,))
            },
        )
        .map_err(|e| e.to_string())?;

    // W1: same as `take` but 2 bytes per poll so a 12-byte multi-chunk write
    // must complete across several consume polls (backpressure).
    linker
        .root()
        .func_wrap(
            "take-chunks",
            |mut store: StoreContextMut<HostState>, (reader,): (StreamReader<u8>,)| {
                let fut = pipe_stream_byte_count(&mut store, reader, 2)?;
                Ok((fut,))
            },
        )
        .map_err(|e| e.to_string())?;

    // WASI 0.3: wasi:cli/stdout@0.3.0 — official write-via-stream →
    // future<result<_, error-code>> (ok; NUL bytes → illegal-byte-sequence).
    linker
        .instance("wasi:cli/stdout@0.3.0")
        .map_err(|e| e.to_string())?
        .func_wrap(
            "write-via-stream",
            |mut store: StoreContextMut<HostState>, (reader,): (StreamReader<u8>,)| {
                let fut = pipe_stream_write_result(&mut store, reader)?;
                Ok((fut,))
            },
        )
        .map_err(|e| e.to_string())?;

    // WASI 0.3: wasi:cli/stderr@0.3.0 — same official write-via-stream result.
    linker
        .instance("wasi:cli/stderr@0.3.0")
        .map_err(|e| e.to_string())?
        .func_wrap(
            "write-via-stream",
            |mut store: StoreContextMut<HostState>, (reader,): (StreamReader<u8>,)| {
                let fut = pipe_stream_write_result(&mut store, reader)?;
                Ok((fut,))
            },
        )
        .map_err(|e| e.to_string())?;

    // WASI 0.3: wasi:cli/stdin@0.3.0 — official read-via-stream →
    // tuple<stream<u8>, future<result<_, error-code>>> (ok path).
    linker
        .instance("wasi:cli/stdin@0.3.0")
        .map_err(|e| e.to_string())?
        .func_wrap(
            "read-via-stream",
            |mut store: StoreContextMut<HostState>, ()| {
                let reader = StreamReader::new(&mut store, b"IN\n".to_vec())?;
                let fut = FutureReader::new(&mut store, async move {
                    Ok::<_, wasmtime::Error>(Ok::<(), CliErrorCode>(()))
                })?;
                Ok(((reader, fut),))
            },
        )
        .map_err(|e| e.to_string())?;

    // WASI 0.3: wasi:cli/environment@0.3.0 — get-environment / get-arguments.
    // Android: empty or documented TMPDIR only (not a full process-env dump).
    // get-initial-cwd is not this lane.
    {
        let mut environment = linker
            .instance("wasi:cli/environment@0.3.0")
            .map_err(|e| e.to_string())?;
        environment
            .func_wrap("get-environment", |_store, ()| {
                let pairs = match std::env::var("TMPDIR") {
                    Ok(v) => vec![("TMPDIR".to_string(), v)],
                    Err(_) => Vec::new(),
                };
                Ok((pairs,))
            })
            .map_err(|e| e.to_string())?;
        environment
            .func_wrap("get-arguments", |_store, ()| Ok((Vec::<String>::new(),)))
            .map_err(|e| e.to_string())?;
    }

    // WASI 0.3: wasi:filesystem Android sandbox (W6 + P1-FS1–FS3).
    // Official packages: wasi:filesystem/types@0.3.0 + preopens@0.3.0.
    // get-directories → list (sandbox directory, ".");
    // open-at(path) -> result; `..` is error-code.access; r/w on the child.
    {
        let mut types = linker
            .instance("wasi:filesystem/types@0.3.0")
            .map_err(|e| e.to_string())?;
        types
            .resource(
                "descriptor",
                ResourceType::host::<FsDescriptor>(),
                |mut store, rep| {
                    let resource = Resource::<FsDescriptor>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        types
            .func_wrap(
                "[method]descriptor.write-via-stream",
                |mut store, (desc, reader, offset): (Resource<FsDescriptor>, StreamReader<u8>, u64)| {
                    let path = store.data_mut().table.get(&desc)?.path.clone();
                    let (tx, rx) = oneshot::channel::<u32>();
                    let buf = Arc::new(Mutex::new(Vec::new()));
                    reader.pipe(
                        &mut store,
                        CollectConsumer {
                            buf: buf.clone(),
                            done: Some(tx),
                            max_per_poll: usize::MAX,
                        },
                    )?;
                    let writer = std::thread::spawn(move || {
                        let _n = pollster::block_on(rx).unwrap_or(0);
                        let _ = _n;
                        let bytes = buf.lock().map(|b| b.clone()).unwrap_or_default();
                        if offset == 0 {
                            std::fs::write(&path, bytes)
                        } else {
                            fs_write_at(&path, offset, &bytes)
                        }
                    });
                    store.data_mut().table.get_mut(&desc)?.writer = Some(writer);
                    let fut = FutureReader::new(&mut store, async move {
                        Ok::<_, wasmtime::Error>(Ok::<(), FsErrorCode>(()))
                    })?;
                    Ok((fut,))
                },
            )
            .map_err(|e| e.to_string())?;
        types
            .func_wrap(
                "[method]descriptor.read-via-stream",
                |mut store, (desc, offset): (Resource<FsDescriptor>, u64)| {
                    let entry = store.data_mut().table.get_mut(&desc)?;
                    if let Some(h) = entry.writer.take() {
                        let _ = h.join();
                    }
                    let path = entry.path.clone();
                    let bytes = fs_read_from(&path, offset);
                    let reader = StreamReader::new(&mut store, bytes)?;
                    let fut = FutureReader::new(&mut store, async move {
                        Ok::<_, wasmtime::Error>(Ok::<(), FsErrorCode>(()))
                    })?;
                    Ok(((reader, fut),))
                },
            )
            .map_err(|e| e.to_string())?;
        types
            .func_wrap(
                "[method]descriptor.open-at",
                |mut store, (desc, path): (Resource<FsDescriptor>, String)| match fs_open_child(
                    &mut store.data_mut().table,
                    &desc,
                    &path,
                ) {
                    Ok(child) => Ok((Ok(child),)),
                    Err(code) => Ok((Err(code),)),
                },
            )
            .map_err(|e| e.to_string())?;
    }
    {
        let mut preopens = linker
            .instance("wasi:filesystem/preopens@0.3.0")
            .map_err(|e| e.to_string())?;
        preopens
            .resource(
                "descriptor",
                ResourceType::host::<FsDescriptor>(),
                |mut store, rep| {
                    let resource = Resource::<FsDescriptor>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        preopens
            .func_wrap("get-directories", |mut store, ()| {
                std::fs::create_dir_all(filesystem_sandbox_root())
                    .map_err(|e| wasmtime::Error::msg(format!("sandbox mkdir: {e}")))?;
                let resource = store.data_mut().table.push(FsDescriptor {
                    path: filesystem_sandbox_root(),
                    writer: None,
                })?;
                Ok((vec![(resource, ".".to_string())],))
            })
            .map_err(|e| e.to_string())?;
    }

    // WASI 0.3: wasi:sockets Android subset (W7 + P1-SK1 + P1-SK2 + P010-TCP).
    // Official packages: wasi:sockets/tcp@0.3.0 + tcp-create-socket@0.3.0.
    // create-tcp-socket(ip-address-family) -> result; connect is a sync WIT
    // func (wasm-tools 1.239 cannot parse import `func async`) that still
    // dials on a helper thread. Loopback: host ignores port (echo pair).
    // Non-loopback: host dials that IPv4:port. write/read via streams (cli shapes).
    // No UDP, no listen, no ip-name-lookup. INTERNET + helper-thread: threading-android.md.
    {
        let mut tcp = linker
            .instance("wasi:sockets/tcp@0.3.0")
            .map_err(|e| e.to_string())?;
        tcp.resource(
            "tcp-socket",
            ResourceType::host::<TcpSocket>(),
            |mut store, rep| {
                let resource = Resource::<TcpSocket>::new_own(rep);
                store.data_mut().table.delete(resource)?;
                Ok(())
            },
        )
        .map_err(|e| e.to_string())?;
        tcp.func_wrap(
            "[method]tcp-socket.connect",
            |mut store, (sock, addr): (Resource<TcpSocket>, IpSocketAddress)| {
                store.data_mut().table.get(&sock)?;
                let (done_tx, done_rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let _ = done_tx.send(tcp_connect_guest(addr));
                });
                let connected = match done_rx
                    .recv()
                    .map_err(|_| wasmtime::Error::msg("connect canceled"))?
                {
                    Ok(c) => c,
                    Err(e) => return Ok((Err(sock_error_from_io(&e)),)),
                };
                let entry = store.data_mut().table.get_mut(&sock)?;
                entry.client = Some(connected.client);
                entry.server = connected.server;
                Ok((Ok::<(), SockErrorCode>(()),))
            },
        )
        .map_err(|e| e.to_string())?;
        tcp.func_wrap(
            "[method]tcp-socket.write-via-stream",
            |mut store, (sock, reader): (Resource<TcpSocket>, StreamReader<u8>)| {
                let client = store
                    .data_mut()
                    .table
                    .get(&sock)?
                    .client
                    .as_ref()
                    .ok_or_else(|| wasmtime::Error::msg("tcp-socket not connected"))?
                    .try_clone()?;
                let (tx, rx) = oneshot::channel::<u32>();
                let buf = Arc::new(Mutex::new(Vec::new()));
                reader.pipe(
                    &mut store,
                    CollectConsumer {
                        buf: buf.clone(),
                        done: Some(tx),
                        max_per_poll: usize::MAX,
                    },
                )?;
                let writer = std::thread::spawn(move || {
                    let _n = pollster::block_on(rx).unwrap_or(0);
                    let _ = _n;
                    let bytes = buf.lock().map(|b| b.clone()).unwrap_or_default();
                    use std::io::Write;
                    let mut client = client;
                    client
                        .write_all(&bytes)
                        .and_then(|_| client.shutdown(std::net::Shutdown::Write))
                });
                store.data_mut().table.get_mut(&sock)?.writer = Some(writer);
                let fut = FutureReader::new(&mut store, async move {
                    Ok::<_, wasmtime::Error>(Ok::<(), SockErrorCode>(()))
                })?;
                Ok((fut,))
            },
        )
        .map_err(|e| e.to_string())?;
        tcp.func_wrap(
            "[method]tcp-socket.read-via-stream",
            |mut store, (sock,): (Resource<TcpSocket>,)| {
                use std::io::Read;
                let entry = store.data_mut().table.get_mut(&sock)?;
                if let Some(h) = entry.writer.take() {
                    let _ = h.join();
                }
                let mut client = entry
                    .client
                    .as_ref()
                    .ok_or_else(|| wasmtime::Error::msg("tcp-socket not connected"))?
                    .try_clone()?;
                let mut incoming = Vec::new();
                client
                    .read_to_end(&mut incoming)
                    .map_err(|e| wasmtime::Error::msg(format!("loopback read: {e}")))?;
                if let Some(h) = entry.server.take() {
                    let _ = h.join();
                }
                let reader = StreamReader::new(&mut store, incoming)?;
                let fut = FutureReader::new(&mut store, async move {
                    Ok::<_, wasmtime::Error>(Ok::<(), SockErrorCode>(()))
                })?;
                Ok(((reader, fut),))
            },
        )
        .map_err(|e| e.to_string())?;
    }
    {
        let mut create = linker
            .instance("wasi:sockets/tcp-create-socket@0.3.0")
            .map_err(|e| e.to_string())?;
        create
            .resource(
                "tcp-socket",
                ResourceType::host::<TcpSocket>(),
                |mut store, rep| {
                    let resource = Resource::<TcpSocket>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        create
            .func_wrap(
                "create-tcp-socket",
                |mut store, (family,): (IpAddressFamily,)| match family {
                    IpAddressFamily::Ipv4 => {
                        let resource = store.data_mut().table.push(TcpSocket {
                            client: None,
                            server: None,
                            writer: None,
                        })?;
                        Ok((Ok(resource),))
                    }
                    IpAddressFamily::Ipv6 => Ok((Err(SockErrorCode::NotSupported),)),
                },
            )
            .map_err(|e| e.to_string())?;
    }

    // WASI 0.3: wasi:http incoming-handler subset (W8 + P1-HT1 + P010-HBODY + P010-HOUT).
    // Official packages: wasi:http/types@0.3.0 + guest export incoming-handler@0.3.0
    // + wasi:http/client@0.3.0#send (0.3 equivalent of outgoing-handler).
    // Subset: constructors + status-code; handle is guest-exported
    // async func(own<request>) -> result<own<response>, error-code> (ok path).
    // Body: [static]request.consume-body / [static]response.consume-body →
    // tuple<stream<u8>, future<result>> (no trailers / res-future param);
    // [static]response.new(contents: stream<u8>) → tuple<response, future>
    // (no headers). Outbound: set-authority + client.send HTTP/1.1 GET on the
    // wire (helper thread). Product linker omits [constructor]request/response
    // (P010-HCTOR; test linker keeps them). No TLS crate / https → TLS-protocol-error.
    {
        let mut types = linker
            .instance("wasi:http/types@0.3.0")
            .map_err(|e| e.to_string())?;
        types
            .resource(
                "request",
                ResourceType::host::<HttpRequest>(),
                |mut store, rep| {
                    let resource = Resource::<HttpRequest>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        types
            .resource(
                "response",
                ResourceType::host::<HttpResponse>(),
                |mut store, rep| {
                    let resource = Resource::<HttpResponse>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        // P010-HCTOR: product linker omits [constructor]request / [constructor]response.
        // Host supplies request when calling handle. Test linker keeps the ctors.
        if fixture_ctors {
            types
                .func_wrap("[constructor]request", |mut store, ()| {
                    let resource = store.data_mut().table.push(HttpRequest {
                        body: b"HBOD".to_vec(),
                        authority: String::new(),
                    })?;
                    Ok((resource,))
                })
                .map_err(|e| e.to_string())?;
            types
                .func_wrap("[constructor]response", |mut store, ()| {
                    let resource = store.data_mut().table.push(HttpResponse {
                        status: 200,
                        body: Arc::new(Mutex::new(Vec::new())),
                    })?;
                    Ok((resource,))
                })
                .map_err(|e| e.to_string())?;
        }
        types
            .func_wrap(
                "[method]response.status-code",
                |mut store, (resp,): (Resource<HttpResponse>,)| {
                    Ok((store.data_mut().table.get(&resp)?.status,))
                },
            )
            .map_err(|e| e.to_string())?;
        types
            .func_wrap(
                "[static]request.consume-body",
                |mut store, (this,): (Resource<HttpRequest>,)| {
                    let req = store.data_mut().table.delete(this)?;
                    let reader = StreamReader::new(&mut store, req.body)?;
                    let fut = FutureReader::new(&mut store, async move {
                        Ok::<_, wasmtime::Error>(Ok::<(), HttpErrorCode>(()))
                    })?;
                    Ok(((reader, fut),))
                },
            )
            .map_err(|e| e.to_string())?;
        types
            .func_wrap(
                "[static]response.new",
                |mut store, (reader,): (StreamReader<u8>,)| {
                    let buf = Arc::new(Mutex::new(Vec::new()));
                    let (tx, rx) = oneshot::channel::<u32>();
                    reader.pipe(
                        &mut store,
                        CollectConsumer {
                            buf: buf.clone(),
                            done: Some(tx),
                            max_per_poll: usize::MAX,
                        },
                    )?;
                    let resource = store.data_mut().table.push(HttpResponse {
                        status: 200,
                        body: buf,
                    })?;
                    let fut = FutureReader::new(&mut store, async move {
                        let _n = rx.await.unwrap_or(0);
                        Ok::<_, wasmtime::Error>(Ok::<(), HttpErrorCode>(()))
                    })?;
                    Ok(((resource, fut),))
                },
            )
            .map_err(|e| e.to_string())?;
        types
            .func_wrap(
                "[static]response.consume-body",
                |mut store, (this,): (Resource<HttpResponse>,)| {
                    let resp = store.data_mut().table.delete(this)?;
                    let bytes = resp.body.lock().map(|b| b.clone()).unwrap_or_default();
                    let reader = StreamReader::new(&mut store, bytes)?;
                    let fut = FutureReader::new(&mut store, async move {
                        Ok::<_, wasmtime::Error>(Ok::<(), HttpErrorCode>(()))
                    })?;
                    Ok(((reader, fut),))
                },
            )
            .map_err(|e| e.to_string())?;
        types
            .func_wrap(
                "[method]request.set-authority",
                |mut store, (req, authority): (Resource<HttpRequest>, String)| {
                    if let Some(code) = http_authority_reject(&authority) {
                        return Ok((Err(code),));
                    }
                    store.data_mut().table.get_mut(&req)?.authority = authority;
                    Ok((Ok::<(), HttpErrorCode>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
    }
    {
        let mut client = linker
            .instance("wasi:http/client@0.3.0")
            .map_err(|e| e.to_string())?;
        client
            .resource(
                "request",
                ResourceType::host::<HttpRequest>(),
                |mut store, rep| {
                    let resource = Resource::<HttpRequest>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        client
            .resource(
                "response",
                ResourceType::host::<HttpResponse>(),
                |mut store, rep| {
                    let resource = Resource::<HttpResponse>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        client
            .func_wrap("send", |mut store, (req,): (Resource<HttpRequest>,)| {
                let authority = store.data_mut().table.delete(req)?.authority;
                if let Some(code) = http_authority_reject(&authority) {
                    return Ok((Err(code),));
                }
                let (done_tx, done_rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let _ = done_tx.send(http_send_get(&authority));
                });
                let outcome = done_rx
                    .recv()
                    .map_err(|_| wasmtime::Error::msg("send canceled"))?;
                match outcome {
                    Ok((status, body)) => {
                        let resource = store.data_mut().table.push(HttpResponse {
                            status,
                            body: Arc::new(Mutex::new(body)),
                        })?;
                        Ok((Ok::<Resource<HttpResponse>, HttpErrorCode>(resource),))
                    }
                    Err(e) => Ok((Err(http_error_from_io(&e)),)),
                }
            })
            .map_err(|e| e.to_string())?;
    }

    // P010-GFXH/V + GFX-SIZE: wasi-gfx:surface@0.2.0 — constructor, on-frame,
    // height/width against the bound window, request-set-size, on-resize.
    // Guest pulls; Choreographer vsync posts a 1-slot gate; poll_produce on the
    // CM driver (GpuThread) writes. Unconsumed beats drop. No JS-style callback.
    {
        let mut surface = linker
            .instance("wasi-gfx:surface/surface@0.2.0")
            .map_err(|e| e.to_string())?;
        surface
            .resource(
                "surface",
                ResourceType::host::<GfxSurface>(),
                |mut store, rep| {
                    let resource = Resource::<GfxSurface>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        surface
            .func_wrap(
                "[constructor]surface",
                |mut store, (desc,): (GfxSurfaceCreateDesc,)| {
                    let resource = store.data_mut().table.push(GfxSurface {
                        desc_height: desc.height,
                        desc_width: desc.width,
                    })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        surface
            .func_wrap(
                "[method]surface.height",
                |mut store, (this,): (Resource<GfxSurface>,)| {
                    let desc = store.data_mut().table.get(&this)?.desc_height;
                    let h = store.data().surface_height(desc);
                    Ok((h,))
                },
            )
            .map_err(|e| e.to_string())?;
        surface
            .func_wrap(
                "[method]surface.width",
                |mut store, (this,): (Resource<GfxSurface>,)| {
                    let desc = store.data_mut().table.get(&this)?.desc_width;
                    let w = store.data().surface_width(desc);
                    Ok((w,))
                },
            )
            .map_err(|e| e.to_string())?;
        surface
            .func_wrap(
                "[method]surface.request-set-size",
                |mut store, (this, height, width): (Resource<GfxSurface>, Option<u32>, Option<u32>)| {
                    store.data_mut().table.get(&this)?;
                    store.data_mut().request_surface_size(height, width);
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        surface
            .func_wrap(
                "[method]surface.on-resize",
                |mut store, (this,): (Resource<GfxSurface>,)| {
                    store.data_mut().table.get(&this)?;
                    let gate = store.data().gfx_on_resize.clone();
                    let reader = StreamReader::new(&mut store, GfxOnResizeProducer { gate })?;
                    Ok((reader,))
                },
            )
            .map_err(|e| e.to_string())?;
        surface
            .func_wrap(
                "[method]surface.on-frame",
                |mut store, (this,): (Resource<GfxSurface>,)| {
                    store.data_mut().table.get(&this)?;
                    let gate = store.data().gfx_on_frame.clone();
                    let reader = StreamReader::new(&mut store, GfxOnFrameProducer { gate })?;
                    Ok((reader,))
                },
            )
            .map_err(|e| e.to_string())?;
        // GFX-PIN: pointer/key streams. Store.postGfxPointer / postGfxKey fill
        // bounded queues. poll_produce waits; it must not return Pending.
        surface
            .func_wrap(
                "[method]surface.on-pointer-up",
                |mut store, (this,): (Resource<GfxSurface>,)| {
                    store.data_mut().table.get(&this)?;
                    let gate = store.data().gfx_input.pointer_up.clone();
                    let reader = StreamReader::new(&mut store, GfxPointerProducer { gate })?;
                    Ok((reader,))
                },
            )
            .map_err(|e| e.to_string())?;
        surface
            .func_wrap(
                "[method]surface.on-pointer-down",
                |mut store, (this,): (Resource<GfxSurface>,)| {
                    store.data_mut().table.get(&this)?;
                    let gate = store.data().gfx_input.pointer_down.clone();
                    let reader = StreamReader::new(&mut store, GfxPointerProducer { gate })?;
                    Ok((reader,))
                },
            )
            .map_err(|e| e.to_string())?;
        surface
            .func_wrap(
                "[method]surface.on-pointer-move",
                |mut store, (this,): (Resource<GfxSurface>,)| {
                    store.data_mut().table.get(&this)?;
                    let gate = store.data().gfx_input.pointer_move.clone();
                    let reader = StreamReader::new(&mut store, GfxPointerProducer { gate })?;
                    Ok((reader,))
                },
            )
            .map_err(|e| e.to_string())?;
        surface
            .func_wrap(
                "[method]surface.on-key-up",
                |mut store, (this,): (Resource<GfxSurface>,)| {
                    store.data_mut().table.get(&this)?;
                    let gate = store.data().gfx_input.key_up.clone();
                    let reader = StreamReader::new(&mut store, GfxKeyProducer { gate })?;
                    Ok((reader,))
                },
            )
            .map_err(|e| e.to_string())?;
        surface
            .func_wrap(
                "[method]surface.on-key-down",
                |mut store, (this,): (Resource<GfxSurface>,)| {
                    store.data_mut().table.get(&this)?;
                    let gate = store.data().gfx_input.key_down.clone();
                    let reader = StreamReader::new(&mut store, GfxKeyProducer { gate })?;
                    Ok((reader,))
                },
            )
            .map_err(|e| e.to_string())?;
    }
    {
        let mut sw = linker
            .instance("wasi-gfx:surface/surface-webgpu@0.2.0")
            .map_err(|e| e.to_string())?;
        sw.resource(
            "context",
            ResourceType::host::<GfxWebGpuContext>(),
            |mut store, rep| {
                let resource = Resource::<GfxWebGpuContext>::new_own(rep);
                store.data_mut().table.delete(resource)?;
                Ok(())
            },
        )
        .map_err(|e| e.to_string())?;
        sw.func_wrap(
            "[constructor]context",
            |mut store, (surf,): (Resource<GfxSurface>,)| {
                store.data_mut().table.get(&surf)?;
                let resource = store
                    .data_mut()
                    .table
                    .push(GfxWebGpuContext { canvas_rep: 0 })?;
                Ok((resource,))
            },
        )
        .map_err(|e| e.to_string())?;
        sw.func_wrap(
            "[method]context.configure",
            |mut caller, (ctx, config): (Resource<GfxWebGpuContext>, GpuCanvasConfiguration)| {
                let ctx_rep = caller.data_mut().table.get(&ctx)?.canvas_rep;
                let device_rep = caller.data_mut().table.get(&config.device)?.rep;
                let format = config.format.to_dawn_u32();
                let usage = config.usage.map(|u| u.to_webgpu_u32()).unwrap_or(0);
                let view_formats: Vec<i32> = config
                    .view_formats
                    .as_ref()
                    .map(|fmts| fmts.iter().map(|f| (*f as i32) + 1).collect())
                    .unwrap_or_default();
                let color_space = config.color_space.map(|c| c as i32).unwrap_or(-1);
                let tone_mapping = config
                    .tone_mapping
                    .and_then(|tm| tm.mode)
                    .map(|m| m as i32)
                    .unwrap_or(-1);
                let alpha_mode = config.alpha_mode.map(|a| a as i32).unwrap_or(-1);
                if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                    let handle = {
                        let gpu = caller.data_mut().require_native_gpu()?;
                        gpu.canvas_configure(
                            ctx_rep,
                            device_rep,
                            format,
                            usage,
                            color_space,
                            tone_mapping,
                            alpha_mode,
                            &view_formats,
                        )
                        .map_err(native_gpu_error)?
                    };
                    caller.data_mut().table.get_mut(&ctx)?.canvas_rep = handle.raw();
                    return Ok(());
                }
                let Some(cb) = caller.data().experimental_host_cb.clone() else {
                    caller.data_mut().table.get_mut(&ctx)?.canvas_rep = 1;
                    return Ok(());
                };
                let l2_device = if device_rep == 0 {
                    let adapter_rep =
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                    jvm::exp_adapter_request_device(&cb, adapter_rep)
                        .map_err(wasmtime::Error::msg)?
                } else {
                    device_rep
                };
                let handle = jvm::exp_canvas_context_configure_described(
                    &cb,
                    ctx_rep,
                    l2_device,
                    format,
                    usage,
                    view_formats,
                    color_space,
                    tone_mapping,
                    alpha_mode,
                )
                .map_err(wasmtime::Error::msg)?;
                if handle == 0 {
                    return Err(wasmtime::Error::msg("gfx context.configure returned 0"));
                }
                caller.data_mut().table.get_mut(&ctx)?.canvas_rep = handle;
                Ok(())
            },
        )
        .map_err(|e| e.to_string())?;
        sw.func_wrap(
            "[method]context.get-current-texture",
            |mut caller, (ctx,): (Resource<GfxWebGpuContext>,)| {
                let ctx_rep = caller.data_mut().table.get(&ctx)?.canvas_rep;
                if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                    let vsync = caller.data().gfx_on_frame.last_take_vsync_ns();
                    let handle = {
                        let gpu = caller.data_mut().require_native_gpu()?;
                        gpu.note_consumed_vsync(vsync);
                        gpu.canvas_current_texture(ctx_rep)
                            .map_err(native_gpu_error)?
                    };
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuTexture { rep: handle.raw() })?;
                    return Ok((resource,));
                }
                let Some(cb) = caller.data().experimental_host_cb.clone() else {
                    let resource = caller.data_mut().table.push(GpuTexture { rep: 0 })?;
                    return Ok((resource,));
                };
                let texture_rep =
                    jvm::exp_canvas_context_get_current_texture_described(&cb, ctx_rep)
                        .map_err(wasmtime::Error::msg)?;
                if texture_rep == 0 {
                    return Err(wasmtime::Error::msg(
                        "gfx context.get-current-texture returned 0",
                    ));
                }
                let resource = caller
                    .data_mut()
                    .table
                    .push(GpuTexture { rep: texture_rep })?;
                Ok((resource,))
            },
        )
        .map_err(|e| e.to_string())?;
        sw.func_wrap(
            "[method]context.present",
            |mut caller, (ctx,): (Resource<GfxWebGpuContext>,)| {
                let ctx_rep = caller.data_mut().table.get(&ctx)?.canvas_rep;
                if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                    let _ = ctx_rep;
                    caller.data_mut().require_native_gpu()?.canvas_present();
                    return Ok(());
                }
                if let Some(cb) = caller.data().experimental_host_cb.clone() {
                    jvm::exp_canvas_context_present_described(&cb, ctx_rep)
                        .map_err(wasmtime::Error::msg)?;
                }
                Ok(())
            },
        )
        .map_err(|e| e.to_string())?;
    }

    // M3/M4: Track A experimental CM host (flat u32 reps) → L2 via Kotlin callbacks.
    // Scope ends before W1 wasi:webgpu dual-register (Linker::instance is once-per-name).
    {
        let mut exp = linker
            .instance("experimental:webgpu-cm/host@0.8.0")
            .map_err(|e| e.to_string())?;

        fn exp_cb(data: &HostState) -> Result<jni::objects::GlobalRef, wasmtime::Error> {
            data.experimental_host_cb
                .as_ref()
                .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                .cloned()
        }

        exp.func_wrap("request-adapter", |caller, ()| {
            let cb = exp_cb(caller.data())?;
            let rep = jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
            Ok((rep,))
        })
        .map_err(|e| e.to_string())?;

        exp.func_wrap("adapter-request-device", |caller, (adapter,): (u32,)| {
            let cb = exp_cb(caller.data())?;
            let rep =
                jvm::exp_adapter_request_device(&cb, adapter).map_err(wasmtime::Error::msg)?;
            Ok((rep,))
        })
        .map_err(|e| e.to_string())?;

        exp.func_wrap("device-get-queue", |caller, (device,): (u32,)| {
            let cb = exp_cb(caller.data())?;
            let rep = jvm::exp_device_get_queue(&cb, device).map_err(wasmtime::Error::msg)?;
            Ok((rep,))
        })
        .map_err(|e| e.to_string())?;

        exp.func_wrap(
            "create-surface-from-native-window",
            |caller, (window,): (u64,)| {
                let cb = exp_cb(caller.data())?;
                let rep = jvm::exp_create_surface(&cb, window).map_err(wasmtime::Error::msg)?;
                Ok((rep,))
            },
        )
        .map_err(|e| e.to_string())?;

        exp.func_wrap(
            "surface-configure",
            |caller, (surface, device, adapter, width, height): (u32, u32, u32, u32, u32)| {
                let cb = exp_cb(caller.data())?;
                let format =
                    jvm::exp_surface_configure(&cb, surface, device, adapter, width, height)
                        .map_err(wasmtime::Error::msg)?;
                Ok((format,))
            },
        )
        .map_err(|e| e.to_string())?;

        exp.func_wrap(
            "surface-get-current-texture-view",
            |caller, (surface,): (u32,)| {
                let cb = exp_cb(caller.data())?;
                let rep = jvm::exp_surface_get_view(&cb, surface).map_err(wasmtime::Error::msg)?;
                Ok((rep,))
            },
        )
        .map_err(|e| e.to_string())?;

        exp.func_wrap(
            "device-create-command-encoder",
            |caller, (device,): (u32,)| {
                let cb = exp_cb(caller.data())?;
                let rep =
                    jvm::exp_create_command_encoder(&cb, device).map_err(wasmtime::Error::msg)?;
                Ok((rep,))
            },
        )
        .map_err(|e| e.to_string())?;

        exp.func_wrap(
            "command-encoder-begin-render-pass-clear",
            |caller, (encoder, view): (u32, u32)| {
                let cb = exp_cb(caller.data())?;
                let rep = jvm::exp_begin_render_pass_clear(&cb, encoder, view)
                    .map_err(wasmtime::Error::msg)?;
                Ok((rep,))
            },
        )
        .map_err(|e| e.to_string())?;

        exp.func_wrap("render-pass-end", |caller, (pass,): (u32,)| {
            let cb = exp_cb(caller.data())?;
            jvm::exp_render_pass_end(&cb, pass).map_err(wasmtime::Error::msg)?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;

        exp.func_wrap("command-encoder-finish", |caller, (encoder,): (u32,)| {
            let cb = exp_cb(caller.data())?;
            let rep =
                jvm::exp_command_encoder_finish(&cb, encoder).map_err(wasmtime::Error::msg)?;
            Ok((rep,))
        })
        .map_err(|e| e.to_string())?;

        exp.func_wrap("queue-submit1", |caller, (queue, commands): (u32, u32)| {
            let cb = exp_cb(caller.data())?;
            jvm::exp_queue_submit1(&cb, queue, commands).map_err(wasmtime::Error::msg)?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;

        exp.func_wrap("surface-present", |caller, (surface,): (u32,)| {
            let cb = exp_cb(caller.data())?;
            jvm::exp_surface_present(&cb, surface).map_err(wasmtime::Error::msg)?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;

        exp.func_wrap("surface-unconfigure", |caller, (surface,): (u32,)| {
            let cb = exp_cb(caller.data())?;
            jvm::exp_surface_unconfigure(&cb, surface).map_err(wasmtime::Error::msg)?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    }

    // W2/W3: proposal instance transitional flat `request-adapter` /
    // `adapter-request-device` as true CM async (`func_wrap_concurrent` + oneshot
    // yield); W3 `device-get-queue`, `device-create-command-encoder`,
    // `command-encoder-finish`, `queue-submit1`,
    // `command-encoder-begin-render-pass-clear`, and `render-pass-end` are sync
    // `func_wrap` (same L2 as experimental). W3 also registers WIT `gpu` +
    // `get-gpu` + `[method]gpu.request-adapter` (S2: async
    // option<own<gpu-adapter>> + option<gpu-request-adapter-options>; P010-GFXB:
    // pin WIT `get-gpu: func() -> gpu` is on the product linker), `gpu-adapter`
    // + `get-adapter`
    // + `[method]gpu-adapter.request-device` (S3: async
    // result<own<gpu-device>, request-device-error> + option<gpu-device-descriptor>),
    // and `gpu-device` + `get-device`
    // + `[method]gpu-device.queue` (S1: sync getter → `own<gpu-queue>`)
    // and `[method]gpu-device.create-command-encoder` (S6: sync
    // (borrow, option<gpu-command-encoder-descriptor>) -> own<gpu-command-encoder>; L2 described label) and `[method]gpu-device.create-buffer`
    // (S4: sync (borrow, gpu-buffer-descriptor) -> own<gpu-buffer>) and
    // `gpu-buffer` + `get-buffer` + `[method]gpu-buffer.map-async` (S6+: true async
    // result<_, map-async-error>; guest mode/offset/size; L2 still host-fixed MAP_READ buffer)
    // and `[method]gpu-buffer.unmap` (S6+: result<_, unmap-error>; L2 described buffer rep)
    // and `[method]gpu-device.create-texture` (S6+: sync (borrow, gpu-texture-descriptor) -> own<gpu-texture>; L2 described size/format/usage/mip/sample/dimension + view-formats + label) and
    // `[method]gpu-device.create-sampler` (S8: sync (borrow, option<gpu-sampler-descriptor>) -> own<gpu-sampler>)
    // and S6+ `[method]gpu-device.create-shader-module` (sync (borrow, gpu-shader-module-descriptor) -> own<gpu-shader-module>; L2 described WGSL code + label + compilation-hints)
    // and `[method]gpu-queue.write-buffer-with-copy` (S6+: borrow buffer + list data → result; L2 described bytes + offset)
    // and S5 `[method]gpu-queue.submit` (sync void; list<borrow<gpu-command-buffer>>; L2 described handles)
    // and S7 `[method]gpu-command-encoder.finish` (sync (borrow, option<gpu-command-buffer-descriptor>) -> own<gpu-command-buffer>; L2 described label)
    // and `gpu-texture` + `get-texture` + S8 `[method]gpu-texture.create-view` (sync (borrow, option<gpu-texture-view-descriptor>) -> own<gpu-texture-view>)
    // and S6+ `[method]gpu-texture.*` info getters / label / set-label (L2 described extent: width/height/depth/mip; remaining still lift-only).
    // and S6+ `[method]record-gpu-pipeline-constant-value.*` map methods (L2 described mutate + iterate).
    // and S6+ `[method]gpu-device.create-bind-group-layout` (sync (borrow, gpu-bind-group-layout-descriptor) -> own<gpu-bind-group-layout>; L2 described all entries)
    // and S6+ `[method]gpu-device.create-pipeline-layout` (sync (borrow, gpu-pipeline-layout-descriptor) -> own<gpu-pipeline-layout>; L2 described BGL handles + label)
    // and S6+ `[method]gpu-device.create-bind-group` (sync (borrow, gpu-bind-group-descriptor) -> own<gpu-bind-group>; L2 described layout + entries + label)
    // and S6+ `[method]gpu-device.create-render-pipeline` (sync (borrow, gpu-render-pipeline-descriptor) -> own<gpu-render-pipeline>; L2 described vertex.buffers + guest color format)
    // and S6+ `[method]gpu-device.create-compute-pipeline` (sync (borrow, gpu-compute-pipeline-descriptor) -> own<gpu-compute-pipeline>; L2 described shader/entry/layout/label)
    // and `[method]gpu-queue.write-texture-with-copy` (S6+: texel copy info + list data; L2 described bytes + size)
    // and S8 `[method]gpu-command-encoder.begin-compute-pass` (sync (borrow, option<gpu-compute-pass-descriptor>) -> own<gpu-compute-pass-encoder>; L2 described timestamp-write indices)
    // and S6+ `[method]gpu-command-encoder.begin-render-pass` (sync (borrow, gpu-render-pass-descriptor) -> own<gpu-render-pass-encoder>; L2 described all color attachments + depth-stencil)
    // and `gpu-compute-pass-encoder` + `get-compute-pass` + `[method]gpu-compute-pass-encoder.end` (sync void; L2 described pass rep)
    // and `[method]gpu-compute-pass-encoder.set-pipeline` (S6+: borrow<gpu-compute-pipeline>; L2 described pass+pipeline reps)
    // and `[method]gpu-compute-pass-encoder.set-bind-group` (S6+: index + option bind-group + option offsets → result; L2 described JNI, offsets none → empty)
    // and `[method]gpu-compute-pass-encoder.dispatch-workgroups` (S6+: x + option y/z; L2 described JNI)
    // and S6+ remaining compute-pass recording: dispatch-workgroups-indirect (L2 described JNI) / set-immediates /
    // push-debug-group / pop-debug-group / insert-debug-marker
    // and S6+ render-pass debug: push-debug-group / pop-debug-group / insert-debug-marker
    // and S6+ remaining render-pass: begin-occlusion-query / end-occlusion-query /
    // execute-bundles / set-immediates
    // and S6+ render-bundle-encoder: finish / set-pipeline / set-bind-group / draw /
    // set-index-buffer / set-vertex-buffer / draw-indexed / draw-indirect /
    // draw-indexed-indirect / push-debug-group / pop-debug-group / insert-debug-marker /
    // set-immediates.
    // and S6+ remaining device create + destroy: create-render-bundle-encoder /
    // create-query-set / device.destroy / buffer.destroy / texture.destroy /
    // query-set.destroy / query-set.type / query-set.count.
    // and S6+ adapter info: adapter.features / limits / info + adapter-info getters.
    // and S6+ bind-group / bind-group-layout / buffer label + set-label and
    // buffer size / usage / map-state.
    // and S6+ command-buffer / encoder label + compilation-info.messages +
    // compilation-message getters.
    // and S6+ compute-pass-encoder / compute-pipeline label + set-label and
    // compute-pipeline.get-bind-group-layout.
    // and S6+ gpu-device adapter-info / features / limits / label / set-label /
    // lost / push-error-scope / pop-error-scope / on-uncaptured-error and
    // gpu-device-lost-info reason / message.
    // and S6+ render-bundle / render-bundle-encoder / render-pass-encoder label +
    // set-label and render-pipeline label / set-label / get-bind-group-layout.
    // and S6+ gpu-supported-limits max-* getters (lift-only stub numerics).
    // and `[method]gpu-render-pass-encoder.set-pipeline` (S6+: borrow<gpu-render-pipeline>; L2 described pass+pipeline reps)
    // and `[method]gpu-render-pass-encoder.draw` (S6+: vertex-count + option instance/first-*; L2 still host-fixed draw(3))
    // and `[method]gpu-render-pass-encoder.set-bind-group` (S6+: index + option bind-group + option offsets → result; L2 described JNI, offsets none → empty)
    // and `[method]gpu-render-pass-encoder.set-vertex-buffer` (S6+: slot + option buffer + option offset/size; L2 described JNI)
    // and `[method]gpu-render-pass-encoder.set-index-buffer` (S6+: buffer + index-format + option offset/size; L2 described JNI)
    // and `[method]gpu-command-encoder.copy-buffer-to-buffer` (S6+: borrow src/dst + option offsets/size; L2 still host-fixed 4-byte copy)
    // and S6+ remaining encoder recording: copy-buffer-to-texture / copy-texture-to-buffer /
    // copy-texture-to-texture / clear-buffer / resolve-query-set / push-debug-group /
    // pop-debug-group / insert-debug-marker.
    // Experimental stays sync.
    // S5: first canonical list is submit; other lists still later.
    {
        // ND-DISP: pin imports dispatch NativeGpu | JniBackend via
        // HostState::webgpu_backend (JNI default; native slot may be unset).
        let mut webgpu = linker
            .instance("wasi:webgpu/webgpu@0.3.0-rc.2")
            .map_err(|e| e.to_string())?;
        webgpu
            .resource("gpu", ResourceType::host::<Gpu>(), |mut store, rep| {
                let resource = Resource::<Gpu>::new_own(rep);
                store.data_mut().table.delete(resource)?;
                Ok(())
            })
            .map_err(|e| e.to_string())?;
        // P010-GFXB: pin WIT `get-gpu: func() -> gpu` is the product entry to
        // `[method]gpu.request-adapter`. `get-device` stays fixture-only.
        webgpu
            .func_wrap("get-gpu", |mut store, ()| {
                let resource = store.data_mut().table.push(Gpu)?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-adapter",
                ResourceType::host::<GpuAdapter>(),
                |mut store, rep| {
                    let resource = Resource::<GpuAdapter>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu.request-adapter",
                |accessor, (gpu, options): (Resource<Gpu>, Option<GpuRequestAdapterOptions>)| {
                    Box::pin(async move {
                        let backend = accessor.with(|mut access| -> wasmtime::Result<_> {
                            let _ = access.data_mut().table.get(&gpu)?;
                            Ok(access.data_mut().webgpu_backend())
                        })?;
                        // True CM async even when unwired (guest `none`, not a trap).
                        let (tx, rx) = oneshot::channel::<()>();
                        std::thread::spawn(move || {
                            let _ = tx.send(());
                        });
                        let _ = rx.await;
                        let (power_preference, force_fallback, feature_level, xr_compatible) =
                            match options.as_ref() {
                                None => (0i32, 0i32, String::new(), -1i32),
                                Some(opts) => {
                                    let power = match opts.power_preference {
                                        None => 0,
                                        Some(p) => p as u8 as i32 + 1,
                                    };
                                    let fallback =
                                        i32::from(opts.force_fallback_adapter.unwrap_or(false));
                                    let feature = opts.feature_level.clone().unwrap_or_default();
                                    let xr = match opts.xr_compatible {
                                        None => -1,
                                        Some(false) => 0,
                                        Some(true) => 1,
                                    };
                                    (power, fallback, feature, xr)
                                }
                            };
                        match backend {
                            GpuBackend::NativeGpu => {
                                let native_opts = NativeRequestAdapterOptions {
                                    feature_level: feature_level.as_str(),
                                    power_preference,
                                    force_fallback_adapter: force_fallback != 0,
                                    xr_compatible: match xr_compatible {
                                        -1 => None,
                                        0 => Some(false),
                                        _ => Some(true),
                                    },
                                };
                                let resource =
                                    accessor.with(|mut access| -> wasmtime::Result<_> {
                                        let handle = {
                                            let gpu = access.data_mut().require_native_gpu()?;
                                            gpu.request_adapter(&native_opts)
                                        };
                                        match handle {
                                            None => Ok(None),
                                            Some(h) => Ok(Some(
                                                access
                                                    .data_mut()
                                                    .table
                                                    .push(GpuAdapter { rep: h.raw() })?,
                                            )),
                                        }
                                    })?;
                                Ok((resource,))
                            }
                            GpuBackend::JniBackend => {
                                let cb = accessor.with(|mut access| -> wasmtime::Result<_> {
                                    Ok(access.data_mut().webgpu_jni_cb())
                                })?;
                                let Some(cb) = cb else {
                                    return Ok((None,));
                                };
                                // L2: `power-preference` 0=none/undefined, 1=low-power, 2=high-performance.
                                // `force-fallback-adapter` 0=none/false, 1=true.
                                // `xr-compatible` -1=none, 0=false, 1=true.
                                let adapter_rep = jvm::exp_request_adapter_described(
                                    &cb,
                                    power_preference,
                                    force_fallback,
                                    feature_level,
                                    xr_compatible,
                                )
                                .map_err(wasmtime::Error::msg)?;
                                if adapter_rep == 0 {
                                    return Ok((None,));
                                }
                                let resource = accessor.with(|mut access| {
                                    access
                                        .data_mut()
                                        .table
                                        .push(GpuAdapter { rep: adapter_rep })
                                })?;
                                Ok((Some(resource),))
                            }
                        }
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "wgsl-language-features",
                ResourceType::host::<WgslLanguageFeatures>(),
                |mut store, rep| {
                    let resource = Resource::<WgslLanguageFeatures>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu.get-preferred-canvas-format",
                |mut caller, (gpu,): (Resource<Gpu>,)| {
                    let _ = caller.data_mut().table.get(&gpu)?;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let dawn = caller
                            .data_mut()
                            .require_native_gpu()?
                            .preferred_canvas_format();
                        return Ok((GpuTextureFormat::from_dawn_u32(dawn),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let dawn = jvm::exp_gpu_get_preferred_canvas_format_described(&cb)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuTextureFormat::from_dawn_u32(dawn),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu.wgsl-language-features",
                |mut caller, (gpu,): (Resource<Gpu>,)| {
                    let _ = caller.data_mut().table.get(&gpu)?;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let resource = caller
                            .data_mut()
                            .table
                            .push(WgslLanguageFeatures { gpu: 0 })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    jvm::exp_gpu_wgsl_language_features_described(&cb)
                        .map_err(wasmtime::Error::msg)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(WgslLanguageFeatures { gpu: 0 })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]wgsl-language-features.has",
                |mut caller, (features, value): (Resource<WgslLanguageFeatures>, String)| {
                    let _features_gpu = caller.data_mut().table.get(&features)?.gpu;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = value;
                        return Ok((false,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let has = jvm::exp_wgsl_language_features_has_described(&cb, value)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((has != 0,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-adapter", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuAdapter { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-supported-features",
                ResourceType::host::<GpuSupportedFeatures>(),
                |mut store, rep| {
                    let resource = Resource::<GpuSupportedFeatures>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-supported-limits",
                ResourceType::host::<GpuSupportedLimits>(),
                |mut store, rep| {
                    let resource = Resource::<GpuSupportedLimits>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-adapter-info",
                ResourceType::host::<GpuAdapterInfo>(),
                |mut store, rep| {
                    let resource = Resource::<GpuAdapterInfo>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter.features",
                |mut caller, (adapter,): (Resource<GpuAdapter>,)| {
                    let backend = caller.data().webgpu_backend();
                    let adapter_rep = caller.data_mut().table.get(&adapter)?.rep;
                    match backend {
                        GpuBackend::NativeGpu => {
                            let l2_adapter = {
                                let gpu = caller.data_mut().require_native_gpu()?;
                                gpu.resolve_adapter(adapter_rep)
                                    .map_err(native_gpu_error)?
                                    .raw()
                            };
                            let resource = caller.data_mut().table.push(GpuSupportedFeatures {
                                adapter: l2_adapter,
                            })?;
                            Ok((resource,))
                        }
                        GpuBackend::JniBackend => {
                            let cb = caller.data().require_webgpu_jni_cb()?;
                            let l2_adapter = if adapter_rep == 0 {
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                            } else {
                                adapter_rep
                            };
                            jvm::exp_adapter_features_described(&cb, l2_adapter)
                                .map_err(wasmtime::Error::msg)?;
                            let resource = caller.data_mut().table.push(GpuSupportedFeatures {
                                adapter: l2_adapter,
                            })?;
                            Ok((resource,))
                        }
                    }
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-features.has",
                |mut caller, (features, value): (Resource<GpuSupportedFeatures>, String)| {
                    let backend = caller.data().webgpu_backend();
                    let features_adapter = caller.data_mut().table.get(&features)?.adapter;
                    match backend {
                        GpuBackend::NativeGpu => {
                            let has = {
                                let gpu = caller.data_mut().require_native_gpu()?;
                                let adapter = gpu
                                    .resolve_adapter(features_adapter)
                                    .map_err(native_gpu_error)?;
                                gpu.adapter_has_feature(adapter, &value)
                                    .map_err(native_gpu_error)?
                            };
                            Ok((has,))
                        }
                        GpuBackend::JniBackend => {
                            let cb = caller.data().require_webgpu_jni_cb()?;
                            let l2_adapter = if features_adapter == 0 {
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                            } else {
                                features_adapter
                            };
                            let has =
                                jvm::exp_supported_features_has_described(&cb, l2_adapter, value)
                                    .map_err(wasmtime::Error::msg)?;
                            Ok((has != 0,))
                        }
                    }
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter.limits",
                |mut caller, (adapter,): (Resource<GpuAdapter>,)| {
                    let backend = caller.data().webgpu_backend();
                    let adapter_rep = caller.data_mut().table.get(&adapter)?.rep;
                    match backend {
                        GpuBackend::NativeGpu => {
                            let l2_adapter = {
                                let gpu = caller.data_mut().require_native_gpu()?;
                                gpu.resolve_adapter(adapter_rep)
                                    .map_err(native_gpu_error)?
                                    .raw()
                            };
                            let resource = caller.data_mut().table.push(GpuSupportedLimits {
                                adapter: l2_adapter,
                                device: 0,
                            })?;
                            Ok((resource,))
                        }
                        GpuBackend::JniBackend => {
                            let cb = caller.data().require_webgpu_jni_cb()?;
                            let l2_adapter = if adapter_rep == 0 {
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                            } else {
                                adapter_rep
                            };
                            jvm::exp_adapter_limits_described(&cb, l2_adapter)
                                .map_err(wasmtime::Error::msg)?;
                            let resource = caller.data_mut().table.push(GpuSupportedLimits {
                                adapter: l2_adapter,
                                device: 0,
                            })?;
                            Ok((resource,))
                        }
                    }
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-supported-limits", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuSupportedLimits {
                    adapter: 0,
                    device: 0,
                })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-bind-groups",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (limits_adapter, limits_device) = {
                        let entry = caller.data_mut().table.get(&limits)?;
                        (entry.adapter, entry.device)
                    };
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((1u32,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_adapter = if limits_adapter == 0 && limits_device == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        limits_adapter
                    };
                    let value = jvm::exp_supported_limits_max_bind_groups_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-bind-groups-plus-vertex-buffers",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (limits_adapter, limits_device) = {
                        let entry = caller.data_mut().table.get(&limits)?;
                        (entry.adapter, entry.device)
                    };
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((1u32,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_adapter = if limits_adapter == 0 && limits_device == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        limits_adapter
                    };
                    let value =
                        jvm::exp_supported_limits_max_bind_groups_plus_vertex_buffers_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-bindings-per-bind-group",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (limits_adapter, limits_device) = {
                        let entry = caller.data_mut().table.get(&limits)?;
                        (entry.adapter, entry.device)
                    };
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((1u32,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_adapter = if limits_adapter == 0 && limits_device == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        limits_adapter
                    };
                    let value = jvm::exp_supported_limits_max_bindings_per_bind_group_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-buffer-size",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (limits_adapter, limits_device) = {
                        let entry = caller.data_mut().table.get(&limits)?;
                        (entry.adapter, entry.device)
                    };
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((1u64,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_adapter = if limits_adapter == 0 && limits_device == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        limits_adapter
                    };
                    let value = jvm::exp_supported_limits_max_buffer_size_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-color-attachment-bytes-per-sample",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_max_color_attachment_bytes_per_sample_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-color-attachments",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_color_attachments_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-invocations-per-workgroup",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_max_compute_invocations_per_workgroup_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-workgroup-size-x",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_compute_workgroup_size_x_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-workgroup-size-y",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_compute_workgroup_size_y_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-workgroup-size-z",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_compute_workgroup_size_z_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-workgroups-per-dimension",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_max_compute_workgroups_per_dimension_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-workgroup-storage-size",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_max_compute_workgroup_storage_size_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-dynamic-storage-buffers-per-pipeline-layout",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_dynamic_storage_buffers_per_pipeline_layout_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-dynamic-uniform-buffers-per-pipeline-layout",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_dynamic_uniform_buffers_per_pipeline_layout_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-immediate-size",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_immediate_size_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-inter-stage-shader-variables",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_max_inter_stage_shader_variables_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-sampled-textures-per-shader-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_max_sampled_textures_per_shader_stage_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-samplers-per-shader-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_samplers_per_shader_stage_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-buffer-binding-size",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u64,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_max_storage_buffer_binding_size_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-buffers-in-fragment-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_max_storage_buffers_in_fragment_stage_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-buffers-in-vertex-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_max_storage_buffers_in_vertex_stage_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-buffers-per-shader-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_max_storage_buffers_per_shader_stage_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-textures-in-fragment-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_max_storage_textures_in_fragment_stage_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-textures-in-vertex-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_max_storage_textures_in_vertex_stage_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-textures-per-shader-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_max_storage_textures_per_shader_stage_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-texture-array-layers",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_texture_array_layers_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-texture-dimension1-d",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_texture_dimension1_d_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-texture-dimension2-d",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_texture_dimension2_d_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-texture-dimension3-d",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_texture_dimension3_d_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-uniform-buffer-binding-size",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u64,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_max_uniform_buffer_binding_size_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-uniform-buffers-per-shader-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_max_uniform_buffers_per_shader_stage_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-vertex-attributes",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_vertex_attributes_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-vertex-buffer-array-stride",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_vertex_buffer_array_stride_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-vertex-buffers",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_vertex_buffers_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.min-storage-buffer-offset-alignment",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_min_storage_buffer_offset_alignment_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.min-uniform-buffer-offset-alignment",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = caller.data_mut().table.get(&limits)?;
                        return Ok((1u32,));
                    }
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value =
                        jvm::exp_supported_limits_min_uniform_buffer_offset_alignment_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter.info",
                |mut caller, (adapter,): (Resource<GpuAdapter>,)| {
                    let backend = caller.data().webgpu_backend();
                    let adapter_rep = caller.data_mut().table.get(&adapter)?.rep;
                    match backend {
                        GpuBackend::NativeGpu => {
                            let l2_adapter = {
                                let gpu = caller.data_mut().require_native_gpu()?;
                                let handle =
                                    gpu.resolve_adapter(adapter_rep).map_err(native_gpu_error)?;
                                let _ = gpu.adapter_info(handle).map_err(native_gpu_error)?;
                                handle.raw()
                            };
                            let resource = caller.data_mut().table.push(GpuAdapterInfo {
                                adapter: l2_adapter,
                            })?;
                            Ok((resource,))
                        }
                        GpuBackend::JniBackend => {
                            let cb = caller.data().require_webgpu_jni_cb()?;
                            let l2_adapter = if adapter_rep == 0 {
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                            } else {
                                adapter_rep
                            };
                            jvm::exp_adapter_info_described(&cb, l2_adapter)
                                .map_err(wasmtime::Error::msg)?;
                            let resource = caller.data_mut().table.push(GpuAdapterInfo {
                                adapter: l2_adapter,
                            })?;
                            Ok((resource,))
                        }
                    }
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-adapter-info", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuAdapterInfo { adapter: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.vendor",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let native = native_adapter_info_for(&mut caller, &info)?;
                        return Ok((native.vendor,));
                    }
                    let info_adapter = caller.data_mut().table.get(&info)?.adapter;
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_adapter = if info_adapter == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        info_adapter
                    };
                    let vendor = jvm::exp_adapter_info_vendor_described(&cb, l2_adapter)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((vendor,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.architecture",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let native = native_adapter_info_for(&mut caller, &info)?;
                        return Ok((native.architecture,));
                    }
                    let info_adapter = caller.data_mut().table.get(&info)?.adapter;
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_adapter = if info_adapter == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        info_adapter
                    };
                    let architecture =
                        jvm::exp_adapter_info_architecture_described(&cb, l2_adapter)
                            .map_err(wasmtime::Error::msg)?;
                    Ok((architecture,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.device",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let native = native_adapter_info_for(&mut caller, &info)?;
                        return Ok((native.device,));
                    }
                    let info_adapter = caller.data_mut().table.get(&info)?.adapter;
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_adapter = if info_adapter == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        info_adapter
                    };
                    let device = jvm::exp_adapter_info_device_described(&cb, l2_adapter)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((device,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.description",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let native = native_adapter_info_for(&mut caller, &info)?;
                        return Ok((native.description,));
                    }
                    let info_adapter = caller.data_mut().table.get(&info)?.adapter;
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_adapter = if info_adapter == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        info_adapter
                    };
                    let description = jvm::exp_adapter_info_description_described(&cb, l2_adapter)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((description,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.subgroup-min-size",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let native = native_adapter_info_for(&mut caller, &info)?;
                        return Ok((native.subgroup_min_size,));
                    }
                    let info_adapter = caller.data_mut().table.get(&info)?.adapter;
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_adapter = if info_adapter == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        info_adapter
                    };
                    let size = jvm::exp_adapter_info_subgroup_min_size_described(&cb, l2_adapter)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((size,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.subgroup-max-size",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let native = native_adapter_info_for(&mut caller, &info)?;
                        return Ok((native.subgroup_max_size,));
                    }
                    let info_adapter = caller.data_mut().table.get(&info)?.adapter;
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_adapter = if info_adapter == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        info_adapter
                    };
                    let size = jvm::exp_adapter_info_subgroup_max_size_described(&cb, l2_adapter)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((size,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.is-fallback-adapter",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let native = native_adapter_info_for(&mut caller, &info)?;
                        return Ok((native.is_fallback_adapter,));
                    }
                    let info_adapter = caller.data_mut().table.get(&info)?.adapter;
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_adapter = if info_adapter == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        info_adapter
                    };
                    let fallback =
                        jvm::exp_adapter_info_is_fallback_adapter_described(&cb, l2_adapter)
                            .map_err(wasmtime::Error::msg)?;
                    Ok((fallback != 0,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "record-option-gpu-size64",
                ResourceType::host::<RecordOptionGpuSize64>(),
                |mut store, rep| {
                    let resource = Resource::<RecordOptionGpuSize64>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("[constructor]record-option-gpu-size64", |mut store, ()| {
                let resource = store.data_mut().table.push(RecordOptionGpuSize64)?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.add",
                |mut caller,
                 (record, key, value): (Resource<RecordOptionGpuSize64>, String, Option<u64>)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller.data_mut().require_native_gpu()?.size64_add(handle, key, value);
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let (has_value, raw) = match value {
                        None => (0i32, 0u64),
                        Some(v) => (1i32, v),
                    };
                    jvm::exp_record_option_gpu_size64_add_described(
                        &cb, handle, key, has_value, raw,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.get",
                |mut caller, (record, key): (Resource<RecordOptionGpuSize64>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let value = caller
                            .data_mut()
                            .require_native_gpu()?
                            .size64_get(handle, &key);
                        return Ok((value,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let state = jvm::exp_record_option_gpu_size64_get_state_described(
                        &cb,
                        handle,
                        key.clone(),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    match state {
                        0 => Ok((None,)),
                        1 => Ok((Some(None),)),
                        _ => {
                            let raw = jvm::exp_record_option_gpu_size64_get_value_described(
                                &cb, handle, key,
                            )
                            .map_err(wasmtime::Error::msg)?;
                            Ok((Some(Some(raw)),))
                        }
                    }
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.has",
                |mut caller, (record, key): (Resource<RecordOptionGpuSize64>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let has = caller
                            .data_mut()
                            .require_native_gpu()?
                            .size64_has(handle, &key);
                        return Ok((has,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let has = jvm::exp_record_option_gpu_size64_has_described(&cb, handle, key)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((has != 0,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.remove",
                |mut caller, (record, key): (Resource<RecordOptionGpuSize64>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .size64_remove(handle, &key);
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    jvm::exp_record_option_gpu_size64_remove_described(&cb, handle, key)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.keys",
                |mut caller, (record,): (Resource<RecordOptionGpuSize64>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let keys = caller.data_mut().require_native_gpu()?.size64_keys(handle);
                        return Ok((keys,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let count = jvm::exp_record_option_gpu_size64_keys_count_described(&cb, handle)
                        .map_err(wasmtime::Error::msg)?;
                    let mut keys = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        keys.push(
                            jvm::exp_record_option_gpu_size64_keys_get_described(&cb, handle, i)
                                .map_err(wasmtime::Error::msg)?,
                        );
                    }
                    Ok((keys,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.values",
                |mut caller, (record,): (Resource<RecordOptionGpuSize64>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let values = caller
                            .data_mut()
                            .require_native_gpu()?
                            .size64_values(handle);
                        return Ok((values,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let count =
                        jvm::exp_record_option_gpu_size64_values_count_described(&cb, handle)
                            .map_err(wasmtime::Error::msg)?;
                    let mut values = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        let state = jvm::exp_record_option_gpu_size64_values_get_state_described(
                            &cb, handle, i,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        if state == 0 {
                            values.push(None);
                        } else {
                            let raw = jvm::exp_record_option_gpu_size64_values_get_value_described(
                                &cb, handle, i,
                            )
                            .map_err(wasmtime::Error::msg)?;
                            values.push(Some(raw));
                        }
                    }
                    Ok((values,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.entries",
                |mut caller, (record,): (Resource<RecordOptionGpuSize64>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let entries = caller
                            .data_mut()
                            .require_native_gpu()?
                            .size64_entries(handle);
                        return Ok((entries,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let count =
                        jvm::exp_record_option_gpu_size64_entries_count_described(&cb, handle)
                            .map_err(wasmtime::Error::msg)?;
                    let mut entries = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        let key = jvm::exp_record_option_gpu_size64_entries_get_key_described(
                            &cb, handle, i,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        let state = jvm::exp_record_option_gpu_size64_entries_get_state_described(
                            &cb, handle, i,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        let value = if state == 0 {
                            None
                        } else {
                            Some(
                                jvm::exp_record_option_gpu_size64_entries_get_value_described(
                                    &cb, handle, i,
                                )
                                .map_err(wasmtime::Error::msg)?,
                            )
                        };
                        entries.push((key, value));
                    }
                    Ok((entries,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-device",
                ResourceType::host::<GpuDevice>(),
                |mut store, rep| {
                    let resource = Resource::<GpuDevice>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu-adapter.request-device",
                |accessor, (adapter, descriptor): (
                    Resource<GpuAdapter>,
                    Option<GpuDeviceDescriptor>,
                )| {
                    Box::pin(async move {
                        let (
                            backend,
                            adapter_rep,
                            required_features,
                            required_limits,
                            label,
                            default_queue_label,
                        ) = accessor.with(|mut access| {
                            let adapter_rep = access.data_mut().table.get(&adapter)?.rep;
                            let (required_features, required_limits, label, default_queue_label) =
                                match descriptor.as_ref() {
                                    None => (Vec::new(), 0i32, String::new(), String::new()),
                                    Some(d) => {
                                        let required_features = d
                                            .required_features
                                            .as_ref()
                                            .map(|v| {
                                                v.iter().map(|f| *f as u8 as i32).collect()
                                            })
                                            .unwrap_or_default();
                                        let required_limits = d
                                            .required_limits
                                            .as_ref()
                                            .map(|r| r.rep() as i32)
                                            .unwrap_or(0);
                                        let label = d.label.clone().unwrap_or_default();
                                        let default_queue_label = d
                                            .default_queue
                                            .as_ref()
                                            .and_then(|q| q.label.clone())
                                            .unwrap_or_default();
                                        (
                                            required_features,
                                            required_limits,
                                            label,
                                            default_queue_label,
                                        )
                                    }
                                };
                            let backend = access.data_mut().webgpu_backend();
                            Ok::<_, wasmtime::Error>((
                                backend,
                                adapter_rep,
                                required_features,
                                required_limits,
                                label,
                                default_queue_label,
                            ))
                        })?;
                        let (tx, rx) = oneshot::channel::<()>();
                        std::thread::spawn(move || {
                            let _ = tx.send(());
                        });
                        let _ = rx.await;
                        match backend {
                            GpuBackend::NativeGpu => {
                                let native_desc = NativeRequestDeviceDescriptor {
                                    required_features: required_features.as_slice(),
                                    required_limits_rep: required_limits,
                                    label: label.as_str(),
                                    default_queue_label: default_queue_label.as_str(),
                                };
                                let resource = accessor.with(|mut access| -> wasmtime::Result<_> {
                                    let handle = {
                                        let gpu = access.data_mut().require_native_gpu()?;
                                        let adapter = gpu
                                            .resolve_adapter(adapter_rep)
                                            .map_err(native_gpu_error)?;
                                        gpu.request_device(adapter, &native_desc)
                                            .map_err(native_gpu_error)?
                                    };
                                    Ok(access
                                        .data_mut()
                                        .table
                                        .push(GpuDevice { rep: handle.raw() })?)
                                })?;
                                Ok((Ok(resource),))
                            }
                            GpuBackend::JniBackend => {
                                let cb = accessor.with(|mut access| {
                                    access.data_mut().require_webgpu_jni_cb()
                                })?;
                                let l2_adapter = if adapter_rep == 0 {
                                    jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                                } else {
                                    adapter_rep
                                };
                                let device_rep = jvm::exp_adapter_request_device_described(
                                    &cb,
                                    l2_adapter,
                                    required_features,
                                    required_limits,
                                    label,
                                    default_queue_label,
                                )
                                .map_err(wasmtime::Error::msg)?;
                                if device_rep == 0 {
                                    return Ok((Err(RequestDeviceError {
                                        kind: RequestDeviceErrorKind::OperationError,
                                        message: "adapter-request-device returned 0".into(),
                                    }),));
                                }
                                let resource = accessor.with(|mut access| {
                                    access
                                        .data_mut()
                                        .table
                                        .push(GpuDevice { rep: device_rep })
                                })?;
                                Ok((Ok(resource),))
                            }
                        }
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        // P010-FIX: `get-device` is a fixture constructor, not the product linker.
        if fixture_ctors {
            webgpu
                .func_wrap("get-device", |mut store, ()| {
                    let resource = store.data_mut().table.push(GpuDevice { rep: 0 })?;
                    Ok((resource,))
                })
                .map_err(|e| e.to_string())?;
        }
        webgpu
            .resource(
                "gpu-queue",
                ResourceType::host::<GpuQueue>(),
                |mut store, rep| {
                    let resource = Resource::<GpuQueue>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.queue",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let backend = caller.data().webgpu_backend();
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    match backend {
                        GpuBackend::NativeGpu => {
                            let handle = {
                                let gpu = caller.data_mut().require_native_gpu()?;
                                let device =
                                    gpu.resolve_device(device_rep).map_err(native_gpu_error)?;
                                gpu.device_queue(device).map_err(native_gpu_error)?
                            };
                            let resource = caller
                                .data_mut()
                                .table
                                .push(GpuQueue { rep: handle.raw() })?;
                            Ok((resource,))
                        }
                        GpuBackend::JniBackend => {
                            let cb = caller.data().require_webgpu_jni_cb()?;
                            let l2_device = if device_rep == 0 {
                                let adapter_rep =
                                    jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                                jvm::exp_adapter_request_device(&cb, adapter_rep)
                                    .map_err(wasmtime::Error::msg)?
                            } else {
                                device_rep
                            };
                            let queue_rep = jvm::exp_device_get_queue_described(&cb, l2_device)
                                .map_err(wasmtime::Error::msg)?;
                            if queue_rep == 0 {
                                return Err(wasmtime::Error::msg("device-queue returned 0"));
                            }
                            let resource =
                                caller.data_mut().table.push(GpuQueue { rep: queue_rep })?;
                            Ok((resource,))
                        }
                    }
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.destroy",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .destroy_rep(crate::native_gpu::ResourceKind::Device, device_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    jvm::exp_device_destroy_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-device-lost-info",
                ResourceType::host::<GpuDeviceLostInfo>(),
                |mut store, rep| {
                    let resource = Resource::<GpuDeviceLostInfo>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-error",
                ResourceType::host::<GpuError>(),
                |mut store, rep| {
                    let resource = Resource::<GpuError>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        // P010-FIX: `get-gpu-error` is a fixture constructor, not the product linker.
        if fixture_ctors {
            webgpu
                .func_wrap("get-gpu-error", |mut store, ()| {
                    let resource = store.data_mut().table.push(GpuError { device: 0 })?;
                    Ok((resource,))
                })
                .map_err(|e| e.to_string())?;
        }
        webgpu
            .func_wrap(
                "[method]gpu-error.message",
                |mut caller, (error,): (Resource<GpuError>,)| {
                    let error_device = caller.data_mut().table.get(&error)?.device;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if error_device == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        error_device
                    };
                    let message = jvm::exp_gpu_error_message_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((message,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-error.kind",
                |mut caller, (error,): (Resource<GpuError>,)| {
                    let error_device = caller.data_mut().table.get(&error)?.device;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((GpuErrorKind::from_host_u32(0),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if error_device == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        error_device
                    };
                    let kind = jvm::exp_gpu_error_kind_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuErrorKind::from_host_u32(kind),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.features",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let adapter = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            let _ = gpu.resolve_device(device_rep).map_err(native_gpu_error)?;
                            gpu.resolve_adapter(0).map_err(native_gpu_error)?.raw()
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuSupportedFeatures { adapter })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    jvm::exp_device_features_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    let adapter_rep = jvm::exp_device_adapter_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    let resource = caller.data_mut().table.push(GpuSupportedFeatures {
                        adapter: adapter_rep,
                    })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.limits",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let device = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            gpu.resolve_device(device_rep)
                                .map_err(native_gpu_error)?
                                .raw()
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuSupportedLimits { adapter: 0, device })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    jvm::exp_device_limits_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    let resource = caller.data_mut().table.push(GpuSupportedLimits {
                        adapter: 0,
                        device: l2_device,
                    })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.adapter-info",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let adapter = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            let _ = gpu.resolve_device(device_rep).map_err(native_gpu_error)?;
                            gpu.resolve_adapter(0).map_err(native_gpu_error)?.raw()
                        };
                        let resource = caller.data_mut().table.push(GpuAdapterInfo { adapter })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    jvm::exp_device_adapter_info_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    let adapter_rep = jvm::exp_device_adapter_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    let resource = caller.data_mut().table.push(GpuAdapterInfo {
                        adapter: adapter_rep,
                    })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.label",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let label = caller.data_mut().require_native_gpu()?.label(device_rep);
                        return Ok((label,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let label =
                        jvm::exp_device_label_described(&cb, l2).map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.set-label",
                |mut caller, (device, label): (Resource<GpuDevice>, String)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .set_label(device_rep, label);
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    jvm::exp_device_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.lost",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let l2_device = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            gpu.resolve_device(device_rep)
                                .map_err(native_gpu_error)?
                                .raw()
                        };
                        let info = caller
                            .data_mut()
                            .table
                            .push(GpuDeviceLostInfo { device: l2_device })?;
                        let fut = FutureReader::new(&mut caller, async move {
                            Ok::<Resource<GpuDeviceLostInfo>, wasmtime::Error>(info)
                        })?;
                        return Ok((fut,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    jvm::exp_device_lost_described(&cb, l2_device).map_err(wasmtime::Error::msg)?;
                    let info = caller
                        .data_mut()
                        .table
                        .push(GpuDeviceLostInfo { device: l2_device })?;
                    let fut = FutureReader::new(&mut caller, async move {
                        Ok::<Resource<GpuDeviceLostInfo>, wasmtime::Error>(info)
                    })?;
                    Ok((fut,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.push-error-scope",
                |mut caller, (device, filter): (Resource<GpuDevice>, GpuErrorFilter)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .push_error_scope(device_rep, filter.to_host_u32() + 1)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    jvm::exp_device_push_error_scope_described(
                        &cb,
                        l2_device,
                        filter.to_host_u32(),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu-device.pop-error-scope",
                |accessor, (device,): (Resource<GpuDevice>,)| {
                    Box::pin(async move {
                        let native = accessor.with(|mut access| -> wasmtime::Result<bool> {
                            let _ = access.data_mut().table.get(&device)?;
                            Ok(access.data_mut().webgpu_backend() == GpuBackend::NativeGpu)
                        })?;
                        if native {
                            accessor.with(|mut access| -> wasmtime::Result<()> {
                                let device_rep = access.data_mut().table.get(&device)?.rep;
                                access
                                    .data_mut()
                                    .require_native_gpu()?
                                    .pop_error_scope(device_rep)
                                    .map_err(native_gpu_error)?;
                                Ok(())
                            })?;
                            return Ok((Ok::<Option<Resource<GpuError>>, PopErrorScopeError>(
                                None,
                            ),));
                        }
                        let (cb, device_rep) =
                            accessor.with(|mut access| -> wasmtime::Result<_> {
                                let device_rep = access.data_mut().table.get(&device)?.rep;
                                let cb = access.data_mut().require_webgpu_jni_cb()?;
                                Ok((cb, device_rep))
                            })?;
                        let l2_device = if device_rep == 0 {
                            let adapter_rep =
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                            jvm::exp_adapter_request_device(&cb, adapter_rep)
                                .map_err(wasmtime::Error::msg)?
                        } else {
                            device_rep
                        };
                        let _ = jvm::exp_device_pop_error_scope_described(&cb, l2_device)
                            .map_err(wasmtime::Error::msg)?;
                        Ok((Ok::<Option<Resource<GpuError>>, PopErrorScopeError>(None),))
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.on-uncaptured-error",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let reader = StreamReader::<Resource<GpuError>>::new(&mut caller, vec![])?;
                        return Ok((reader,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    jvm::exp_device_on_uncaptured_error_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    let reader = StreamReader::<Resource<GpuError>>::new(&mut caller, vec![])?;
                    Ok((reader,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-uncaptured-error-event",
                ResourceType::host::<GpuUncapturedErrorEvent>(),
                |mut store, rep| {
                    let resource = Resource::<GpuUncapturedErrorEvent>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-uncaptured-error-event", |mut store, ()| {
                let resource = store
                    .data_mut()
                    .table
                    .push(GpuUncapturedErrorEvent { device: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-uncaptured-error-event.error",
                |mut caller, (event,): (Resource<GpuUncapturedErrorEvent>,)| {
                    let event_device = caller.data_mut().table.get(&event)?.device;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let resource = caller.data_mut().table.push(GpuError {
                            device: event_device,
                        })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if event_device == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        event_device
                    };
                    jvm::exp_uncaptured_error_event_error_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuError { device: l2_device })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        // P010-FIX: `get-device-lost-info` is a fixture constructor, not the product linker.
        if fixture_ctors {
            webgpu
                .func_wrap("get-device-lost-info", |mut store, ()| {
                    let resource = store
                        .data_mut()
                        .table
                        .push(GpuDeviceLostInfo { device: 0 })?;
                    Ok((resource,))
                })
                .map_err(|e| e.to_string())?;
        }
        webgpu
            .func_wrap(
                "[method]gpu-device-lost-info.reason",
                |mut caller, (info,): (Resource<GpuDeviceLostInfo>,)| {
                    let info_device = caller.data_mut().table.get(&info)?.device;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((GpuDeviceLostReason::Unknown,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if info_device == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        info_device
                    };
                    let reason = jvm::exp_device_lost_info_reason_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuDeviceLostReason::from_host_u32(reason),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device-lost-info.message",
                |mut caller, (info,): (Resource<GpuDeviceLostInfo>,)| {
                    let info_device = caller.data_mut().table.get(&info)?.device;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if info_device == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        info_device
                    };
                    let message = jvm::exp_device_lost_info_message_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((message,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-command-encoder",
                ResourceType::host::<GpuCommandEncoder>(),
                |mut store, rep| {
                    let resource = Resource::<GpuCommandEncoder>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-command-encoder",
                |mut caller,
                 (device, descriptor): (
                    Resource<GpuDevice>,
                    Option<GpuCommandEncoderDescriptor>,
                )| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let label = descriptor
                        .as_ref()
                        .and_then(|d| d.label.clone())
                        .unwrap_or_default();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let handle = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            let device =
                                gpu.resolve_device(device_rep).map_err(native_gpu_error)?;
                            gpu.create_command_encoder(device, &label)
                                .map_err(native_gpu_error)?
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuCommandEncoder { rep: handle.raw() })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let encoder_rep =
                        jvm::exp_create_command_encoder_described(&cb, l2_device, label)
                            .map_err(wasmtime::Error::msg)?;
                    if encoder_rep == 0 {
                        return Err(wasmtime::Error::msg(
                            "device-create-command-encoder returned 0",
                        ));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuCommandEncoder { rep: encoder_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-buffer",
                |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuBufferDescriptor)| {
                    let backend = caller.data().webgpu_backend();
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let mapped = match descriptor.mapped_at_creation {
                        None => -1,
                        Some(false) => 0,
                        Some(true) => 1,
                    };
                    let label = descriptor.label.clone().unwrap_or_default();
                    let size = descriptor.size;
                    let usage = descriptor.usage.to_webgpu_u32();
                    match backend {
                        GpuBackend::NativeGpu => {
                            let handle = {
                                let gpu = caller.data_mut().require_native_gpu()?;
                                let device =
                                    gpu.resolve_device(device_rep).map_err(native_gpu_error)?;
                                gpu.create_buffer(device, size, usage, mapped, &label)
                                    .map_err(native_gpu_error)?
                            };
                            let resource = caller
                                .data_mut()
                                .table
                                .push(GpuBuffer { rep: handle.raw() })?;
                            Ok((resource,))
                        }
                        GpuBackend::JniBackend => {
                            let cb = caller.data().require_webgpu_jni_cb()?;
                            let l2_device = if device_rep == 0 {
                                let adapter_rep =
                                    jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                                jvm::exp_adapter_request_device(&cb, adapter_rep)
                                    .map_err(wasmtime::Error::msg)?
                            } else {
                                device_rep
                            };
                            let buffer_rep = jvm::exp_create_buffer_described(
                                &cb, l2_device, size, usage, mapped, label,
                            )
                            .map_err(wasmtime::Error::msg)?;
                            if buffer_rep == 0 {
                                return Err(wasmtime::Error::msg(
                                    "device-create-buffer returned 0",
                                ));
                            }
                            let resource = caller
                                .data_mut()
                                .table
                                .push(GpuBuffer { rep: buffer_rep })?;
                            Ok((resource,))
                        }
                    }
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-texture",
                ResourceType::host::<GpuTexture>(),
                |mut store, rep| {
                    let resource = Resource::<GpuTexture>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-texture",
                |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuTextureDescriptor)| {
                    let backend = caller.data().webgpu_backend();
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let width = descriptor.size.width;
                    let height = descriptor.size.height.unwrap_or(1);
                    let depth = descriptor.size.depth_or_array_layers.unwrap_or(1);
                    let mip = descriptor.mip_level_count.unwrap_or(1);
                    let sample = descriptor.sample_count.unwrap_or(1);
                    // WIT d1/d2/d3 → Dawn TextureDimension 1D=1, 2D=2, 3D=3 (none → 2D).
                    let dimension = match descriptor.dimension {
                        Some(GpuTextureDimension::D1) => 1u32,
                        Some(GpuTextureDimension::D3) => 3,
                        Some(GpuTextureDimension::D2) | None => 2,
                    };
                    let view_formats: Vec<i32> = descriptor
                        .view_formats
                        .as_ref()
                        .map(|v| v.iter().map(|f| f.to_dawn_u32() as i32).collect())
                        .unwrap_or_default();
                    let label = descriptor.label.clone().unwrap_or_default();
                    let format = descriptor.format.to_dawn_u32();
                    let usage = descriptor.usage.to_webgpu_u32();
                    match backend {
                        GpuBackend::NativeGpu => {
                            let handle = {
                                let gpu = caller.data_mut().require_native_gpu()?;
                                let device =
                                    gpu.resolve_device(device_rep).map_err(native_gpu_error)?;
                                gpu.create_texture(
                                    device,
                                    width,
                                    height,
                                    depth,
                                    format,
                                    usage,
                                    mip,
                                    sample,
                                    dimension,
                                    &view_formats,
                                    &label,
                                )
                                .map_err(native_gpu_error)?
                            };
                            let resource = caller
                                .data_mut()
                                .table
                                .push(GpuTexture { rep: handle.raw() })?;
                            Ok((resource,))
                        }
                        GpuBackend::JniBackend => {
                            let cb = caller.data().require_webgpu_jni_cb()?;
                            let l2_device = if device_rep == 0 {
                                let adapter_rep =
                                    jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                                jvm::exp_adapter_request_device(&cb, adapter_rep)
                                    .map_err(wasmtime::Error::msg)?
                            } else {
                                device_rep
                            };
                            let texture_rep = jvm::exp_create_texture_described(
                                &cb,
                                l2_device,
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
                            )
                            .map_err(wasmtime::Error::msg)?;
                            if texture_rep == 0 {
                                return Err(wasmtime::Error::msg(
                                    "device-create-texture returned 0",
                                ));
                            }
                            let resource = caller
                                .data_mut()
                                .table
                                .push(GpuTexture { rep: texture_rep })?;
                            Ok((resource,))
                        }
                    }
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-texture", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuTexture { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-canvas-context",
                ResourceType::host::<GpuCanvasContext>(),
                |mut store, rep| {
                    let resource = Resource::<GpuCanvasContext>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-canvas-context", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuCanvasContext { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-canvas-context.configure",
                |mut caller, (ctx, config): (Resource<GpuCanvasContext>, GpuCanvasConfiguration)| {
                    let ctx_rep = caller.data_mut().table.get(&ctx)?.rep;
                    let device_rep = caller.data_mut().table.get(&config.device)?.rep;
                    let format = config.format.to_dawn_u32();
                    let usage = config.usage.map(|u| u.to_webgpu_u32()).unwrap_or(0);
                    let view_formats: Vec<i32> = config
                        .view_formats
                        .as_ref()
                        .map(|fmts| fmts.iter().map(|f| (*f as i32) + 1).collect())
                        .unwrap_or_default();
                    let color_space = config.color_space.map(|c| c as i32).unwrap_or(-1);
                    let tone_mapping = config
                        .tone_mapping
                        .and_then(|tm| tm.mode)
                        .map(|m| m as i32)
                        .unwrap_or(-1);
                    let alpha_mode = config.alpha_mode.map(|a| a as i32).unwrap_or(-1);
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let handle = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            gpu.canvas_configure(
                                ctx_rep,
                                device_rep,
                                format,
                                usage,
                                color_space,
                                tone_mapping,
                                alpha_mode,
                                &view_formats,
                            )
                            .map_err(native_gpu_error)?
                        };
                        caller.data_mut().table.get_mut(&ctx)?.rep = handle.raw();
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let handle = jvm::exp_canvas_context_configure_described(
                        &cb,
                        ctx_rep,
                        l2_device,
                        format,
                        usage,
                        view_formats,
                        color_space,
                        tone_mapping,
                        alpha_mode,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if handle == 0 {
                        return Err(wasmtime::Error::msg(
                            "canvas-context-configure returned 0",
                        ));
                    }
                    caller.data_mut().table.get_mut(&ctx)?.rep = handle;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-canvas-context.unconfigure",
                |mut caller, (ctx,): (Resource<GpuCanvasContext>,)| {
                    let ctx_rep = caller.data_mut().table.get(&ctx)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .canvas_unconfigure(ctx_rep);
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    jvm::exp_canvas_context_unconfigure_described(&cb, ctx_rep)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-canvas-context.get-configuration",
                |mut caller, (ctx,): (Resource<GpuCanvasContext>,)| {
                    let ctx_rep = caller.data_mut().table.get(&ctx)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let snap = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            gpu.canvas_configuration(ctx_rep)
                                .map(|c| (c.device, c.format, c.usage))
                        };
                        let Some((device_rep, format, usage)) = snap else {
                            return Ok((Option::<GpuCanvasConfigurationOwned>::None,));
                        };
                        let device = caller
                            .data_mut()
                            .table
                            .push(GpuDevice { rep: device_rep })?;
                        let usage_opt = if usage == 0 {
                            None
                        } else {
                            Some(GpuTextureUsage::from_webgpu_u32(usage))
                        };
                        return Ok((Some(GpuCanvasConfigurationOwned {
                            device,
                            format: GpuTextureFormat::from_dawn_u32(format),
                            usage: usage_opt,
                            view_formats: None,
                            color_space: None,
                            tone_mapping: None,
                            alpha_mode: None,
                        }),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let has = jvm::exp_canvas_context_has_configuration_described(&cb, ctx_rep)
                        .map_err(wasmtime::Error::msg)?;
                    if has == 0 {
                        return Ok((Option::<GpuCanvasConfigurationOwned>::None,));
                    }
                    let device_rep =
                        jvm::exp_canvas_context_configuration_device_described(&cb, ctx_rep)
                            .map_err(wasmtime::Error::msg)?;
                    let format =
                        jvm::exp_canvas_context_configuration_format_described(&cb, ctx_rep)
                            .map_err(wasmtime::Error::msg)?;
                    let usage = jvm::exp_canvas_context_configuration_usage_described(&cb, ctx_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let device = caller
                        .data_mut()
                        .table
                        .push(GpuDevice { rep: device_rep })?;
                    let usage_opt = if usage == 0 {
                        None
                    } else {
                        Some(GpuTextureUsage::from_webgpu_u32(usage))
                    };
                    Ok((Some(GpuCanvasConfigurationOwned {
                        device,
                        format: GpuTextureFormat::from_dawn_u32(format),
                        usage: usage_opt,
                        view_formats: None,
                        color_space: None,
                        tone_mapping: None,
                        alpha_mode: None,
                    }),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-canvas-context.get-current-texture",
                |mut caller, (ctx,): (Resource<GpuCanvasContext>,)| {
                    let ctx_rep = caller.data_mut().table.get(&ctx)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let vsync = caller.data().gfx_on_frame.last_take_vsync_ns();
                        let handle = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            gpu.note_consumed_vsync(vsync);
                            gpu.canvas_current_texture(ctx_rep)
                                .map_err(native_gpu_error)?
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuTexture { rep: handle.raw() })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let texture_rep =
                        jvm::exp_canvas_context_get_current_texture_described(&cb, ctx_rep)
                            .map_err(wasmtime::Error::msg)?;
                    if texture_rep == 0 {
                        return Err(wasmtime::Error::msg(
                            "canvas-context-get-current-texture returned 0",
                        ));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuTexture { rep: texture_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-texture-view",
                ResourceType::host::<GpuTextureView>(),
                |mut store, rep| {
                    let resource = Resource::<GpuTextureView>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.create-view",
                |mut caller,
                 (texture, descriptor): (
                    Resource<GpuTexture>,
                    Option<GpuTextureViewDescriptor>,
                )| {
                    let backend = caller.data().webgpu_backend();
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    let (dimension, aspect, format, base_mip, mip_count, base_layer, layer_count) =
                        match &descriptor {
                            None => (0, 0, 0, 0, -1, 0, -1),
                            Some(d) => (
                                d.dimension.map(|m| m.to_dawn_u32()).unwrap_or(0),
                                d.aspect.map(|m| m.to_dawn_u32()).unwrap_or(0),
                                d.format.map(|m| m.to_dawn_u32()).unwrap_or(0),
                                d.base_mip_level.unwrap_or(0) as i32,
                                d.mip_level_count.map(|v| v as i32).unwrap_or(-1),
                                d.base_array_layer.unwrap_or(0) as i32,
                                d.array_layer_count.map(|v| v as i32).unwrap_or(-1),
                            ),
                        };
                    match backend {
                        GpuBackend::NativeGpu => {
                            let handle = {
                                let gpu = caller.data_mut().require_native_gpu()?;
                                let texture =
                                    gpu.resolve_texture(texture_rep).map_err(native_gpu_error)?;
                                gpu.create_texture_view(
                                    texture,
                                    dimension,
                                    aspect,
                                    format,
                                    base_mip,
                                    mip_count,
                                    base_layer,
                                    layer_count,
                                )
                                .map_err(native_gpu_error)?
                            };
                            let resource = caller
                                .data_mut()
                                .table
                                .push(GpuTextureView { rep: handle.raw() })?;
                            Ok((resource,))
                        }
                        GpuBackend::JniBackend => {
                            let cb = caller.data().require_webgpu_jni_cb()?;
                            let l2_texture = if texture_rep == 0 {
                                let adapter_rep =
                                    jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                                let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                                    .map_err(wasmtime::Error::msg)?;
                                jvm::exp_create_texture(&cb, device_rep)
                                    .map_err(wasmtime::Error::msg)?
                            } else {
                                texture_rep
                            };
                            let view_rep = jvm::exp_texture_create_view_described(
                                &cb,
                                l2_texture,
                                dimension,
                                aspect,
                                format,
                                base_mip,
                                mip_count,
                                base_layer,
                                layer_count,
                            )
                            .map_err(wasmtime::Error::msg)?;
                            if view_rep == 0 {
                                return Err(wasmtime::Error::msg("texture-create-view returned 0"));
                            }
                            let resource = caller
                                .data_mut()
                                .table
                                .push(GpuTextureView { rep: view_rep })?;
                            Ok((resource,))
                        }
                    }
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-texture-view", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuTextureView { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture-view.label",
                |mut caller, (view,): (Resource<GpuTextureView>,)| {
                    let view_rep = caller.data_mut().table.get(&view)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if view_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let texture_rep = jvm::exp_create_texture(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_texture_create_view_described(
                            &cb,
                            texture_rep,
                            0,
                            0,
                            0,
                            0,
                            -1,
                            0,
                            -1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        view_rep
                    };
                    let label = jvm::exp_texture_view_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture-view.set-label",
                |mut caller, (view, label): (Resource<GpuTextureView>, String)| {
                    let view_rep = caller.data_mut().table.get(&view)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if view_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let texture_rep = jvm::exp_create_texture(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_texture_create_view_described(
                            &cb,
                            texture_rep,
                            0,
                            0,
                            0,
                            0,
                            -1,
                            0,
                            -1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        view_rep
                    };
                    jvm::exp_texture_view_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.destroy",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .destroy_rep(crate::native_gpu::ResourceKind::Texture, texture_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    jvm::exp_texture_destroy_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.width",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let w = caller
                            .data_mut()
                            .require_native_gpu()?
                            .texture_meta(texture_rep)
                            .map_err(native_gpu_error)?
                            .width;
                        return Ok((w,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let width = jvm::exp_texture_width_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((width,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.height",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let h = caller
                            .data_mut()
                            .require_native_gpu()?
                            .texture_meta(texture_rep)
                            .map_err(native_gpu_error)?
                            .height;
                        return Ok((h,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let height = jvm::exp_texture_height_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((height,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.depth-or-array-layers",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((1u32,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let depth = jvm::exp_texture_depth_or_array_layers_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((depth,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.mip-level-count",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((1u32,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let mip = jvm::exp_texture_mip_level_count_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((mip,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.sample-count",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((1u32,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let sample = jvm::exp_texture_sample_count_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((sample,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.dimension",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((GpuTextureDimension::D2,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let dawn = jvm::exp_texture_dimension_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuTextureDimension::from_dawn_u32(dawn),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.format",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((GpuTextureFormat::Rgba8unorm,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let dawn = jvm::exp_texture_format_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuTextureFormat::from_dawn_u32(dawn),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.usage",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((GpuTextureUsage::from_webgpu_u32(0),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let bits = jvm::exp_texture_usage_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuTextureUsage::from_webgpu_u32(bits),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.texture-binding-view-dimension",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((GpuTextureViewDimension::from_dawn_u32(2),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let dawn = jvm::exp_texture_binding_view_dimension_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuTextureViewDimension::from_dawn_u32(dawn),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.label",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let label =
                        jvm::exp_texture_label_described(&cb, l2).map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.set-label",
                |mut caller, (texture, label): (Resource<GpuTexture>, String)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    jvm::exp_texture_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-buffer",
                ResourceType::host::<GpuBuffer>(),
                |mut store, rep| {
                    let resource = Resource::<GpuBuffer>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-buffer", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuBuffer { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.size",
                |mut caller, (buffer,): (Resource<GpuBuffer>,)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let size = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            gpu.buffer_size(buffer_rep).map_err(native_gpu_error)?
                        };
                        return Ok((size,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    let size = jvm::exp_buffer_size_described(&cb, l2_buffer)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((size,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.usage",
                |mut caller, (buffer,): (Resource<GpuBuffer>,)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let bits = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            gpu.buffer_usage(buffer_rep).map_err(native_gpu_error)?
                        };
                        return Ok((GpuBufferUsage::from_webgpu_u32(bits),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    let bits = jvm::exp_buffer_usage_described(&cb, l2_buffer)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuBufferUsage::from_webgpu_u32(bits),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.map-state",
                |mut caller, (buffer,): (Resource<GpuBuffer>,)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let mapped = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            gpu.buffer_mapped(buffer_rep).map_err(native_gpu_error)?
                        };
                        return Ok((if mapped {
                            GpuBufferMapState::Mapped
                        } else {
                            GpuBufferMapState::Unmapped
                        },));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    let state = jvm::exp_buffer_map_state_described(&cb, l2_buffer)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuBufferMapState::from_host_u32(state),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.label",
                |mut caller, (buffer,): (Resource<GpuBuffer>,)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    let label = jvm::exp_buffer_label_described(&cb, l2_buffer)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.set-label",
                |mut caller, (buffer, label): (Resource<GpuBuffer>, String)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    jvm::exp_buffer_set_label_described(&cb, l2_buffer, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu-buffer.map-async",
                |accessor,
                 (buffer, mode, offset, size): (
                    Resource<GpuBuffer>,
                    GpuMapMode,
                    Option<u64>,
                    Option<u64>,
                )| {
                    Box::pin(async move {
                        let native = accessor.with(|mut access| -> wasmtime::Result<bool> {
                            let _ = access.data_mut().table.get(&buffer)?;
                            Ok(access.data_mut().webgpu_backend() == GpuBackend::NativeGpu)
                        })?;
                        if native {
                            let (tx, rx) = oneshot::channel::<()>();
                            std::thread::spawn(move || {
                                let _ = tx.send(());
                            });
                            let _ = rx.await;
                            accessor.with(|mut access| -> wasmtime::Result<()> {
                                let buffer_rep = access.data_mut().table.get(&buffer)?.rep;
                                let gpu = access.data_mut().require_native_gpu()?;
                                gpu.buffer_map_async_range(
                                    buffer_rep,
                                    mode.to_webgpu_u32(),
                                    offset.unwrap_or(0),
                                    size.unwrap_or(0),
                                )
                                .map_err(native_gpu_error)?;
                                Ok(())
                            })?;
                            return Ok((Ok::<(), MapAsyncError>(()),));
                        }
                        let (cb, buffer_rep) =
                            accessor.with(|mut access| -> wasmtime::Result<_> {
                                let buffer_rep = access.data_mut().table.get(&buffer)?.rep;
                                let cb = access.data_mut().require_webgpu_jni_cb()?;
                                Ok((cb, buffer_rep))
                            })?;
                        let (tx, rx) = oneshot::channel::<()>();
                        std::thread::spawn(move || {
                            let _ = tx.send(());
                        });
                        let _ = rx.await;
                        let l2_buffer = if buffer_rep == 0 {
                            let adapter_rep =
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                            let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                                .map_err(wasmtime::Error::msg)?;
                            jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                        } else {
                            buffer_rep
                        };
                        jvm::exp_buffer_map_async_described(
                            &cb,
                            l2_buffer,
                            mode.to_webgpu_u32(),
                            offset.unwrap_or(0),
                            size.unwrap_or(4),
                        )
                        .map_err(wasmtime::Error::msg)?;
                        Ok((Ok::<(), MapAsyncError>(()),))
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.unmap",
                |mut caller, (buffer,): (Resource<GpuBuffer>,)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            gpu.buffer_unmap(buffer_rep).map_err(native_gpu_error)?;
                        }
                        return Ok((Ok::<(), UnmapError>(()),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    jvm::exp_buffer_unmap_described(&cb, l2_buffer)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((Ok::<(), UnmapError>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.get-mapped-range-get-with-copy",
                |mut caller, (buffer, offset, size): (
                    Resource<GpuBuffer>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let data = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            let _ = (offset, size);
                            gpu.buffer_mapped_range(buffer_rep).map_err(native_gpu_error)?
                        };
                        return Ok((Ok::<Vec<u8>, GetMappedRangeError>(data),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    let data = jvm::exp_buffer_get_mapped_range_described(
                        &cb,
                        l2_buffer,
                        offset.unwrap_or(0),
                        size.unwrap_or(4),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((Ok::<Vec<u8>, GetMappedRangeError>(data),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.get-mapped-range-set-with-copy",
                |mut caller,
                 (buffer, data, offset, size): (
                    Resource<GpuBuffer>,
                    Vec<u8>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            let _ = (offset, size);
                            gpu.buffer_set_mapped_range(buffer_rep, data)
                                .map_err(native_gpu_error)?;
                        }
                        return Ok((Ok::<(), GetMappedRangeError>(()),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    let _ = size;
                    jvm::exp_buffer_set_mapped_range_described(
                        &cb,
                        l2_buffer,
                        data,
                        offset.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((Ok::<(), GetMappedRangeError>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.destroy",
                |mut caller, (buffer,): (Resource<GpuBuffer>,)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .destroy_rep(crate::native_gpu::ResourceKind::Buffer, buffer_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    jvm::exp_buffer_destroy_described(&cb, l2_buffer)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-sampler",
                ResourceType::host::<GpuSampler>(),
                |mut store, rep| {
                    let resource = Resource::<GpuSampler>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-sampler",
                |mut caller, (device, descriptor): (
                    Resource<GpuDevice>,
                    Option<GpuSamplerDescriptor>,
                )| {
                    let backend = caller.data().webgpu_backend();
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let (
                        mag_filter,
                        min_filter,
                        address_mode_u,
                        address_mode_v,
                        address_mode_w,
                        mipmap_filter,
                        compare,
                        has_lod_min,
                        lod_min,
                        has_lod_max,
                        lod_max,
                    ) = match &descriptor {
                        None => (0, 0, 0, 0, 0, 0, 0, 0, 0.0, 0, 0.0),
                        Some(d) => (
                            d.mag_filter.map(|m| m.to_dawn_u32()).unwrap_or(0),
                            d.min_filter.map(|m| m.to_dawn_u32()).unwrap_or(0),
                            d.address_mode_u.map(|m| m.to_dawn_u32()).unwrap_or(0),
                            d.address_mode_v.map(|m| m.to_dawn_u32()).unwrap_or(0),
                            d.address_mode_w.map(|m| m.to_dawn_u32()).unwrap_or(0),
                            d.mipmap_filter
                                .map(|m| match m {
                                    GpuMipmapFilterMode::Nearest => 1u32,
                                    GpuMipmapFilterMode::Linear => 2,
                                })
                                .unwrap_or(0),
                            d.compare
                                .map(|c| match c {
                                    GpuCompareFunction::Never => 1u32,
                                    GpuCompareFunction::Less => 2,
                                    GpuCompareFunction::Equal => 3,
                                    GpuCompareFunction::LessEqual => 4,
                                    GpuCompareFunction::Greater => 5,
                                    GpuCompareFunction::NotEqual => 6,
                                    GpuCompareFunction::GreaterEqual => 7,
                                    GpuCompareFunction::Always => 8,
                                })
                                .unwrap_or(0),
                            if d.lod_min_clamp.is_some() { 1i32 } else { 0 },
                            d.lod_min_clamp.unwrap_or(0.0),
                            if d.lod_max_clamp.is_some() { 1i32 } else { 0 },
                            d.lod_max_clamp.unwrap_or(0.0),
                        ),
                    };
                    match backend {
                        GpuBackend::NativeGpu => {
                            let handle = {
                                let gpu = caller.data_mut().require_native_gpu()?;
                                let device = gpu
                                    .resolve_device(device_rep)
                                    .map_err(native_gpu_error)?;
                                gpu.create_sampler(
                                    device,
                                    mag_filter,
                                    min_filter,
                                    address_mode_u,
                                    address_mode_v,
                                    address_mode_w,
                                    mipmap_filter,
                                    compare,
                                    has_lod_min,
                                    lod_min,
                                    has_lod_max,
                                    lod_max,
                                )
                                .map_err(native_gpu_error)?
                            };
                            let resource = caller
                                .data_mut()
                                .table
                                .push(GpuSampler { rep: handle.raw() })?;
                            Ok((resource,))
                        }
                        GpuBackend::JniBackend => {
                            let cb = caller.data().require_webgpu_jni_cb()?;
                            let l2_device = if device_rep == 0 {
                                let adapter_rep =
                                    jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                                jvm::exp_adapter_request_device(&cb, adapter_rep)
                                    .map_err(wasmtime::Error::msg)?
                            } else {
                                device_rep
                            };
                            let sampler_rep = jvm::exp_create_sampler_described(
                                &cb,
                                l2_device,
                                mag_filter,
                                min_filter,
                                address_mode_u,
                                address_mode_v,
                                address_mode_w,
                                mipmap_filter,
                                compare,
                                has_lod_min,
                                lod_min,
                                has_lod_max,
                                lod_max,
                            )
                            .map_err(wasmtime::Error::msg)?;
                            if sampler_rep == 0 {
                                return Err(wasmtime::Error::msg(
                                    "device-create-sampler returned 0",
                                ));
                            }
                            let resource = caller
                                .data_mut()
                                .table
                                .push(GpuSampler { rep: sampler_rep })?;
                            Ok((resource,))
                        }
                    }
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-sampler", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuSampler { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-sampler.label",
                |mut caller, (sampler,): (Resource<GpuSampler>,)| {
                    let sampler_rep = caller.data_mut().table.get(&sampler)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if sampler_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_sampler_described(
                            &cb, device_rep, 0, 0, 0, 0, 0, 0, 0, 0, 0.0, 0, 0.0,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        sampler_rep
                    };
                    let label =
                        jvm::exp_sampler_label_described(&cb, l2).map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-sampler.set-label",
                |mut caller, (sampler, label): (Resource<GpuSampler>, String)| {
                    let sampler_rep = caller.data_mut().table.get(&sampler)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if sampler_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_sampler_described(
                            &cb, device_rep, 0, 0, 0, 0, 0, 0, 0, 0, 0.0, 0, 0.0,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        sampler_rep
                    };
                    jvm::exp_sampler_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-pipeline-layout",
                ResourceType::host::<GpuPipelineLayout>(),
                |mut store, rep| {
                    let resource = Resource::<GpuPipelineLayout>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-shader-module",
                ResourceType::host::<GpuShaderModule>(),
                |mut store, rep| {
                    let resource = Resource::<GpuShaderModule>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-shader-module",
                |mut caller, (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuShaderModuleDescriptor,
                )| {
                    let backend = caller.data().webgpu_backend();
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let code = descriptor.code;
                    let label = descriptor.label.clone().unwrap_or_default();
                    let mut hint_layouts = Vec::new();
                    let mut hint_entries = String::new();
                    if let Some(hints) = &descriptor.compilation_hints {
                        for (i, h) in hints.iter().enumerate() {
                            if i > 0 {
                                hint_entries.push('\n');
                            }
                            hint_entries.push_str(&h.entry_point);
                            let layout = match &h.layout {
                                None => -1,
                                Some(GpuLayoutMode::Auto) => 0,
                                Some(GpuLayoutMode::Specific(layout)) => layout.rep() as i32,
                            };
                            hint_layouts.push(layout);
                        }
                    }
                    match backend {
                        GpuBackend::NativeGpu => {
                            let handle = {
                                let gpu = caller.data_mut().require_native_gpu()?;
                                let device = gpu
                                    .resolve_device(device_rep)
                                    .map_err(native_gpu_error)?;
                                gpu.create_shader_module(
                                    device,
                                    &code,
                                    &label,
                                    &hint_layouts,
                                    &hint_entries,
                                )
                                .map_err(native_gpu_error)?
                            };
                            let resource = caller
                                .data_mut()
                                .table
                                .push(GpuShaderModule { rep: handle.raw() })?;
                            Ok((resource,))
                        }
                        GpuBackend::JniBackend => {
                            let cb = caller.data().require_webgpu_jni_cb()?;
                            let l2_device = if device_rep == 0 {
                                let adapter_rep =
                                    jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                                jvm::exp_adapter_request_device(&cb, adapter_rep)
                                    .map_err(wasmtime::Error::msg)?
                            } else {
                                device_rep
                            };
                            let shader_rep = jvm::exp_create_shader_module_described(
                                &cb,
                                l2_device,
                                code,
                                label,
                                hint_layouts,
                                hint_entries,
                            )
                            .map_err(wasmtime::Error::msg)?;
                            if shader_rep == 0 {
                                return Err(wasmtime::Error::msg(
                                    "device-create-shader-module returned 0",
                                ));
                            }
                            let resource = caller
                                .data_mut()
                                .table
                                .push(GpuShaderModule { rep: shader_rep })?;
                            Ok((resource,))
                        }
                    }
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-shader-module", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuShaderModule { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu-shader-module.get-compilation-info",
                |accessor, (shader,): (Resource<GpuShaderModule>,)| {
                    Box::pin(async move {
                        let native = accessor.with(|mut access| -> wasmtime::Result<bool> {
                            let _ = access.data_mut().table.get(&shader)?;
                            Ok(access.data_mut().webgpu_backend() == GpuBackend::NativeGpu)
                        })?;
                        if native {
                            let (tx, rx) = oneshot::channel::<()>();
                            std::thread::spawn(move || {
                                let _ = tx.send(());
                            });
                            let _ = rx.await;
                            let resource = accessor.with(|mut access| -> wasmtime::Result<_> {
                                let shader_rep = access.data_mut().table.get(&shader)?.rep;
                                let gpu = access.data_mut().require_native_gpu()?;
                                let _ = gpu.resolve_shader(shader_rep).map_err(native_gpu_error)?;
                                Ok(access.data_mut().table.push(GpuCompilationInfo {
                                    shader_module: shader_rep,
                                })?)
                            })?;
                            return Ok((resource,));
                        }
                        let (cb, shader_rep) =
                            accessor.with(|mut access| -> wasmtime::Result<_> {
                                let shader_rep = access.data_mut().table.get(&shader)?.rep;
                                let cb = access.data_mut().require_webgpu_jni_cb()?;
                                Ok((cb, shader_rep))
                            })?;
                        let l2_shader = if shader_rep == 0 {
                            let adapter_rep =
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                            let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                                .map_err(wasmtime::Error::msg)?;
                            jvm::exp_create_shader_module(&cb, device_rep)
                                .map_err(wasmtime::Error::msg)?
                        } else {
                            shader_rep
                        };
                        jvm::exp_shader_module_get_compilation_info_described(&cb, l2_shader)
                            .map_err(wasmtime::Error::msg)?;
                        let resource = accessor.with(|mut access| {
                            access.data_mut().table.push(GpuCompilationInfo {
                                shader_module: l2_shader,
                            })
                        })?;
                        Ok((resource,))
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-shader-module.label",
                |mut caller, (shader,): (Resource<GpuShaderModule>,)| {
                    let shader_rep = caller.data_mut().table.get(&shader)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if shader_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        shader_rep
                    };
                    let label = jvm::exp_shader_module_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-shader-module.set-label",
                |mut caller, (shader, label): (Resource<GpuShaderModule>, String)| {
                    let shader_rep = caller.data_mut().table.get(&shader)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if shader_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        shader_rep
                    };
                    jvm::exp_shader_module_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "record-gpu-pipeline-constant-value",
                ResourceType::host::<RecordGpuPipelineConstantValue>(),
                |mut store, rep| {
                    let resource = Resource::<RecordGpuPipelineConstantValue>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[constructor]record-gpu-pipeline-constant-value",
                |mut store, ()| {
                    let resource = store
                        .data_mut()
                        .table
                        .push(RecordGpuPipelineConstantValue)?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.add",
                |mut caller,
                 (record, key, value): (
                    Resource<RecordGpuPipelineConstantValue>,
                    String,
                    f64,
                )| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller.data_mut().require_native_gpu()?.pipeline_constant_add(
                            handle,
                            key,
                            value,
                        );
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    jvm::exp_record_pipeline_constant_value_add_described(&cb, handle, key, value)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.get",
                |mut caller, (record, key): (Resource<RecordGpuPipelineConstantValue>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let value = caller
                            .data_mut()
                            .require_native_gpu()?
                            .pipeline_constant_get(handle, &key);
                        return Ok((value,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let has = jvm::exp_record_pipeline_constant_value_has_described(
                        &cb,
                        handle,
                        key.clone(),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if has == 0 {
                        return Ok((None,));
                    }
                    let value = jvm::exp_record_pipeline_constant_value_get_value_described(
                        &cb, handle, key,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((Some(value),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.has",
                |mut caller, (record, key): (Resource<RecordGpuPipelineConstantValue>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let has = caller
                            .data_mut()
                            .require_native_gpu()?
                            .pipeline_constant_has(handle, &key);
                        return Ok((has,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let has =
                        jvm::exp_record_pipeline_constant_value_has_described(&cb, handle, key)
                            .map_err(wasmtime::Error::msg)?;
                    Ok((has != 0,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.remove",
                |mut caller, (record, key): (Resource<RecordGpuPipelineConstantValue>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .pipeline_constant_remove(handle, &key);
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    jvm::exp_record_pipeline_constant_value_remove_described(&cb, handle, key)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.keys",
                |mut caller, (record,): (Resource<RecordGpuPipelineConstantValue>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let keys = caller
                            .data_mut()
                            .require_native_gpu()?
                            .pipeline_constant_keys(handle);
                        return Ok((keys,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let count =
                        jvm::exp_record_pipeline_constant_value_keys_count_described(&cb, handle)
                            .map_err(wasmtime::Error::msg)?;
                    let mut keys = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        keys.push(
                            jvm::exp_record_pipeline_constant_value_keys_get_described(
                                &cb, handle, i,
                            )
                            .map_err(wasmtime::Error::msg)?,
                        );
                    }
                    Ok((keys,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.values",
                |mut caller, (record,): (Resource<RecordGpuPipelineConstantValue>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let values = caller
                            .data_mut()
                            .require_native_gpu()?
                            .pipeline_constant_values(handle);
                        return Ok((values,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let count =
                        jvm::exp_record_pipeline_constant_value_values_count_described(&cb, handle)
                            .map_err(wasmtime::Error::msg)?;
                    let mut values = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        values.push(
                            jvm::exp_record_pipeline_constant_value_values_get_described(
                                &cb, handle, i,
                            )
                            .map_err(wasmtime::Error::msg)?,
                        );
                    }
                    Ok((values,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.entries",
                |mut caller, (record,): (Resource<RecordGpuPipelineConstantValue>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let entries = caller
                            .data_mut()
                            .require_native_gpu()?
                            .pipeline_constant_entries(handle);
                        return Ok((entries,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let count = jvm::exp_record_pipeline_constant_value_entries_count_described(
                        &cb, handle,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    let mut entries = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        let key =
                            jvm::exp_record_pipeline_constant_value_entries_get_key_described(
                                &cb, handle, i,
                            )
                            .map_err(wasmtime::Error::msg)?;
                        let value =
                            jvm::exp_record_pipeline_constant_value_entries_get_value_described(
                                &cb, handle, i,
                            )
                            .map_err(wasmtime::Error::msg)?;
                        entries.push((key, value));
                    }
                    Ok((entries,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-bind-group-layout",
                ResourceType::host::<GpuBindGroupLayout>(),
                |mut store, rep| {
                    let resource = Resource::<GpuBindGroupLayout>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-bind-group-layout", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuBindGroupLayout { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-bind-group-layout.label",
                |mut caller, (layout,): (Resource<GpuBindGroupLayout>,)| {
                    let layout_rep = caller.data_mut().table.get(&layout)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if layout_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_bind_group_layout(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        layout_rep
                    };
                    let label = jvm::exp_bind_group_layout_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-bind-group-layout.set-label",
                |mut caller, (layout, label): (Resource<GpuBindGroupLayout>, String)| {
                    let layout_rep = caller.data_mut().table.get(&layout)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if layout_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_bind_group_layout(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        layout_rep
                    };
                    jvm::exp_bind_group_layout_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-bind-group-layout",
                |mut caller, (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuBindGroupLayoutDescriptor,
                )| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let mut bindings = Vec::with_capacity(descriptor.entries.len());
                    let mut visibilities = Vec::with_capacity(descriptor.entries.len());
                    let mut buffer_types = Vec::with_capacity(descriptor.entries.len());
                    let mut sampler_types = Vec::with_capacity(descriptor.entries.len());
                    let mut texture_sample_types = Vec::with_capacity(descriptor.entries.len());
                    for entry in &descriptor.entries {
                        bindings.push(entry.binding as i32);
                        let mut visibility = 0i32;
                        if entry.visibility.contains(GpuShaderStage::VERTEX) {
                            visibility |= 1;
                        }
                        if entry.visibility.contains(GpuShaderStage::FRAGMENT) {
                            visibility |= 2;
                        }
                        if entry.visibility.contains(GpuShaderStage::COMPUTE) {
                            visibility |= 4;
                        }
                        visibilities.push(visibility);
                        buffer_types.push(match entry.buffer.as_ref().and_then(|b| b.ty) {
                            Some(GpuBufferBindingType::Uniform) => 0,
                            Some(GpuBufferBindingType::Storage) => 1,
                            Some(GpuBufferBindingType::ReadOnlyStorage) => 2,
                            None => -1,
                        });
                        sampler_types.push(match &entry.sampler {
                            None => -1,
                            Some(sampler) => match sampler.ty {
                                Some(GpuSamplerBindingType::NonFiltering) => 1,
                                Some(GpuSamplerBindingType::Comparison) => 2,
                                Some(GpuSamplerBindingType::Filtering) | None => 0,
                            },
                        });
                        texture_sample_types.push(match &entry.texture {
                            None => -1,
                            Some(texture) => match texture.sample_type {
                                Some(GpuTextureSampleType::UnfilterableFloat) => 1,
                                Some(GpuTextureSampleType::Depth) => 2,
                                Some(GpuTextureSampleType::Sint) => 3,
                                Some(GpuTextureSampleType::Uint) => 4,
                                Some(GpuTextureSampleType::Float) | None => 0,
                            },
                        });
                    }
                    let cb = if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let handle = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            let device = gpu
                                .resolve_device(device_rep)
                                .map_err(native_gpu_error)?;
                            gpu.create_bind_group_layout(
                                device,
                                &bindings,
                                &visibilities,
                                &buffer_types,
                                &sampler_types,
                                &texture_sample_types,
                            )
                            .map_err(native_gpu_error)?
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuBindGroupLayout { rep: handle.raw() })?;
                        return Ok((resource,));
                    } else {
                        caller.data().require_webgpu_jni_cb()?
                    };
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let layout_rep = jvm::exp_create_bind_group_layout_described(
                        &cb,
                        l2_device,
                        bindings,
                        visibilities,
                        buffer_types,
                        sampler_types,
                        texture_sample_types,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if layout_rep == 0 {
                        return Err(wasmtime::Error::msg(
                            "device-create-bind-group-layout returned 0",
                        ));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuBindGroupLayout { rep: layout_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-pipeline-layout",
                |mut caller, (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuPipelineLayoutDescriptor,
                )| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let mut layouts = Vec::with_capacity(descriptor.bind_group_layouts.len());
                    for opt in &descriptor.bind_group_layouts {
                        match opt {
                            Some(layout) => {
                                layouts.push(caller.data_mut().table.get(layout)?.rep as i32);
                            }
                            None => layouts.push(0),
                        }
                    }
                    let label = descriptor.label.clone().unwrap_or_default();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let mut native_layouts = layouts.clone();
                        let handle = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            let device = gpu
                                .resolve_device(device_rep)
                                .map_err(native_gpu_error)?;
                            for slot in native_layouts.iter_mut() {
                                if *slot == 0 {
                                    *slot = gpu
                                        .resolve_bind_group_layout(0)
                                        .map_err(native_gpu_error)?
                                        .raw() as i32;
                                }
                            }
                            gpu.create_pipeline_layout(device, &native_layouts, &label)
                                .map_err(native_gpu_error)?
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuPipelineLayout { rep: handle.raw() })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let mut l2_layouts = Vec::with_capacity(layouts.len());
                    for layout_rep in layouts {
                        if layout_rep != 0 {
                            l2_layouts.push(layout_rep);
                            continue;
                        }
                        l2_layouts.push(
                            jvm::exp_create_bind_group_layout(&cb, l2_device)
                                .map_err(wasmtime::Error::msg)? as i32,
                        );
                    }
                    let layout_rep = jvm::exp_create_pipeline_layout_described(
                        &cb, l2_device, l2_layouts, label,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if layout_rep == 0 {
                        return Err(wasmtime::Error::msg(
                            "device-create-pipeline-layout returned 0",
                        ));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuPipelineLayout { rep: layout_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-pipeline-layout", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuPipelineLayout { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-pipeline-layout.label",
                |mut caller, (layout,): (Resource<GpuPipelineLayout>,)| {
                    let layout_rep = caller.data_mut().table.get(&layout)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if layout_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_pipeline_layout(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        layout_rep
                    };
                    let label = jvm::exp_pipeline_layout_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-pipeline-layout.set-label",
                |mut caller, (layout, label): (Resource<GpuPipelineLayout>, String)| {
                    let layout_rep = caller.data_mut().table.get(&layout)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if layout_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_pipeline_layout(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        layout_rep
                    };
                    jvm::exp_pipeline_layout_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-bind-group",
                ResourceType::host::<GpuBindGroup>(),
                |mut store, rep| {
                    let resource = Resource::<GpuBindGroup>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-bind-group",
                |mut caller, (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuBindGroupDescriptor,
                )| {
                    let backend = caller.data().webgpu_backend();
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let layout_rep = caller.data_mut().table.get(&descriptor.layout)?.rep;
                    let label = descriptor.label.clone().unwrap_or_default();
                    let mut bindings = Vec::with_capacity(descriptor.entries.len());
                    let mut kinds = Vec::with_capacity(descriptor.entries.len());
                    let mut handles = Vec::with_capacity(descriptor.entries.len());
                    for entry in &descriptor.entries {
                        bindings.push(entry.binding as i32);
                        let (kind, raw) = match &entry.resource {
                            GpuBindingResource::GpuBuffer(buffer) => {
                                (0, caller.data_mut().table.get(buffer)?.rep)
                            }
                            GpuBindingResource::GpuBufferBinding(binding) => {
                                (0, caller.data_mut().table.get(&binding.buffer)?.rep)
                            }
                            GpuBindingResource::GpuSampler(sampler) => {
                                (1, caller.data_mut().table.get(sampler)?.rep)
                            }
                            GpuBindingResource::GpuTexture(texture) => {
                                (2, caller.data_mut().table.get(texture)?.rep)
                            }
                            GpuBindingResource::GpuTextureView(view) => {
                                (2, caller.data_mut().table.get(view)?.rep)
                            }
                        };
                        kinds.push(kind);
                        handles.push(raw as i32);
                    }
                    if backend == GpuBackend::NativeGpu {
                        let handle = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            let device = gpu
                                .resolve_device(device_rep)
                                .map_err(native_gpu_error)?;
                            let layout = gpu
                                .resolve_bind_group_layout(layout_rep)
                                .map_err(native_gpu_error)?;
                            for (kind, raw) in kinds.iter().zip(handles.iter_mut()) {
                                if *kind == 0 && *raw == 0 {
                                    *raw = gpu.resolve_buffer(0).map_err(native_gpu_error)?.raw()
                                        as i32;
                                }
                            }
                            gpu.create_bind_group(
                                device, layout, &label, &bindings, &kinds, &handles,
                            )
                            .map_err(native_gpu_error)?
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuBindGroup { rep: handle.raw() })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let l2_layout = if layout_rep == 0 {
                        jvm::exp_create_bind_group_layout(&cb, l2_device)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        layout_rep
                    };
                    let mut jni_handles = Vec::with_capacity(handles.len());
                    for (kind, raw) in kinds.iter().zip(handles.iter()) {
                        let handle = if *raw != 0 {
                            *raw as u32
                        } else if *kind == 0 {
                            jvm::exp_create_buffer(&cb, l2_device)
                                .map_err(wasmtime::Error::msg)?
                        } else {
                            *raw as u32
                        };
                        jni_handles.push(handle as i32);
                    }
                    let bg_rep = jvm::exp_create_bind_group_described(
                        &cb, l2_device, l2_layout, label, bindings, kinds, jni_handles,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if bg_rep == 0 {
                        return Err(wasmtime::Error::msg("device-create-bind-group returned 0"));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuBindGroup { rep: bg_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-bind-group", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuBindGroup { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-bind-group.label",
                |mut caller, (bind_group,): (Resource<GpuBindGroup>,)| {
                    let bind_group_rep = caller.data_mut().table.get(&bind_group)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if bind_group_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_bind_group(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        bind_group_rep
                    };
                    let label = jvm::exp_bind_group_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-bind-group.set-label",
                |mut caller, (bind_group, label): (Resource<GpuBindGroup>, String)| {
                    let bind_group_rep = caller.data_mut().table.get(&bind_group)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if bind_group_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_bind_group(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        bind_group_rep
                    };
                    jvm::exp_bind_group_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-render-pipeline",
                ResourceType::host::<GpuRenderPipeline>(),
                |mut store, rep| {
                    let resource = Resource::<GpuRenderPipeline>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-render-pipeline", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuRenderPipeline { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pipeline.label",
                |mut caller, (pipeline,): (Resource<GpuRenderPipeline>,)| {
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if pipeline_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_pipeline(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pipeline_rep
                    };
                    let label = jvm::exp_render_pipeline_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pipeline.set-label",
                |mut caller, (pipeline, label): (Resource<GpuRenderPipeline>, String)| {
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if pipeline_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_pipeline(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pipeline_rep
                    };
                    jvm::exp_render_pipeline_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pipeline.get-bind-group-layout",
                |mut caller, (pipeline, index): (Resource<GpuRenderPipeline>, u32)| {
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = index;
                        let handle = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            gpu.pipeline_bind_group_layout(pipeline_rep)
                                .map_err(native_gpu_error)?
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuBindGroupLayout { rep: handle.raw() })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let layout_rep = jvm::exp_render_pipeline_get_bind_group_layout_described(
                        &cb,
                        pipeline_rep,
                        index,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuBindGroupLayout { rep: layout_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-compute-pipeline",
                ResourceType::host::<GpuComputePipeline>(),
                |mut store, rep| {
                    let resource = Resource::<GpuComputePipeline>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-compute-pipeline", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuComputePipeline { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pipeline.label",
                |mut caller, (pipeline,): (Resource<GpuComputePipeline>,)| {
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if pipeline_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_compute_pipeline(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pipeline_rep
                    };
                    let label = jvm::exp_compute_pipeline_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pipeline.set-label",
                |mut caller, (pipeline, label): (Resource<GpuComputePipeline>, String)| {
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if pipeline_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_compute_pipeline(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pipeline_rep
                    };
                    jvm::exp_compute_pipeline_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pipeline.get-bind-group-layout",
                |mut caller, (pipeline, index): (Resource<GpuComputePipeline>, u32)| {
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = index;
                        let handle = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            gpu.pipeline_bind_group_layout(pipeline_rep)
                                .map_err(native_gpu_error)?
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuBindGroupLayout { rep: handle.raw() })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let layout_rep = jvm::exp_compute_pipeline_get_bind_group_layout_described(
                        &cb,
                        pipeline_rep,
                        index,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuBindGroupLayout { rep: layout_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-render-pipeline",
                |mut caller, (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuRenderPipelineDescriptor,
                )| {
                    let vertex_shader = caller
                        .data_mut()
                        .table
                        .get(&descriptor.vertex.module)?
                        .rep;
                    let vertex_entry = descriptor.vertex.entry_point.clone().unwrap_or_default();
                    let (fragment_shader, fragment_entry) = match &descriptor.fragment {
                        Some(fragment) => {
                            let fs = caller.data_mut().table.get(&fragment.module)?.rep as i32;
                            (fs, fragment.entry_point.clone().unwrap_or_default())
                        }
                        None => (0, String::new()),
                    };
                    let layout_rep = match &descriptor.layout {
                        GpuLayoutMode::Specific(layout) => {
                            caller.data_mut().table.get(layout)?.rep as i32
                        }
                        GpuLayoutMode::Auto => 0,
                    };
                    let format = first_fragment_target_format(&descriptor.fragment);
                    let (
                        vb_strides,
                        vb_step_modes,
                        attr_index,
                        attr_formats,
                        attr_offsets,
                        attr_locations,
                    ) = pack_vertex_buffers(&descriptor.vertex.buffers);
                    let label = descriptor.label.clone().unwrap_or_default();
                    let vertex_constants = pipeline_constant_rep(&descriptor.vertex.constants);
                    let fragment_constants = match &descriptor.fragment {
                        Some(fragment) => pipeline_constant_rep(&fragment.constants),
                        None => 0,
                    };
                    let (primitive, multisample, blend, write_mask, depth_stencil) =
                        pack_render_pipeline_semantics(&descriptor);
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let handle = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            let device = gpu
                                .resolve_device(device_rep)
                                .map_err(native_gpu_error)?;
                            gpu.create_render_pipeline_described(
                                device,
                                vertex_shader,
                                &vertex_entry,
                                fragment_shader,
                                &fragment_entry,
                                format,
                                layout_rep,
                                &label,
                                vertex_constants,
                                fragment_constants,
                                &vb_strides,
                                &vb_step_modes,
                                &attr_index,
                                &attr_formats,
                                &attr_offsets,
                                &attr_locations,
                                &primitive,
                                &multisample,
                                &blend,
                                &write_mask,
                                &depth_stencil,
                            )
                            .map_err(native_gpu_error)?
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuRenderPipeline { rep: handle.raw() })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let pipeline_rep = jvm::exp_create_render_pipeline_described(
                        &cb,
                        l2_device,
                        vertex_shader,
                        vertex_entry,
                        fragment_shader,
                        fragment_entry,
                        format,
                        layout_rep,
                        label,
                        vb_strides,
                        vb_step_modes,
                        attr_index,
                        attr_formats,
                        attr_offsets,
                        attr_locations,
                        vertex_constants,
                        fragment_constants,
                        primitive,
                        multisample,
                        blend,
                        write_mask,
                        depth_stencil,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if pipeline_rep == 0 {
                        return Err(wasmtime::Error::msg(
                            "device-create-render-pipeline returned 0",
                        ));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuRenderPipeline { rep: pipeline_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-compute-pipeline",
                |mut caller, (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuComputePipelineDescriptor,
                )| {
                    let shader_rep = caller
                        .data_mut()
                        .table
                        .get(&descriptor.compute.module)?
                        .rep;
                    let layout_rep = match &descriptor.layout {
                        GpuLayoutMode::Specific(layout) => {
                            caller.data_mut().table.get(layout)?.rep as i32
                        }
                        GpuLayoutMode::Auto => 0,
                    };
                    let entry_point = descriptor.compute.entry_point.clone().unwrap_or_default();
                    let label = descriptor.label.clone().unwrap_or_default();
                    let constants = pipeline_constant_rep(&descriptor.compute.constants);
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let handle = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            let device = gpu
                                .resolve_device(device_rep)
                                .map_err(native_gpu_error)?;
                            gpu.create_compute_pipeline(
                                device,
                                shader_rep,
                                &entry_point,
                                layout_rep,
                                &label,
                                constants,
                            )
                            .map_err(native_gpu_error)?
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuComputePipeline { rep: handle.raw() })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let pipeline_rep = jvm::exp_create_compute_pipeline_described(
                        &cb,
                        l2_device,
                        shader_rep,
                        entry_point,
                        layout_rep,
                        label,
                        constants,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if pipeline_rep == 0 {
                        return Err(wasmtime::Error::msg(
                            "device-create-compute-pipeline returned 0",
                        ));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuComputePipeline { rep: pipeline_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu-device.create-render-pipeline-async",
                |accessor, (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuRenderPipelineDescriptor,
                )| {
                    Box::pin(async move {
                        let (
                            backend,
                            device_rep,
                            vertex_shader,
                            vertex_entry,
                            fragment_shader,
                            fragment_entry,
                            format,
                            layout_rep,
                            label,
                            vb_strides,
                            vb_step_modes,
                            attr_index,
                            attr_formats,
                            attr_offsets,
                            attr_locations,
                            vertex_constants,
                            fragment_constants,
                            primitive,
                            multisample,
                            blend,
                            write_mask,
                            depth_stencil,
                        ) = accessor.with(|mut access| -> wasmtime::Result<_> {
                            let vertex_shader = access
                                .data_mut()
                                .table
                                .get(&descriptor.vertex.module)?
                                .rep;
                            let vertex_entry =
                                descriptor.vertex.entry_point.clone().unwrap_or_default();
                            let (fragment_shader, fragment_entry) = match &descriptor.fragment {
                                Some(fragment) => {
                                    let fs =
                                        access.data_mut().table.get(&fragment.module)?.rep as i32;
                                    (fs, fragment.entry_point.clone().unwrap_or_default())
                                }
                                None => (0, String::new()),
                            };
                            let layout_rep = match &descriptor.layout {
                                GpuLayoutMode::Specific(layout) => {
                                    access.data_mut().table.get(layout)?.rep as i32
                                }
                                GpuLayoutMode::Auto => 0,
                            };
                            let format = first_fragment_target_format(&descriptor.fragment);
                            let packed = pack_vertex_buffers(&descriptor.vertex.buffers);
                            let label = descriptor.label.clone().unwrap_or_default();
                            let vertex_constants =
                                pipeline_constant_rep(&descriptor.vertex.constants);
                            let fragment_constants = match &descriptor.fragment {
                                Some(fragment) => pipeline_constant_rep(&fragment.constants),
                                None => 0,
                            };
                            let (primitive, multisample, blend, write_mask, depth_stencil) =
                                pack_render_pipeline_semantics(&descriptor);
                            let device_rep = access.data_mut().table.get(&device)?.rep;
                            let backend = access.data_mut().webgpu_backend();
                            Ok((
                                backend,
                                device_rep,
                                vertex_shader,
                                vertex_entry,
                                fragment_shader,
                                fragment_entry,
                                format,
                                layout_rep,
                                label,
                                packed.0,
                                packed.1,
                                packed.2,
                                packed.3,
                                packed.4,
                                packed.5,
                                vertex_constants,
                                fragment_constants,
                                primitive,
                                multisample,
                                blend,
                                write_mask,
                                depth_stencil,
                            ))
                        })?;
                        let (tx, rx) = oneshot::channel::<()>();
                        std::thread::spawn(move || {
                            let _ = tx.send(());
                        });
                        let _ = rx.await;
                        match backend {
                            GpuBackend::NativeGpu => {
                                let resource =
                                    accessor.with(|mut access| -> wasmtime::Result<_> {
                                        let handle = {
                                            let gpu = access.data_mut().require_native_gpu()?;
                                            let device = gpu
                                                .resolve_device(device_rep)
                                                .map_err(native_gpu_error)?;
                                            gpu.create_render_pipeline_described(
                                                device,
                                                vertex_shader,
                                                &vertex_entry,
                                                fragment_shader,
                                                &fragment_entry,
                                                format,
                                                layout_rep,
                                                &label,
                                                vertex_constants,
                                                fragment_constants,
                                                &vb_strides,
                                                &vb_step_modes,
                                                &attr_index,
                                                &attr_formats,
                                                &attr_offsets,
                                                &attr_locations,
                                                &primitive,
                                                &multisample,
                                                &blend,
                                                &write_mask,
                                                &depth_stencil,
                                            )
                                            .map_err(native_gpu_error)?
                                        };
                                        Ok(access.data_mut().table.push(GpuRenderPipeline {
                                            rep: handle.raw(),
                                        })?)
                                    })?;
                                Ok((Ok(resource),))
                            }
                            GpuBackend::JniBackend => {
                                let cb = accessor.with(|mut access| {
                                    access.data_mut().require_webgpu_jni_cb()
                                })?;
                                let l2_device = if device_rep == 0 {
                                    let adapter_rep = jvm::exp_request_adapter(&cb)
                                        .map_err(wasmtime::Error::msg)?;
                                    jvm::exp_adapter_request_device(&cb, adapter_rep)
                                        .map_err(wasmtime::Error::msg)?
                                } else {
                                    device_rep
                                };
                                let pipeline_rep = jvm::exp_create_render_pipeline_described(
                                    &cb,
                                    l2_device,
                                    vertex_shader,
                                    vertex_entry,
                                    fragment_shader,
                                    fragment_entry,
                                    format,
                                    layout_rep,
                                    label,
                                    vb_strides,
                                    vb_step_modes,
                                    attr_index,
                                    attr_formats,
                                    attr_offsets,
                                    attr_locations,
                                    vertex_constants,
                                    fragment_constants,
                                    primitive,
                                    multisample,
                                    blend,
                                    write_mask,
                                    depth_stencil,
                                )
                                .map_err(wasmtime::Error::msg)?;
                                if pipeline_rep == 0 {
                                    return Ok((Err(CreatePipelineError {
                                        kind: CreatePipelineErrorKind::GpuPipelineError(
                                            GpuPipelineErrorReason::Internal,
                                        ),
                                        message: "device-create-render-pipeline returned 0"
                                            .into(),
                                    }),));
                                }
                                let resource = accessor.with(|mut access| {
                                    access
                                        .data_mut()
                                        .table
                                        .push(GpuRenderPipeline { rep: pipeline_rep })
                                })?;
                                Ok((Ok(resource),))
                            }
                        }
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu-device.create-compute-pipeline-async",
                |accessor, (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuComputePipelineDescriptor,
                )| {
                    Box::pin(async move {
                        let (
                            backend,
                            device_rep,
                            shader_rep,
                            entry_point,
                            layout_rep,
                            label,
                            constants,
                        ) = accessor.with(|mut access| -> wasmtime::Result<_> {
                            let shader_rep =
                                access.data_mut().table.get(&descriptor.compute.module)?.rep;
                            let layout_rep = match &descriptor.layout {
                                GpuLayoutMode::Specific(layout) => {
                                    access.data_mut().table.get(layout)?.rep as i32
                                }
                                GpuLayoutMode::Auto => 0,
                            };
                            let entry_point =
                                descriptor.compute.entry_point.clone().unwrap_or_default();
                            let label = descriptor.label.clone().unwrap_or_default();
                            let constants = pipeline_constant_rep(&descriptor.compute.constants);
                            let device_rep = access.data_mut().table.get(&device)?.rep;
                            let backend = access.data_mut().webgpu_backend();
                            Ok((
                                backend,
                                device_rep,
                                shader_rep,
                                entry_point,
                                layout_rep,
                                label,
                                constants,
                            ))
                        })?;
                        let (tx, rx) = oneshot::channel::<()>();
                        std::thread::spawn(move || {
                            let _ = tx.send(());
                        });
                        let _ = rx.await;
                        match backend {
                            GpuBackend::NativeGpu => {
                                let resource =
                                    accessor.with(|mut access| -> wasmtime::Result<_> {
                                        let handle = {
                                            let gpu = access.data_mut().require_native_gpu()?;
                                            let device = gpu
                                                .resolve_device(device_rep)
                                                .map_err(native_gpu_error)?;
                                            gpu.create_compute_pipeline(
                                                device,
                                                shader_rep,
                                                &entry_point,
                                                layout_rep,
                                                &label,
                                                constants,
                                            )
                                            .map_err(native_gpu_error)?
                                        };
                                        Ok(access.data_mut().table.push(GpuComputePipeline {
                                            rep: handle.raw(),
                                        })?)
                                    })?;
                                Ok((Ok(resource),))
                            }
                            GpuBackend::JniBackend => {
                                let cb = accessor.with(|mut access| {
                                    access.data_mut().require_webgpu_jni_cb()
                                })?;
                                let l2_device = if device_rep == 0 {
                                    let adapter_rep = jvm::exp_request_adapter(&cb)
                                        .map_err(wasmtime::Error::msg)?;
                                    jvm::exp_adapter_request_device(&cb, adapter_rep)
                                        .map_err(wasmtime::Error::msg)?
                                } else {
                                    device_rep
                                };
                                let pipeline_rep = jvm::exp_create_compute_pipeline_described(
                                    &cb,
                                    l2_device,
                                    shader_rep,
                                    entry_point,
                                    layout_rep,
                                    label,
                                    constants,
                                )
                                .map_err(wasmtime::Error::msg)?;
                                if pipeline_rep == 0 {
                                    return Ok((Err(CreatePipelineError {
                                        kind: CreatePipelineErrorKind::GpuPipelineError(
                                            GpuPipelineErrorReason::Internal,
                                        ),
                                        message: "device-create-compute-pipeline returned 0"
                                            .into(),
                                    }),));
                                }
                                let resource = accessor.with(|mut access| {
                                    access
                                        .data_mut()
                                        .table
                                        .push(GpuComputePipeline { rep: pipeline_rep })
                                })?;
                                Ok((Ok(resource),))
                            }
                        }
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-encoder", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuCommandEncoder { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.label",
                |mut caller, (encoder,): (Resource<GpuCommandEncoder>,)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    let label = jvm::exp_command_encoder_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.set-label",
                |mut caller, (encoder, label): (Resource<GpuCommandEncoder>, String)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_command_encoder_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-query-set",
                ResourceType::host::<GpuQuerySet>(),
                |mut store, rep| {
                    let resource = Resource::<GpuQuerySet>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-query-set", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuQuerySet { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-query-set",
                |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuQuerySetDescriptor)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let handle = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            let device =
                                gpu.resolve_device(device_rep).map_err(native_gpu_error)?;
                            gpu.create_query_set(
                                device,
                                descriptor.type_.to_host_u32(),
                                descriptor.count,
                            )
                            .map_err(native_gpu_error)?
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuQuerySet { rep: handle.raw() })?;
                        return Ok((Ok::<_, CreateQuerySetError>(resource),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let query_rep = jvm::exp_create_query_set_described(
                        &cb,
                        l2_device,
                        descriptor.type_.to_host_u32(),
                        descriptor.count,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuQuerySet { rep: query_rep })?;
                    Ok((Ok::<_, CreateQuerySetError>(resource),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-query-set.destroy",
                |mut caller, (query_set,): (Resource<GpuQuerySet>,)| {
                    let query_rep = caller.data_mut().table.get(&query_set)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let gpu = caller.data_mut().require_native_gpu()?;
                        let query = gpu.resolve_query_set(query_rep).map_err(native_gpu_error)?;
                        gpu.query_set_destroy(query).map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_query = if query_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_query_set(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        query_rep
                    };
                    jvm::exp_query_set_destroy_described(&cb, l2_query)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-query-set.type",
                |mut caller, (query_set,): (Resource<GpuQuerySet>,)| {
                    let query_rep = caller.data_mut().table.get(&query_set)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let gpu = caller.data_mut().require_native_gpu()?;
                        let query = gpu.resolve_query_set(query_rep).map_err(native_gpu_error)?;
                        let ty = gpu.query_set_type(query).map_err(native_gpu_error)?;
                        return Ok((GpuQueryType::from_host_u32(ty),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_query = if query_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_query_set(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        query_rep
                    };
                    let ty = jvm::exp_query_set_type_described(&cb, l2_query)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuQueryType::from_host_u32(ty),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-query-set.count",
                |mut caller, (query_set,): (Resource<GpuQuerySet>,)| {
                    let query_rep = caller.data_mut().table.get(&query_set)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let gpu = caller.data_mut().require_native_gpu()?;
                        let query = gpu.resolve_query_set(query_rep).map_err(native_gpu_error)?;
                        let count = gpu.query_set_count(query).map_err(native_gpu_error)?;
                        return Ok((count,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_query = if query_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_query_set(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        query_rep
                    };
                    let count = jvm::exp_query_set_count_described(&cb, l2_query)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((count,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-query-set.label",
                |mut caller, (query_set,): (Resource<GpuQuerySet>,)| {
                    let query_set_rep = caller.data_mut().table.get(&query_set)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if query_set_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_query_set(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        query_set_rep
                    };
                    let label = jvm::exp_query_set_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-query-set.set-label",
                |mut caller, (query_set, label): (Resource<GpuQuerySet>, String)| {
                    let query_set_rep = caller.data_mut().table.get(&query_set)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if query_set_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_query_set(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        query_set_rep
                    };
                    jvm::exp_query_set_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-render-pass-encoder",
                ResourceType::host::<GpuRenderPassEncoder>(),
                |mut store, rep| {
                    let resource = Resource::<GpuRenderPassEncoder>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.begin-render-pass",
                |mut caller, (encoder, descriptor): (Resource<GpuCommandEncoder>, GpuRenderPassDescriptor)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let mut color_views = Vec::new();
                    let mut color_loads = Vec::new();
                    let mut color_stores = Vec::new();
                    let mut color_has_clears = Vec::new();
                    let mut color_clear_bits = Vec::new();
                    for att in &descriptor.color_attachments {
                        match att {
                            None => {
                                color_views.push(0);
                                color_loads.push(0);
                                color_stores.push(0);
                                color_has_clears.push(0);
                                color_clear_bits.extend_from_slice(&[0, 0, 0, 0]);
                            }
                            Some(att) => {
                                let view_rep = caller.data_mut().table.get(&att.view)?.rep as i32;
                                color_views.push(view_rep);
                                color_loads.push(att.load_op.to_dawn_u32() as i32);
                                color_stores.push(att.store_op.to_dawn_u32() as i32);
                                match &att.clear_value {
                                    Some(c) => {
                                        color_has_clears.push(1);
                                        color_clear_bits.extend_from_slice(&pack_color_clear_bits(c));
                                    }
                                    None => {
                                        color_has_clears.push(0);
                                        color_clear_bits.extend_from_slice(&[0, 0, 0, 0]);
                                    }
                                }
                            }
                        }
                    }
                    let (depth_view, depth_load, depth_store, has_depth_clear, depth_clear) =
                        match &descriptor.depth_stencil_attachment {
                            Some(ds) => {
                                let view = caller.data_mut().table.get(&ds.view)?.rep;
                                let load = ds
                                    .depth_load_op
                                    .map(|op| op.to_dawn_u32() as i32)
                                    .unwrap_or(-1);
                                let store = ds
                                    .depth_store_op
                                    .map(|op| op.to_dawn_u32() as i32)
                                    .unwrap_or(-1);
                                let (has, v) = match ds.depth_clear_value {
                                    Some(c) => (1, c),
                                    None => (0, 1.0),
                                };
                                (view, load, store, has, v)
                            }
                            None => (0, -1, -1, 0, 1.0),
                        };
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let handle = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            let encoder =
                                gpu.resolve_encoder(encoder_rep).map_err(native_gpu_error)?;
                            gpu.begin_render_pass_described(
                                encoder,
                                &color_views,
                                &color_loads,
                                &color_stores,
                                &color_has_clears,
                                &color_clear_bits,
                                depth_view,
                            )
                            .map_err(native_gpu_error)?
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuRenderPassEncoder { rep: handle.raw() })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    let pass_rep = jvm::exp_begin_render_pass_described(
                        &cb,
                        l2_encoder,
                        color_views,
                        color_loads,
                        color_stores,
                        color_has_clears,
                        color_clear_bits,
                        depth_view,
                        depth_load,
                        depth_store,
                        has_depth_clear,
                        depth_clear,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if pass_rep == 0 {
                        return Err(wasmtime::Error::msg("begin-render-pass returned 0"));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuRenderPassEncoder { rep: pass_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-compute-pass-encoder",
                ResourceType::host::<GpuComputePassEncoder>(),
                |mut store, rep| {
                    let resource = Resource::<GpuComputePassEncoder>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.begin-compute-pass",
                |mut caller,
                 (encoder, descriptor): (
                    Resource<GpuCommandEncoder>,
                    Option<GpuComputePassDescriptor>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let (begin_idx, end_idx, query_rep) = match descriptor
                        .as_ref()
                        .and_then(|d| d.timestamp_writes.as_ref())
                    {
                        Some(ts) => {
                            let begin_idx = ts.beginning_of_pass_write_index.unwrap_or(0);
                            let end_idx = ts.end_of_pass_write_index.unwrap_or(0);
                            let query_rep = caller.data_mut().table.get(&ts.query_set)?.rep;
                            (begin_idx, end_idx, query_rep)
                        }
                        None => (0, 0, 0),
                    };
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let handle = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            let encoder =
                                gpu.resolve_encoder(encoder_rep).map_err(native_gpu_error)?;
                            gpu.begin_compute_pass(encoder, query_rep, begin_idx, end_idx)
                                .map_err(native_gpu_error)?
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuComputePassEncoder { rep: handle.raw() })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    let pass_rep =
                        jvm::exp_begin_compute_pass_described(&cb, l2_encoder, begin_idx, end_idx)
                            .map_err(wasmtime::Error::msg)?;
                    if pass_rep == 0 {
                        return Err(wasmtime::Error::msg("begin-compute-pass returned 0"));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuComputePassEncoder { rep: pass_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.copy-buffer-to-buffer",
                |mut caller,
                 (encoder, source, source_offset, destination, destination_offset, size): (
                    Resource<GpuCommandEncoder>,
                    Resource<GpuBuffer>,
                    Option<u64>,
                    Resource<GpuBuffer>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let source_rep = caller.data_mut().table.get(&source)?.rep;
                    let dest_rep = caller.data_mut().table.get(&destination)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .encoder_copy_sized(
                                encoder_rep,
                                Some(source_rep),
                                Some(dest_rep),
                                None,
                                None,
                                source_offset.unwrap_or(0),
                                destination_offset.unwrap_or(0),
                                size.unwrap_or(0),
                                1,
                                1,
                                1,
                            )
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_copy_buffer_to_buffer_described(
                        &cb,
                        l2_encoder,
                        source_rep,
                        source_offset.unwrap_or(0),
                        dest_rep,
                        destination_offset.unwrap_or(0),
                        size.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.copy-buffer-to-texture",
                |mut caller,
                 (encoder, source, destination, copy_size): (
                    Resource<GpuCommandEncoder>,
                    GpuTexelCopyBufferInfo,
                    GpuTexelCopyTextureInfo,
                    GpuExtent3D,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let source_rep = caller.data_mut().table.get(&source.buffer)?.rep;
                    let dest_rep = caller.data_mut().table.get(&destination.texture)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .encoder_copy_sized(
                                encoder_rep,
                                Some(source_rep),
                                None,
                                None,
                                Some(dest_rep),
                                0,
                                0,
                                0,
                                copy_size.width,
                                copy_size.height.unwrap_or(1),
                                copy_size.depth_or_array_layers.unwrap_or(1),
                            )
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_copy_buffer_to_texture_described(
                        &cb,
                        l2_encoder,
                        source_rep,
                        dest_rep,
                        copy_size.width,
                        copy_size.height.unwrap_or(1),
                        copy_size.depth_or_array_layers.unwrap_or(1),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.copy-texture-to-buffer",
                |mut caller,
                 (encoder, source, destination, copy_size): (
                    Resource<GpuCommandEncoder>,
                    GpuTexelCopyTextureInfo,
                    GpuTexelCopyBufferInfo,
                    GpuExtent3D,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let source_rep = caller.data_mut().table.get(&source.texture)?.rep;
                    let dest_rep = caller.data_mut().table.get(&destination.buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .encoder_copy_sized(
                                encoder_rep,
                                None,
                                Some(dest_rep),
                                Some(source_rep),
                                None,
                                0,
                                0,
                                0,
                                copy_size.width,
                                copy_size.height.unwrap_or(1),
                                copy_size.depth_or_array_layers.unwrap_or(1),
                            )
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_copy_texture_to_buffer_described(
                        &cb,
                        l2_encoder,
                        source_rep,
                        dest_rep,
                        copy_size.width,
                        copy_size.height.unwrap_or(1),
                        copy_size.depth_or_array_layers.unwrap_or(1),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.copy-texture-to-texture",
                |mut caller,
                 (encoder, source, destination, copy_size): (
                    Resource<GpuCommandEncoder>,
                    GpuTexelCopyTextureInfo,
                    GpuTexelCopyTextureInfo,
                    GpuExtent3D,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let source_rep = caller.data_mut().table.get(&source.texture)?.rep;
                    let dest_rep = caller.data_mut().table.get(&destination.texture)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .encoder_copy_sized(
                                encoder_rep,
                                None,
                                None,
                                Some(source_rep),
                                Some(dest_rep),
                                0,
                                0,
                                0,
                                copy_size.width,
                                copy_size.height.unwrap_or(1),
                                copy_size.depth_or_array_layers.unwrap_or(1),
                            )
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_copy_texture_to_texture_described(
                        &cb,
                        l2_encoder,
                        source_rep,
                        dest_rep,
                        copy_size.width,
                        copy_size.height.unwrap_or(1),
                        copy_size.depth_or_array_layers.unwrap_or(1),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.clear-buffer",
                |mut caller,
                 (encoder, buffer, offset, size): (
                    Resource<GpuCommandEncoder>,
                    Resource<GpuBuffer>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .encoder_clear_buffer_range(
                                encoder_rep,
                                buffer_rep,
                                offset.unwrap_or(0),
                                size.unwrap_or(0),
                            )
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_clear_buffer_described(
                        &cb,
                        l2_encoder,
                        buffer_rep,
                        offset.unwrap_or(0),
                        size.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.resolve-query-set",
                |mut caller,
                 (
                    encoder,
                    query_set,
                    first_query,
                    query_count,
                    destination,
                    destination_offset,
                ): (
                    Resource<GpuCommandEncoder>,
                    Resource<GpuQuerySet>,
                    u32,
                    u32,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let query_rep = caller.data_mut().table.get(&query_set)?.rep;
                    let dest_rep = caller.data_mut().table.get(&destination)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .encoder_resolve_query_set_range(
                                encoder_rep,
                                query_rep,
                                first_query,
                                query_count,
                                dest_rep,
                                destination_offset,
                            )
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_resolve_query_set_described(
                        &cb,
                        l2_encoder,
                        query_rep,
                        first_query,
                        query_count,
                        dest_rep,
                        destination_offset,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.push-debug-group",
                |mut caller, (encoder, group_label): (Resource<GpuCommandEncoder>, String)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .encoder_debug(encoder_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_push_debug_group_described(&cb, l2_encoder, group_label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.pop-debug-group",
                |mut caller, (encoder,): (Resource<GpuCommandEncoder>,)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .encoder_debug(encoder_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_pop_debug_group_described(&cb, l2_encoder)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.insert-debug-marker",
                |mut caller, (encoder, marker_label): (Resource<GpuCommandEncoder>, String)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .encoder_debug(encoder_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_insert_debug_marker_described(&cb, l2_encoder, marker_label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-command-buffer",
                ResourceType::host::<GpuCommandBuffer>(),
                |mut store, rep| {
                    let resource = Resource::<GpuCommandBuffer>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.finish",
                |mut caller,
                 (encoder, descriptor): (
                    Resource<GpuCommandEncoder>,
                    Option<GpuCommandBufferDescriptor>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let label = descriptor
                        .as_ref()
                        .and_then(|d| d.label.clone())
                        .unwrap_or_default();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let handle = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            let encoder =
                                gpu.resolve_encoder(encoder_rep).map_err(native_gpu_error)?;
                            gpu.encoder_finish(encoder, &label)
                                .map_err(native_gpu_error)?
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuCommandBuffer { rep: handle.raw() })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    let buffer_rep =
                        jvm::exp_command_encoder_finish_described(&cb, l2_encoder, label)
                            .map_err(wasmtime::Error::msg)?;
                    if buffer_rep == 0 {
                        return Err(wasmtime::Error::msg("command-encoder-finish returned 0"));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuCommandBuffer { rep: buffer_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-queue", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuQueue { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-command-buffer", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuCommandBuffer { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-buffer.label",
                |mut caller, (buffer,): (Resource<GpuCommandBuffer>,)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_command_encoder_finish(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    let label = jvm::exp_command_buffer_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-buffer.set-label",
                |mut caller, (buffer, label): (Resource<GpuCommandBuffer>, String)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_command_encoder_finish(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    jvm::exp_command_buffer_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-compilation-message",
                ResourceType::host::<GpuCompilationMessage>(),
                |mut store, rep| {
                    let resource = Resource::<GpuCompilationMessage>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-compilation-info",
                ResourceType::host::<GpuCompilationInfo>(),
                |mut store, rep| {
                    let resource = Resource::<GpuCompilationInfo>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-compilation-info", |mut store, ()| {
                let resource = store
                    .data_mut()
                    .table
                    .push(GpuCompilationInfo { shader_module: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-compilation-message", |mut store, ()| {
                let resource = store
                    .data_mut()
                    .table
                    .push(GpuCompilationMessage { shader_module: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-info.messages",
                |mut caller, (info,): (Resource<GpuCompilationInfo>,)| {
                    let info_shader = caller.data_mut().table.get(&info)?.shader_module;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((Vec::<Resource<GpuCompilationMessage>>::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_shader = if info_shader == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        info_shader
                    };
                    let count = jvm::exp_compilation_info_messages_count_described(&cb, l2_shader)
                        .map_err(wasmtime::Error::msg)?;
                    let mut messages = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        messages.push(caller.data_mut().table.push(GpuCompilationMessage {
                            shader_module: l2_shader,
                        })?);
                    }
                    Ok((messages,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.message",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let msg_shader = caller.data_mut().table.get(&msg)?.shader_module;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_shader = if msg_shader == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        msg_shader
                    };
                    let message = jvm::exp_compilation_message_message_described(&cb, l2_shader)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((message,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.type",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let msg_shader = caller.data_mut().table.get(&msg)?.shader_module;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((GpuCompilationMessageType::from_host_u32(0),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_shader = if msg_shader == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        msg_shader
                    };
                    let ty = jvm::exp_compilation_message_type_described(&cb, l2_shader)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuCompilationMessageType::from_host_u32(ty),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.line-num",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let msg_shader = caller.data_mut().table.get(&msg)?.shader_module;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((0u64,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_shader = if msg_shader == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        msg_shader
                    };
                    let line_num = jvm::exp_compilation_message_line_num_described(&cb, l2_shader)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((line_num,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.line-pos",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let msg_shader = caller.data_mut().table.get(&msg)?.shader_module;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((0u64,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_shader = if msg_shader == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        msg_shader
                    };
                    let line_pos = jvm::exp_compilation_message_line_pos_described(&cb, l2_shader)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((line_pos,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.offset",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let msg_shader = caller.data_mut().table.get(&msg)?.shader_module;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((0u64,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_shader = if msg_shader == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        msg_shader
                    };
                    let offset = jvm::exp_compilation_message_offset_described(&cb, l2_shader)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((offset,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.length",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let msg_shader = caller.data_mut().table.get(&msg)?.shader_module;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((0u64,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_shader = if msg_shader == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        msg_shader
                    };
                    let length = jvm::exp_compilation_message_length_described(&cb, l2_shader)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((length,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-queue.label",
                |mut caller, (queue,): (Resource<GpuQueue>,)| {
                    let queue_rep = caller.data_mut().table.get(&queue)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if queue_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_device_get_queue(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        queue_rep
                    };
                    let label =
                        jvm::exp_queue_label_described(&cb, l2).map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-queue.set-label",
                |mut caller, (queue, label): (Resource<GpuQueue>, String)| {
                    let queue_rep = caller.data_mut().table.get(&queue)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if queue_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_device_get_queue(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        queue_rep
                    };
                    jvm::exp_queue_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu-queue.on-submitted-work-done",
                |accessor, (queue,): (Resource<GpuQueue>,)| {
                    Box::pin(async move {
                        let (backend, queue_rep) =
                            accessor.with(|mut access| -> wasmtime::Result<_> {
                                let queue_rep = access.data_mut().table.get(&queue)?.rep;
                                let backend = access.data_mut().webgpu_backend();
                                Ok((backend, queue_rep))
                            })?;
                        let (tx, rx) = oneshot::channel::<()>();
                        std::thread::spawn(move || {
                            let _ = tx.send(());
                        });
                        let _ = rx.await;
                        match backend {
                            GpuBackend::NativeGpu => {
                                accessor.with(|mut access| -> wasmtime::Result<_> {
                                    access
                                        .data_mut()
                                        .require_native_gpu()?
                                        .on_submitted_work_done(queue_rep)
                                        .map_err(native_gpu_error)
                                })?;
                                Ok(())
                            }
                            GpuBackend::JniBackend => {
                                let cb = accessor
                                    .with(|mut access| access.data_mut().require_webgpu_jni_cb())?;
                                let l2_queue = if queue_rep == 0 {
                                    let adapter_rep = jvm::exp_request_adapter(&cb)
                                        .map_err(wasmtime::Error::msg)?;
                                    let device_rep =
                                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                                            .map_err(wasmtime::Error::msg)?;
                                    jvm::exp_device_get_queue(&cb, device_rep)
                                        .map_err(wasmtime::Error::msg)?
                                } else {
                                    queue_rep
                                };
                                jvm::exp_queue_on_submitted_work_done_described(&cb, l2_queue)
                                    .map_err(wasmtime::Error::msg)?;
                                Ok(())
                            }
                        }
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-queue.submit",
                |mut caller, (queue, commands): (
                    Resource<GpuQueue>,
                    Vec<Resource<GpuCommandBuffer>>,
                )| {
                    let queue_rep = caller.data_mut().table.get(&queue)?.rep;
                    let mut command_reps = Vec::with_capacity(commands.len());
                    for command in &commands {
                        command_reps.push(caller.data_mut().table.get(command)?.rep);
                    }
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .queue_submit(queue_rep, &command_reps)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let mut device_rep = 0u32;
                    let l2_queue = if queue_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_device_get_queue(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        queue_rep
                    };
                    let mut l2_commands = Vec::with_capacity(command_reps.len());
                    for command_rep in command_reps {
                        if command_rep != 0 {
                            l2_commands.push(command_rep as i32);
                            continue;
                        }
                        if device_rep == 0 {
                            let adapter_rep =
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                            device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                                .map_err(wasmtime::Error::msg)?;
                        }
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let finished = jvm::exp_command_encoder_finish(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?;
                        l2_commands.push(finished as i32);
                    }
                    jvm::exp_queue_submit_described(&cb, l2_queue, l2_commands)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-queue.write-buffer-with-copy",
                |mut caller,
                 (queue, buffer, offset, data, data_offset, size): (
                    Resource<GpuQueue>,
                    Resource<GpuBuffer>,
                    u64,
                    Vec<u8>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let queue_rep = caller.data_mut().table.get(&queue)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let start = data_offset.unwrap_or(0) as usize;
                    let copy_len = size
                        .map(|s| s as usize)
                        .unwrap_or_else(|| data.len().saturating_sub(start));
                    let payload = if start >= data.len() {
                        Vec::new()
                    } else {
                        let end = (start + copy_len).min(data.len());
                        if start == 0 && end == data.len() {
                            data
                        } else {
                            data[start..end].to_vec()
                        }
                    };
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .write_buffer_with_copy(queue_rep, buffer_rep, offset, payload)
                            .map_err(native_gpu_error)?;
                        return Ok((Ok::<(), WriteBufferError>(()),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let mut device_rep = 0u32;
                    let l2_queue = if queue_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_device_get_queue(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        queue_rep
                    };
                    let l2_buffer = if buffer_rep == 0 {
                        if device_rep == 0 {
                            let adapter_rep =
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                            device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                                .map_err(wasmtime::Error::msg)?;
                        }
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    jvm::exp_queue_write_buffer_described(
                        &cb, l2_queue, l2_buffer, offset, payload,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((Ok::<(), WriteBufferError>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-queue.write-texture-with-copy",
                |mut caller,
                 (queue, destination, data, layout, size): (
                    Resource<GpuQueue>,
                    GpuTexelCopyTextureInfo,
                    Vec<u8>,
                    GpuTexelCopyBufferLayout,
                    GpuExtent3D,
                )| {
                    let queue_rep = caller.data_mut().table.get(&queue)?.rep;
                    let texture_rep = caller.data_mut().table.get(&destination.texture)?.rep;
                    let start = layout.offset.unwrap_or(0) as usize;
                    let payload = if start >= data.len() {
                        Vec::new()
                    } else {
                        data[start..].to_vec()
                    };
                    let width = size.width.max(1);
                    let height = size.height.unwrap_or(1).max(1);
                    let bytes_per_row = layout
                        .bytes_per_row
                        .unwrap_or(width.saturating_mul(4))
                        .max(1);
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .write_texture_described(
                                queue_rep,
                                texture_rep,
                                payload,
                                bytes_per_row,
                                width,
                                height,
                                size.depth_or_array_layers.unwrap_or(1).max(1),
                            )
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let mut device_rep = 0u32;
                    let l2_queue = if queue_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_device_get_queue(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        queue_rep
                    };
                    let l2_texture = if texture_rep == 0 {
                        if device_rep == 0 {
                            let adapter_rep =
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                            device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                                .map_err(wasmtime::Error::msg)?;
                        }
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    jvm::exp_queue_write_texture_described(
                        &cb,
                        l2_queue,
                        l2_texture,
                        payload,
                        width,
                        height,
                        bytes_per_row,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-pass", |mut store, ()| {
                let resource = store
                    .data_mut()
                    .table
                    .push(GpuRenderPassEncoder { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.end",
                |mut caller, (pass,): (Resource<GpuRenderPassEncoder>,)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .render_pass_end(pass_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_end_described(&cb, l2_pass)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-pipeline",
                |mut caller,
                 (pass, pipeline): (
                    Resource<GpuRenderPassEncoder>,
                    Resource<GpuRenderPipeline>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .render_pass_set_pipeline(pass_rep, pipeline_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_set_pipeline_described(&cb, l2_pass, pipeline_rep)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.draw",
                |mut caller,
                 (pass, vertex_count, instance_count, first_vertex, first_instance): (
                    Resource<GpuRenderPassEncoder>,
                    u32,
                    Option<u32>,
                    Option<u32>,
                    Option<u32>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .render_pass_draw_counts(
                                pass_rep,
                                vertex_count,
                                instance_count.unwrap_or(1),
                                first_vertex.unwrap_or(0),
                                first_instance.unwrap_or(0),
                            )
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_draw_described(
                        &cb,
                        l2_pass,
                        vertex_count,
                        instance_count.unwrap_or(1),
                        first_vertex.unwrap_or(0),
                        first_instance.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-bind-group",
                |mut caller,
                 (pass, index, bind_group, _offsets, _start, _length): (
                    Resource<GpuRenderPassEncoder>,
                    u32,
                    Option<Resource<GpuBindGroup>>,
                    Option<Vec<u32>>,
                    Option<u64>,
                    Option<u32>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let bind_group_rep = match bind_group {
                        Some(ref g) => caller.data_mut().table.get(g)?.rep,
                        None => 0,
                    };
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .render_pass_set_bind_group(pass_rep, index, bind_group_rep)
                            .map_err(native_gpu_error)?;
                        return Ok((Ok::<(), SetBindGroupError>(()),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_set_bind_group_described(
                        &cb,
                        l2_pass,
                        index,
                        bind_group_rep,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((Ok::<(), SetBindGroupError>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-vertex-buffer",
                |mut caller,
                 (pass, slot, buffer, offset, size): (
                    Resource<GpuRenderPassEncoder>,
                    u32,
                    Option<Resource<GpuBuffer>>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let buffer_rep = match buffer {
                        Some(ref b) => caller.data_mut().table.get(b)?.rep,
                        None => 0,
                    };
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .render_pass_set_vertex_buffer(
                                pass_rep,
                                slot,
                                buffer_rep,
                                offset.unwrap_or(0),
                                size.unwrap_or(0),
                            )
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_set_vertex_buffer_described(
                        &cb,
                        l2_pass,
                        slot,
                        buffer_rep,
                        offset.unwrap_or(0),
                        size.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-viewport",
                |mut caller,
                 (pass, x, y, width, height, min_depth, max_depth): (
                    Resource<GpuRenderPassEncoder>,
                    f32,
                    f32,
                    f32,
                    f32,
                    f32,
                    f32,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .render_pass_set_viewport(
                                pass_rep, x, y, width, height, min_depth, max_depth,
                            )
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_set_viewport_described(
                        &cb, l2_pass, x, y, width, height, min_depth, max_depth,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-scissor-rect",
                |mut caller,
                 (pass, x, y, width, height): (
                    Resource<GpuRenderPassEncoder>,
                    u32,
                    u32,
                    u32,
                    u32,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .render_pass_set_scissor(pass_rep, x, y, width, height)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_set_scissor_rect_described(
                        &cb, l2_pass, x, y, width, height,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-blend-constant",
                |mut caller, (pass, color): (Resource<GpuRenderPassEncoder>, GpuColor)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .render_pass_set_blend_constant(
                                pass_rep, color.r, color.g, color.b, color.a,
                            )
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_set_blend_constant_described(
                        &cb, l2_pass, color.r, color.g, color.b, color.a,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-stencil-reference",
                |mut caller, (pass, reference): (Resource<GpuRenderPassEncoder>, u32)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .render_pass_set_stencil_reference(pass_rep, reference)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_set_stencil_reference_described(&cb, l2_pass, reference)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-index-buffer",
                |mut caller,
                 (pass, buffer, format, offset, size): (
                    Resource<GpuRenderPassEncoder>,
                    Resource<GpuBuffer>,
                    GpuIndexFormat,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let format_u32 = match format {
                        GpuIndexFormat::Uint16 => 1,
                        GpuIndexFormat::Uint32 => 2,
                    };
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .render_pass_set_index_buffer(
                                pass_rep,
                                buffer_rep,
                                format_u32,
                                offset.unwrap_or(0),
                                size.unwrap_or(0),
                            )
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_set_index_buffer_described(
                        &cb,
                        l2_pass,
                        buffer_rep,
                        format_u32,
                        offset.unwrap_or(0),
                        size.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.draw-indexed",
                |mut caller,
                 (
                    pass,
                    index_count,
                    instance_count,
                    first_index,
                    base_vertex,
                    first_instance,
                ): (
                    Resource<GpuRenderPassEncoder>,
                    u32,
                    Option<u32>,
                    Option<u32>,
                    Option<i32>,
                    Option<u32>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .render_pass_draw_indexed(
                                pass_rep,
                                index_count,
                                instance_count.unwrap_or(1),
                                first_index.unwrap_or(0),
                                base_vertex.unwrap_or(0),
                                first_instance.unwrap_or(0),
                            )
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_draw_indexed_described(
                        &cb,
                        l2_pass,
                        index_count,
                        instance_count.unwrap_or(1),
                        first_index.unwrap_or(0),
                        base_vertex.unwrap_or(0),
                        first_instance.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.draw-indirect",
                |mut caller,
                 (pass, buffer, offset): (
                    Resource<GpuRenderPassEncoder>,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .render_pass_draw_indirect(pass_rep, buffer_rep, offset)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_draw_indirect_described(&cb, l2_pass, buffer_rep, offset)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.draw-indexed-indirect",
                |mut caller,
                 (pass, buffer, offset): (
                    Resource<GpuRenderPassEncoder>,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .render_pass_draw_indexed_indirect(pass_rep, buffer_rep, offset)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_draw_indexed_indirect_described(
                        &cb, l2_pass, buffer_rep, offset,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.push-debug-group",
                |mut caller, (pass, group_label): (Resource<GpuRenderPassEncoder>, String)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .resolve_render_pass(pass_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_push_debug_group_described(&cb, l2_pass, group_label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.pop-debug-group",
                |mut caller, (pass,): (Resource<GpuRenderPassEncoder>,)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .resolve_render_pass(pass_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_pop_debug_group_described(&cb, l2_pass)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.insert-debug-marker",
                |mut caller, (pass, marker_label): (Resource<GpuRenderPassEncoder>, String)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .resolve_render_pass(pass_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_insert_debug_marker_described(&cb, l2_pass, marker_label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.begin-occlusion-query",
                |mut caller, (pass, query_index): (Resource<GpuRenderPassEncoder>, u32)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .resolve_render_pass(pass_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_begin_occlusion_query_described(&cb, l2_pass, query_index)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.end-occlusion-query",
                |mut caller, (pass,): (Resource<GpuRenderPassEncoder>,)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .resolve_render_pass(pass_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_end_occlusion_query_described(&cb, l2_pass)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.label",
                |mut caller, (pass,): (Resource<GpuRenderPassEncoder>,)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let texture_rep = jvm::exp_create_texture(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let view_rep = jvm::exp_texture_create_view_described(
                            &cb,
                            texture_rep,
                            0,
                            0,
                            0,
                            0,
                            -1,
                            0,
                            -1,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, view_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    let label = jvm::exp_render_pass_encoder_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-label",
                |mut caller, (pass, label): (Resource<GpuRenderPassEncoder>, String)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let texture_rep = jvm::exp_create_texture(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let view_rep = jvm::exp_texture_create_view_described(
                            &cb,
                            texture_rep,
                            0,
                            0,
                            0,
                            0,
                            -1,
                            0,
                            -1,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, view_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_encoder_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-render-bundle",
                ResourceType::host::<GpuRenderBundle>(),
                |mut store, rep| {
                    let resource = Resource::<GpuRenderBundle>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-render-bundle", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuRenderBundle { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle.label",
                |mut caller, (bundle,): (Resource<GpuRenderBundle>,)| {
                    let bundle_rep = caller.data_mut().table.get(&bundle)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if bundle_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_render_bundle_encoder_described(
                            &cb, device_rep, 0x16, 1,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        jvm::exp_render_bundle_encoder_finish_described(
                            &cb,
                            encoder_rep,
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        bundle_rep
                    };
                    let label = jvm::exp_render_bundle_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle.set-label",
                |mut caller, (bundle, label): (Resource<GpuRenderBundle>, String)| {
                    let bundle_rep = caller.data_mut().table.get(&bundle)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if bundle_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_render_bundle_encoder_described(
                            &cb, device_rep, 0x16, 1,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        jvm::exp_render_bundle_encoder_finish_described(
                            &cb,
                            encoder_rep,
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        bundle_rep
                    };
                    jvm::exp_render_bundle_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.execute-bundles",
                |mut caller,
                 (pass, bundles): (
                    Resource<GpuRenderPassEncoder>,
                    Vec<Resource<GpuRenderBundle>>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let mut bundle_reps: Vec<i32> = Vec::with_capacity(bundles.len());
                    for bundle in &bundles {
                        bundle_reps.push(caller.data_mut().table.get(bundle)?.rep as i32);
                    }
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .resolve_render_pass(pass_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_execute_bundles_described(&cb, l2_pass, bundle_reps)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-immediates",
                |mut caller,
                 (pass, range_offset, data, data_offset, data_size): (
                    Resource<GpuRenderPassEncoder>,
                    u32,
                    Vec<u8>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .resolve_render_pass(pass_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    let _ = data_size;
                    jvm::exp_render_pass_set_immediates_described(
                        &cb,
                        l2_pass,
                        range_offset,
                        data,
                        data_offset.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-render-bundle-encoder",
                ResourceType::host::<GpuRenderBundleEncoder>(),
                |mut store, rep| {
                    let resource = Resource::<GpuRenderBundleEncoder>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-render-bundle-encoder", |mut store, ()| {
                let resource = store
                    .data_mut()
                    .table
                    .push(GpuRenderBundleEncoder { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.label",
                |mut caller, (encoder,): (Resource<GpuRenderBundleEncoder>,)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(&cb, device_rep, 0x16, 1)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    let label = jvm::exp_render_bundle_encoder_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-label",
                |mut caller, (encoder, label): (Resource<GpuRenderBundleEncoder>, String)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(&cb, device_rep, 0x16, 1)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-render-bundle-encoder",
                |mut caller,
                 (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuRenderBundleEncoderDescriptor,
                )| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let color_format = descriptor
                        .color_formats
                        .iter()
                        .flatten()
                        .next()
                        .map(|f| f.to_dawn_u32())
                        .unwrap_or_else(|| GpuTextureFormat::Rgba8unorm.to_dawn_u32());
                    let sample_count = descriptor.sample_count.unwrap_or(1);
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = descriptor;
                        let handle = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            let device =
                                gpu.resolve_device(device_rep).map_err(native_gpu_error)?;
                            gpu.create_render_bundle_encoder(device)
                                .map_err(native_gpu_error)?
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuRenderBundleEncoder { rep: handle.raw() })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let encoder_rep = jvm::exp_create_render_bundle_encoder_described(
                        &cb,
                        l2_device,
                        color_format,
                        sample_count,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuRenderBundleEncoder { rep: encoder_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.finish",
                |mut caller,
                 (encoder, descriptor): (
                    Resource<GpuRenderBundleEncoder>,
                    Option<GpuRenderBundleDescriptor>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let label = descriptor
                        .as_ref()
                        .and_then(|d| d.label.clone())
                        .unwrap_or_default();
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        let handle = {
                            let gpu = caller.data_mut().require_native_gpu()?;
                            gpu.finish_render_bundle(encoder_rep)
                                .map_err(native_gpu_error)?
                        };
                        let resource = caller
                            .data_mut()
                            .table
                            .push(GpuRenderBundle { rep: handle.raw() })?;
                        return Ok((resource,));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    let bundle_rep =
                        jvm::exp_render_bundle_encoder_finish_described(&cb, l2_encoder, label)
                            .map_err(wasmtime::Error::msg)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuRenderBundle { rep: bundle_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-pipeline",
                |mut caller,
                 (encoder, pipeline): (
                    Resource<GpuRenderBundleEncoder>,
                    Resource<GpuRenderPipeline>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_set_pipeline_described(
                        &cb,
                        l2_encoder,
                        pipeline_rep,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-bind-group",
                |mut caller,
                 (encoder, index, bind_group, _offsets, _start, _length): (
                    Resource<GpuRenderBundleEncoder>,
                    u32,
                    Option<Resource<GpuBindGroup>>,
                    Option<Vec<u32>>,
                    Option<u64>,
                    Option<u32>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let bind_group_rep = match bind_group.as_ref() {
                        Some(bind_group) => caller.data_mut().table.get(bind_group)?.rep,
                        None => 0,
                    };
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((Ok::<(), SetBindGroupError>(()),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_set_bind_group_described(
                        &cb,
                        l2_encoder,
                        index,
                        bind_group_rep,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((Ok::<(), SetBindGroupError>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.draw",
                |mut caller,
                 (encoder, vertex_count, instance_count, first_vertex, first_instance): (
                    Resource<GpuRenderBundleEncoder>,
                    u32,
                    Option<u32>,
                    Option<u32>,
                    Option<u32>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_draw_described(
                        &cb,
                        l2_encoder,
                        vertex_count,
                        instance_count.unwrap_or(1),
                        first_vertex.unwrap_or(0),
                        first_instance.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-index-buffer",
                |mut caller,
                 (encoder, buffer, format, offset, size): (
                    Resource<GpuRenderBundleEncoder>,
                    Resource<GpuBuffer>,
                    GpuIndexFormat,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_set_index_buffer_described(
                        &cb,
                        l2_encoder,
                        buffer_rep,
                        match format {
                            GpuIndexFormat::Uint16 => 1,
                            GpuIndexFormat::Uint32 => 2,
                        },
                        offset.unwrap_or(0),
                        size.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-vertex-buffer",
                |mut caller,
                 (encoder, slot, buffer, offset, size): (
                    Resource<GpuRenderBundleEncoder>,
                    u32,
                    Option<Resource<GpuBuffer>>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let buffer_rep = match buffer.as_ref() {
                        Some(buffer) => caller.data_mut().table.get(buffer)?.rep,
                        None => 0,
                    };
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_set_vertex_buffer_described(
                        &cb,
                        l2_encoder,
                        slot,
                        buffer_rep,
                        offset.unwrap_or(0),
                        size.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.draw-indexed",
                |mut caller,
                 (
                    encoder,
                    index_count,
                    instance_count,
                    first_index,
                    base_vertex,
                    first_instance,
                ): (
                    Resource<GpuRenderBundleEncoder>,
                    u32,
                    Option<u32>,
                    Option<u32>,
                    Option<i32>,
                    Option<u32>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_draw_indexed_described(
                        &cb,
                        l2_encoder,
                        index_count,
                        instance_count.unwrap_or(1),
                        first_index.unwrap_or(0),
                        base_vertex.unwrap_or(0),
                        first_instance.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.draw-indirect",
                |mut caller,
                 (encoder, buffer, offset): (
                    Resource<GpuRenderBundleEncoder>,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_draw_indirect_described(
                        &cb, l2_encoder, buffer_rep, offset,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.draw-indexed-indirect",
                |mut caller,
                 (encoder, buffer, offset): (
                    Resource<GpuRenderBundleEncoder>,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_draw_indexed_indirect_described(
                        &cb, l2_encoder, buffer_rep, offset,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.push-debug-group",
                |mut caller, (encoder, group_label): (Resource<GpuRenderBundleEncoder>, String)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_push_debug_group_described(
                        &cb,
                        l2_encoder,
                        group_label,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.pop-debug-group",
                |mut caller, (encoder,): (Resource<GpuRenderBundleEncoder>,)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_pop_debug_group_described(&cb, l2_encoder)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.insert-debug-marker",
                |mut caller, (encoder, marker_label): (Resource<GpuRenderBundleEncoder>, String)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_insert_debug_marker_described(
                        &cb,
                        l2_encoder,
                        marker_label,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-immediates",
                |mut caller,
                 (encoder, range_offset, data, data_offset, data_size): (
                    Resource<GpuRenderBundleEncoder>,
                    u32,
                    Vec<u8>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    let _ = data_size;
                    jvm::exp_render_bundle_encoder_set_immediates_described(
                        &cb,
                        l2_encoder,
                        range_offset,
                        data,
                        data_offset.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-compute-pass", |mut store, ()| {
                let resource = store
                    .data_mut()
                    .table
                    .push(GpuComputePassEncoder { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.label",
                |mut caller, (pass,): (Resource<GpuComputePassEncoder>,)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        return Ok((String::new(),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    let label = jvm::exp_compute_pass_encoder_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.set-label",
                |mut caller, (pass, label): (Resource<GpuComputePassEncoder>, String)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        let _ = label;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2 = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_encoder_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.end",
                |mut caller, (pass,): (Resource<GpuComputePassEncoder>,)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .compute_pass_end(pass_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_end_described(&cb, l2_pass)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.set-pipeline",
                |mut caller,
                 (pass, pipeline): (
                    Resource<GpuComputePassEncoder>,
                    Resource<GpuComputePipeline>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .compute_pass_set_pipeline(pass_rep, pipeline_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_set_pipeline_described(&cb, l2_pass, pipeline_rep)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.set-bind-group",
                |mut caller,
                 (pass, index, bind_group, _offsets, _start, _length): (
                    Resource<GpuComputePassEncoder>,
                    u32,
                    Option<Resource<GpuBindGroup>>,
                    Option<Vec<u32>>,
                    Option<u64>,
                    Option<u32>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let bind_group_rep = match bind_group {
                        Some(ref g) => caller.data_mut().table.get(g)?.rep,
                        None => 0,
                    };
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .compute_pass_set_bind_group(pass_rep, index, bind_group_rep)
                            .map_err(native_gpu_error)?;
                        return Ok((Ok::<(), SetBindGroupError>(()),));
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_set_bind_group_described(
                        &cb,
                        l2_pass,
                        index,
                        bind_group_rep,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((Ok::<(), SetBindGroupError>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.dispatch-workgroups",
                |mut caller,
                 (pass, x, y, z): (
                    Resource<GpuComputePassEncoder>,
                    u32,
                    Option<u32>,
                    Option<u32>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .compute_pass_dispatch_xyz(pass_rep, x, y.unwrap_or(1), z.unwrap_or(1))
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_dispatch_workgroups_described(
                        &cb,
                        l2_pass,
                        x,
                        y.unwrap_or(1),
                        z.unwrap_or(1),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.dispatch-workgroups-indirect",
                |mut caller,
                 (pass, buffer, offset): (
                    Resource<GpuComputePassEncoder>,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .compute_pass_dispatch_indirect(pass_rep, buffer_rep, offset)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_dispatch_workgroups_indirect_described(
                        &cb, l2_pass, buffer_rep, offset,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.set-immediates",
                |mut caller,
                 (pass, range_offset, data, data_offset, data_size): (
                    Resource<GpuComputePassEncoder>,
                    u32,
                    Vec<u8>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .resolve_compute_pass(pass_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    let _ = data_size;
                    jvm::exp_compute_pass_set_immediates_described(
                        &cb,
                        l2_pass,
                        range_offset,
                        data,
                        data_offset.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.push-debug-group",
                |mut caller, (pass, group_label): (Resource<GpuComputePassEncoder>, String)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .resolve_compute_pass(pass_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_push_debug_group_described(&cb, l2_pass, group_label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.pop-debug-group",
                |mut caller, (pass,): (Resource<GpuComputePassEncoder>,)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .resolve_compute_pass(pass_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_pop_debug_group_described(&cb, l2_pass)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.insert-debug-marker",
                |mut caller, (pass, marker_label): (Resource<GpuComputePassEncoder>, String)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    if caller.data().webgpu_backend() == GpuBackend::NativeGpu {
                        caller
                            .data_mut()
                            .require_native_gpu()?
                            .resolve_compute_pass(pass_rep)
                            .map_err(native_gpu_error)?;
                        return Ok(());
                    }
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_insert_debug_marker_described(&cb, l2_pass, marker_label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent("request-adapter", |accessor, ()| {
                Box::pin(async move {
                    let cb = accessor.with(|mut access| access.data_mut().webgpu_jni_cb());
                    // Yield so this is true concurrent (not sync wrap / Latch fake-async).
                    let (tx, rx) = oneshot::channel::<()>();
                    std::thread::spawn(move || {
                        let _ = tx.send(());
                    });
                    let _ = rx.await;
                    let Some(cb) = cb else {
                        return Ok((0,));
                    };
                    let rep = jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                    Ok((rep,))
                })
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent("adapter-request-device", |accessor, (adapter,): (u32,)| {
                Box::pin(async move {
                    let cb =
                        accessor.with(|mut access| access.data_mut().require_webgpu_jni_cb())?;
                    let (tx, rx) = oneshot::channel::<()>();
                    std::thread::spawn(move || {
                        let _ = tx.send(());
                    });
                    let _ = rx.await;
                    let rep = jvm::exp_adapter_request_device(&cb, adapter)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((rep,))
                })
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("device-get-queue", |caller, (device,): (u32,)| {
                let cb = caller.data().require_webgpu_jni_cb()?;
                let rep = jvm::exp_device_get_queue(&cb, device).map_err(wasmtime::Error::msg)?;
                Ok((rep,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "device-create-command-encoder",
                |caller, (device,): (u32,)| {
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let rep = jvm::exp_create_command_encoder(&cb, device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((rep,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("command-encoder-finish", |caller, (encoder,): (u32,)| {
                let cb = caller.data().require_webgpu_jni_cb()?;
                let rep =
                    jvm::exp_command_encoder_finish(&cb, encoder).map_err(wasmtime::Error::msg)?;
                Ok((rep,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("queue-submit1", |caller, (queue, commands): (u32, u32)| {
                let cb = caller.data().require_webgpu_jni_cb()?;
                jvm::exp_queue_submit1(&cb, queue, commands).map_err(wasmtime::Error::msg)?;
                Ok(())
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "command-encoder-begin-render-pass-clear",
                |caller, (encoder, view): (u32, u32)| {
                    let cb = caller.data().require_webgpu_jni_cb()?;
                    let rep = jvm::exp_begin_render_pass_clear(&cb, encoder, view)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((rep,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("render-pass-end", |caller, (pass,): (u32,)| {
                let cb = caller.data().require_webgpu_jni_cb()?;
                jvm::exp_render_pass_end(&cb, pass).map_err(wasmtime::Error::msg)?;
                Ok(())
            })
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeEngineNew(
    mut env: JNIEnv,
    _class: JClass,
) -> jlong {
    match new_engine() {
        Ok(engine) => to_handle(engine),
        Err(e) => {
            throw(&mut env, e);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeEngineClose(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    unsafe { drop_handle::<Engine>(handle) }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeStoreNew(
    mut env: JNIEnv,
    _class: JClass,
    engine: jlong,
) -> jlong {
    if engine == 0 {
        throw(&mut env, "null engine handle");
        return 0;
    }
    let engine = unsafe { from_handle::<Engine>(engine) };
    let store = Store::new(engine, HostState::default());
    let gate = store.data().gfx_on_frame.clone();
    let input = store.data().gfx_input.clone();
    let handle = to_handle(store);
    gfx_on_frame_register(handle, gate);
    gfx_input_register(handle, input);
    handle
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeStoreClose(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if let Some(gate) = gfx_on_frame_unregister(handle) {
        gate.close();
    }
    if let Some(input) = gfx_input_unregister(handle) {
        input.close();
    }
    if handle != 0 {
        let store = unsafe { from_handle::<HostStore>(handle) };
        store.data().gfx_on_resize.close();
        store.data().gfx_input.close();
    }
    unsafe { drop_handle::<HostStore>(handle) }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeStorePostGfxVsync(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    frame_time_nanos: jlong,
) {
    if store == 0 {
        throw(&mut env, "null store handle");
        return;
    }
    if let Some(gate) = gfx_on_frame_lookup(store) {
        gate.post(frame_time_nanos);
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeStorePostGfxPointer(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    kind: jint,
    x: jdouble,
    y: jdouble,
) {
    if store == 0 {
        throw(&mut env, "null store handle");
        return;
    }
    if !(0..=2).contains(&kind) {
        throw(&mut env, "pointer kind must be 0=up, 1=down, 2=move");
        return;
    }
    if let Some(input) = gfx_input_lookup(store) {
        input.post_pointer(kind, x, y);
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeStorePostGfxKey(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    down: jboolean,
    android_key_code: jint,
    text: JString,
    alt: jboolean,
    ctrl: jboolean,
    meta: jboolean,
    shift: jboolean,
) {
    if store == 0 {
        throw(&mut env, "null store handle");
        return;
    }
    let text = if text.is_null() {
        None
    } else {
        match env.get_string(&text) {
            Ok(s) => {
                let owned = s.to_string_lossy().into_owned();
                if owned.is_empty() {
                    None
                } else {
                    Some(owned)
                }
            }
            Err(_) => None,
        }
    };
    let sample = GfxKeySample {
        key: gfx_key_from_android(android_key_code).map(|k| k as u8),
        text,
        alt_key: alt != 0,
        ctrl_key: ctrl != 0,
        meta_key: meta != 0,
        shift_key: shift != 0,
    };
    if let Some(input) = gfx_input_lookup(store) {
        input.post_key(down != 0, sample);
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeStoreCloseGfxOnFrame(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
) {
    if store == 0 {
        throw(&mut env, "null store handle");
        return;
    }
    if let Some(gate) = gfx_on_frame_lookup(store) {
        gate.close();
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeStoreBindCanvasNativeWindow(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    window: jlong,
    width: jint,
    height: jint,
) {
    if store == 0 {
        throw(&mut env, "null store handle");
        return;
    }
    if window == 0 || width <= 0 || height <= 0 {
        throw(&mut env, "invalid canvas native window");
        return;
    }
    let store = unsafe { from_handle::<HostStore>(store) };
    store
        .data_mut()
        .bind_canvas_native_window(window, width as u32, height as u32);
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeStoreSetHostAdd(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    callback: JObject,
) {
    if store == 0 {
        throw(&mut env, "null store handle");
        return;
    }
    if callback.is_null() {
        throw(&mut env, "null host add callback");
        return;
    }
    let gref = match jvm::global_ref(&mut env, callback) {
        Ok(g) => g,
        Err(e) => {
            throw(&mut env, e);
            return;
        }
    };
    let store = unsafe { from_handle::<HostStore>(store) };
    store.data_mut().add_cb = Some(gref);
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeStoreSetExperimentalHost(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    callback: JObject,
) {
    if store == 0 {
        throw(&mut env, "null store handle");
        return;
    }
    if callback.is_null() {
        throw(&mut env, "null experimental host callback");
        return;
    }
    let gref = match jvm::global_ref(&mut env, callback) {
        Ok(g) => g,
        Err(e) => {
            throw(&mut env, e);
            return;
        }
    };
    let store = unsafe { from_handle::<HostStore>(store) };
    let data = store.data_mut();
    data.disable_native_gpu();
    data.experimental_host_cb = Some(gref);
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeStoreSetNativeGpu(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
) {
    if store == 0 {
        throw(&mut env, "null store handle");
        return;
    }
    let store = unsafe { from_handle::<HostStore>(store) };
    store.data_mut().enable_native_gpu();
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeComponentCompile(
    mut env: JNIEnv,
    _class: JClass,
    engine: jlong,
    bytes: JByteArray,
) -> jlong {
    if engine == 0 {
        throw(&mut env, "null engine handle");
        return 0;
    }
    let engine = unsafe { from_handle::<Engine>(engine) };
    let data = match env.convert_byte_array(&bytes) {
        Ok(d) => d,
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    match Component::new(engine, &data) {
        Ok(c) => to_handle(c),
        Err(e) => {
            throw_compile(&mut env, e);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeComponentClose(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    unsafe { drop_handle::<Component>(handle) }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeLinkerNew(
    mut env: JNIEnv,
    _class: JClass,
    engine: jlong,
) -> jlong {
    if engine == 0 {
        throw(&mut env, "null engine handle");
        return 0;
    }
    let engine = unsafe { from_handle::<Engine>(engine) };
    let mut linker = Linker::<HostState>::new(engine);
    if let Err(e) = define_host(&mut linker, false) {
        throw_link(&mut env, e);
        return 0;
    }
    to_handle(linker)
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeLinkerNewWithFixtureConstructors(
    mut env: JNIEnv,
    _class: JClass,
    engine: jlong,
) -> jlong {
    if engine == 0 {
        throw(&mut env, "null engine handle");
        return 0;
    }
    let engine = unsafe { from_handle::<Engine>(engine) };
    let mut linker = Linker::<HostState>::new(engine);
    if let Err(e) = define_host(&mut linker, true) {
        throw_link(&mut env, e);
        return 0;
    }
    to_handle(linker)
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeLinkerClose(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    unsafe { drop_handle::<Linker<HostState>>(handle) }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeInstantiate(
    mut env: JNIEnv,
    _class: JClass,
    linker: jlong,
    store: jlong,
    component: jlong,
) -> jlong {
    if linker == 0 || store == 0 || component == 0 {
        throw(&mut env, "null linker/store/component handle");
        return 0;
    }
    let linker = unsafe { from_handle::<Linker<HostState>>(linker) };
    let store = unsafe { from_handle::<HostStore>(store) };
    let component = unsafe { from_handle::<Component>(component) };
    match pollster::block_on(linker.instantiate_async(&mut *store, component)) {
        Ok(instance) => to_handle(instance),
        Err(e) => {
            throw_link(&mut env, e);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeInstanceClose(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    unsafe { drop_handle::<wasmtime::component::Instance>(handle) }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeCallU32(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    instance: jlong,
    export_name: JString,
    arg: jint,
) -> jint {
    if store == 0 || instance == 0 {
        throw(&mut env, "null store/instance handle");
        return 0;
    }
    let name: String = match env.get_string(&export_name) {
        Ok(s) => s.into(),
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    let store = unsafe { from_handle::<HostStore>(store) };
    let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };
    let func = match instance.get_typed_func::<(u32,), (u32,)>(&mut *store, name.as_str()) {
        Ok(f) => f,
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    match func.call(&mut *store, (arg as u32,)) {
        Ok((result,)) => result as jint,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeCallUnitToU32(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    instance: jlong,
    export_name: JString,
) -> jint {
    if store == 0 || instance == 0 {
        throw(&mut env, "null store/instance handle");
        return 0;
    }
    let name: String = match env.get_string(&export_name) {
        Ok(s) => s.into(),
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    let store = unsafe { from_handle::<HostStore>(store) };
    let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };
    let func = match instance.get_typed_func::<(), (u32,)>(&mut *store, name.as_str()) {
        Ok(f) => f,
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    match func.call(&mut *store, ()) {
        Ok((result,)) => result as jint,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeCallUnitToU64(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    instance: jlong,
    export_name: JString,
) -> jlong {
    if store == 0 || instance == 0 {
        throw(&mut env, "null store/instance handle");
        return 0;
    }
    let name: String = match env.get_string(&export_name) {
        Ok(s) => s.into(),
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    let store = unsafe { from_handle::<HostStore>(store) };
    let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };
    let func = match instance.get_typed_func::<(), (u64,)>(&mut *store, name.as_str()) {
        Ok(f) => f,
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    match func.call(&mut *store, ()) {
        Ok((result,)) => result as jlong,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeCallU32U32(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    instance: jlong,
    export_name: JString,
    a: jint,
    b: jint,
) -> jint {
    if store == 0 || instance == 0 {
        throw(&mut env, "null store/instance handle");
        return 0;
    }
    let name: String = match env.get_string(&export_name) {
        Ok(s) => s.into(),
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    let store = unsafe { from_handle::<HostStore>(store) };
    let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };
    let func = match instance.get_typed_func::<(u32, u32), (u32,)>(&mut *store, name.as_str()) {
        Ok(f) => f,
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    match func.call(&mut *store, (a as u32, b as u32)) {
        Ok((result,)) => result as jint,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}

/// M4: call root export `(u64, u32, u32) -> u32` (e.g. `run-clear`).
#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeCallU64U32U32(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    instance: jlong,
    export_name: JString,
    a: jlong,
    b: jint,
    c: jint,
) -> jint {
    if store == 0 || instance == 0 {
        throw(&mut env, "null store/instance handle");
        return 0;
    }
    let name: String = match env.get_string(&export_name) {
        Ok(s) => s.into(),
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    let store = unsafe { from_handle::<HostStore>(store) };
    let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };
    let func = match instance.get_typed_func::<(u64, u32, u32), (u32,)>(&mut *store, name.as_str())
    {
        Ok(f) => f,
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    match func.call(&mut *store, (a as u64, b as u32, c as u32)) {
        Ok((result,)) => result as jint,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}

/// ART instrument threads are ~1MiB; W3 extra JNI hops overflow that.
/// Pump Wasmtime on an 8MiB pthread; bounce L2 JNI to the caller (ART aborts
/// AttachCurrentThread on a custom-stack pthread — Java Thread stackSize is ignored).
const CM_PUMP_STACK_BYTES: usize = 8 * 1024 * 1024;

/// M2: call root export `run: func() -> u32` under `run_concurrent` / `call_concurrent`.
#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeCallRunConcurrent(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    instance: jlong,
) -> jint {
    if store == 0 || instance == 0 {
        throw(&mut env, "null store/instance handle");
        return 0;
    }

    // Sync export that sync-lowers an async import: drive with run_concurrent + call_concurrent.
    // (Matches Wasmtime's sync-lower-async-host pattern; pollster pumps the event loop.)
    let result = match jvm::run_on_cm_pump(&mut env, CM_PUMP_STACK_BYTES, move || {
        let store = unsafe { from_handle::<HostStore>(store) };
        let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };
        pollster::block_on(async {
            store
                .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                    let func = accessor.with(|mut access| {
                        instance.get_typed_func::<(), (u32,)>(&mut access, "run")
                    })?;
                    let (value,) = func.call_concurrent(accessor, ()).await?;
                    Ok(value)
                })
                .await?
        })
    }) {
        Ok(inner) => inner,
        Err(e) => {
            throw(&mut env, e);
            return 0;
        }
    };

    match result {
        Ok(v) => v as jint,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}

/// P3-PRIM-3: host `StreamReader` (fixed `P3ST` bytes) → guest export `read`.
/// Packed result: `(nbytes << 4) | status` (status 1 = DROPPED).
#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeCallStreamRead(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    instance: jlong,
    max_len: jint,
) -> jint {
    if store == 0 || instance == 0 {
        throw(&mut env, "null store/instance handle");
        return 0;
    }
    if max_len <= 0 {
        throw(&mut env, "max_len must be positive");
        return 0;
    }
    let store = unsafe { from_handle::<HostStore>(store) };
    let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };

    let result = (|| -> wasmtime::Result<u32> {
        let func =
            instance.get_typed_func::<(StreamReader<u8>, u32), (u32,)>(&mut *store, "read")?;
        let reader = StreamReader::new(&mut *store, b"P3ST".to_vec())?;
        let (packed,) = pollster::block_on(func.call_async(&mut *store, (reader, max_len as u32)))?;
        Ok(packed)
    })();

    match result {
        Ok(v) => v as jint,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}

/// P3-PRIM-5: guest `stream.write` → host `take`/`StreamConsumer`; returns byte count.
#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeCallStreamWrite(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    instance: jlong,
) -> jint {
    if store == 0 || instance == 0 {
        throw(&mut env, "null store/instance handle");
        return 0;
    }
    // Same 8MiB pump as nativeCallRunConcurrent: ART instrument threads are
    // ~1MiB; run_concurrent + StreamConsumer on that stack crashes the
    // instrumentation process (Vivo). Do not AttachCurrentThread on the pump.
    let result = match jvm::run_on_cm_pump(&mut env, CM_PUMP_STACK_BYTES, move || {
        let store = unsafe { from_handle::<HostStore>(store) };
        let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };
        pollster::block_on(async {
            store
                .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                    let func = accessor.with(|mut access| {
                        instance.get_typed_func::<(), (u32,)>(&mut access, "run")
                    })?;
                    let (n,) = func.call_concurrent(accessor, ()).await?;
                    Ok(n)
                })
                .await?
        })
    }) {
        Ok(inner) => inner,
        Err(e) => {
            throw(&mut env, e);
            return 0;
        }
    };

    match result {
        Ok(v) => v as jint,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}
