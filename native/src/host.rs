//! Store host state: Kotlin callbacks + u32-rep widget / gpu resources.

use jni::objects::GlobalRef;
use wasmtime::component::ResourceTable;

#[derive(Debug)]
pub struct Widget {
    pub rep: u32,
}

/// Host representation of WIT `resource gpu` (W3 `[method]` slice). No L2 handle.
#[derive(Debug)]
pub struct Gpu;

/// Host representation of WIT `resource gpu-adapter` (W3 `[method]` slice).
/// No L2 handle yet; `[method]gpu-adapter.request-device` calls L2
/// `request-adapter` then `adapter-request-device`.
#[derive(Debug)]
pub struct GpuAdapter;

/// Host representation of WIT `resource gpu-device` (W3 `[method]` slice).
/// No L2 handle yet; `[method]gpu-device.queue` calls L2 `request-adapter` then
/// `adapter-request-device` then `device-get-queue`.
#[derive(Debug)]
pub struct GpuDevice;

/// Host representation of WIT `resource gpu-command-encoder` (W3 `[method]`).
/// No L2 handle yet; begin-render-pass / finish chain L2 from adapter.
#[derive(Debug)]
pub struct GpuCommandEncoder;

/// Host representation of WIT `resource gpu-render-pass-encoder` (W3 `[method]`).
#[derive(Debug)]
pub struct GpuRenderPassEncoder;

/// Host representation of WIT `resource gpu-compute-pass-encoder` (W3+ `[method]`).
#[derive(Debug)]
pub struct GpuComputePassEncoder;

/// Host representation of WIT `resource gpu-queue` (W3 `[method]`).
#[derive(Debug)]
pub struct GpuQueue;

/// Host representation of WIT `resource gpu-texture` (W3+ `[method]`).
#[derive(Debug)]
pub struct GpuTexture;

/// Host representation of WIT `resource gpu-buffer` (W3+ `[method]`).
#[derive(Debug)]
pub struct GpuBuffer;

pub struct HostState {
    pub table: ResourceTable,
    pub add_cb: Option<GlobalRef>,
    /// Kotlin [ExperimentalHostCallbacks] for experimental CM host (M3/M4).
    pub experimental_host_cb: Option<GlobalRef>,
}

impl Default for HostState {
    fn default() -> Self {
        Self {
            table: ResourceTable::new(),
            add_cb: None,
            experimental_host_cb: None,
        }
    }
}
