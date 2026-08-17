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

/// Host representation of WIT `resource gpu-adapter`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
#[derive(Debug)]
pub struct GpuAdapter {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-device`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
/// S1 `get-device` still pushes `{ rep: 0 }`; L2 handles for that path are
/// obtained inside later methods. S3 `request-device` stores the real L2 rep.
#[derive(Debug)]
pub struct GpuDevice {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-command-buffer`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
/// `get-command-buffer` still pushes `{ rep: 0 }`; S7 `encoder.finish` stores
/// the real L2 rep.
#[derive(Debug)]
pub struct GpuCommandBuffer {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-command-encoder`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
/// `get-encoder` still pushes `{ rep: 0 }`; S6 `create-command-encoder` stores
/// the real L2 rep.
#[derive(Debug)]
pub struct GpuCommandEncoder {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-render-pass-encoder` (W3 `[method]`).
#[derive(Debug)]
pub struct GpuRenderPassEncoder;

/// Host representation of WIT `resource gpu-compute-pass-encoder`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
/// `get-compute-pass` still pushes `{ rep: 0 }`; S8 `begin-compute-pass` stores
/// the real L2 rep.
#[derive(Debug)]
pub struct GpuComputePassEncoder {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-queue`.
/// `rep` is the Dawn / Cpu L2 handle (u32); Guest sees `own`/`borrow`, not this value.
/// Read by later slices (submit / write-*); S1 only stores it.
#[derive(Debug)]
pub struct GpuQueue {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-texture` (W3+ `[method]`).
/// `get-texture` still pushes `{ rep: 0 }`; create-texture still returns u32.
#[derive(Debug)]
pub struct GpuTexture {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-sampler`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
#[derive(Debug)]
pub struct GpuSampler {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-texture-view`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
#[derive(Debug)]
pub struct GpuTextureView {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-buffer`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
/// `get-buffer` still pushes `{ rep: 0 }`; S4 `create-buffer` stores the real L2 rep.
#[derive(Debug)]
pub struct GpuBuffer {
    #[allow(dead_code)]
    pub rep: u32,
}

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
