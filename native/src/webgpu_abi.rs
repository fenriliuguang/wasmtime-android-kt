//! WIT types for `wasi:webgpu@0.3.0-rc.2` used by canonical-shape slices.
//! S2: `gpu-request-adapter-options` + `gpu-power-preference`.
//! S3: `gpu-device-descriptor` + `request-device-error` (+ feature enum / queue descriptor).

use wasmtime::component::{ComponentType, Lift, Lower, Resource};

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuPowerPreference {
    #[component(name = "low-power")]
    LowPower,
    #[component(name = "high-performance")]
    HighPerformance,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuRequestAdapterOptions {
    #[component(name = "feature-level")]
    pub feature_level: Option<String>,
    #[component(name = "power-preference")]
    pub power_preference: Option<GpuPowerPreference>,
    #[component(name = "force-fallback-adapter")]
    pub force_fallback_adapter: Option<bool>,
    #[component(name = "xr-compatible")]
    pub xr_compatible: Option<bool>,
}

/// WIT `enum gpu-feature-name` (S3 descriptor graph; guest currently passes none).
#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuFeatureName {
    #[component(name = "core-features-and-limits")]
    CoreFeaturesAndLimits,
    #[component(name = "depth-clip-control")]
    DepthClipControl,
    #[component(name = "depth32float-stencil8")]
    Depth32floatStencil8,
    #[component(name = "texture-compression-bc")]
    TextureCompressionBc,
    #[component(name = "texture-compression-bc-sliced3d")]
    TextureCompressionBcSliced3d,
    #[component(name = "texture-compression-etc2")]
    TextureCompressionEtc2,
    #[component(name = "texture-compression-astc")]
    TextureCompressionAstc,
    #[component(name = "texture-compression-astc-sliced3d")]
    TextureCompressionAstcSliced3d,
    #[component(name = "timestamp-query")]
    TimestampQuery,
    #[component(name = "indirect-first-instance")]
    IndirectFirstInstance,
    #[component(name = "shader-f16")]
    ShaderF16,
    #[component(name = "rg11b10ufloat-renderable")]
    Rg11b10ufloatRenderable,
    #[component(name = "bgra8unorm-storage")]
    Bgra8unormStorage,
    #[component(name = "float32-filterable")]
    Float32Filterable,
    #[component(name = "float32-blendable")]
    Float32Blendable,
    #[component(name = "clip-distances")]
    ClipDistances,
    #[component(name = "dual-source-blending")]
    DualSourceBlending,
    #[component(name = "subgroups")]
    Subgroups,
    #[component(name = "texture-formats-tier1")]
    TextureFormatsTier1,
    #[component(name = "texture-formats-tier2")]
    TextureFormatsTier2,
    #[component(name = "primitive-index")]
    PrimitiveIndex,
    #[component(name = "texture-component-swizzle")]
    TextureComponentSwizzle,
}

/// WIT `resource record-option-gpu-size64` (limits map). S3 only needs the type in the graph.
#[derive(Debug)]
pub struct RecordOptionGpuSize64;

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuQueueDescriptor {
    pub label: Option<String>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuDeviceDescriptor {
    #[component(name = "required-features")]
    pub required_features: Option<Vec<GpuFeatureName>>,
    #[component(name = "required-limits")]
    pub required_limits: Option<Resource<RecordOptionGpuSize64>>,
    #[component(name = "default-queue")]
    pub default_queue: Option<GpuQueueDescriptor>,
    pub label: Option<String>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
pub enum RequestDeviceErrorKind {
    #[component(name = "type-error")]
    TypeError,
    #[component(name = "operation-error")]
    OperationError,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct RequestDeviceError {
    pub kind: RequestDeviceErrorKind,
    pub message: String,
}
