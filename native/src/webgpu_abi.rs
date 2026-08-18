//! WIT types for `wasi:webgpu@0.3.0-rc.2` used by canonical-shape slices.
//! S2: `gpu-request-adapter-options` + `gpu-power-preference`.
//! S3: `gpu-device-descriptor` + `request-device-error` (+ feature enum / queue descriptor).
//! S4: `gpu-buffer-descriptor` + `gpu-buffer-usage` flags.
//! S6: `gpu-command-encoder-descriptor`.
//! S7: `gpu-command-buffer-descriptor`.
//! S8: sampler / texture-view / compute-pass option descriptors (guest passes none).
//! S6+: `gpu-texture-descriptor`, `gpu-render-pass-descriptor`, `map-async` result.
//! S6+ layout/shader: shader-module / bind-group-layout / pipeline-layout / bind-group descriptors.
//! S6+ pass commands: `set-bind-group-error` (+ kind).
//! S6+ unmap / write-*-with-copy: `unmap-error`, `write-buffer-error`, texel copy info.
//! S6+ pipeline create: render/compute pipeline descriptors (+ vertex/fragment graph).
//! S6+ pipeline-async / mapped-range: `create-pipeline-error`, `get-mapped-range-error`.

use crate::host::{
    GpuBindGroupLayout, GpuBuffer, GpuPipelineLayout, GpuSampler, GpuShaderModule, GpuTexture,
    GpuTextureView,
};
use wasmtime::component::{flags, ComponentType, Lift, Lower, Resource};

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

impl GpuBufferUsage {
    /// WIT declaration order matches WebGPU / Dawn `GPUBufferUsage` bits.
    pub fn to_webgpu_u32(self) -> u32 {
        let mut bits = 0u32;
        if self.contains(Self::MAP_READ) {
            bits |= 1 << 0;
        }
        if self.contains(Self::MAP_WRITE) {
            bits |= 1 << 1;
        }
        if self.contains(Self::COPY_SRC) {
            bits |= 1 << 2;
        }
        if self.contains(Self::COPY_DST) {
            bits |= 1 << 3;
        }
        if self.contains(Self::INDEX) {
            bits |= 1 << 4;
        }
        if self.contains(Self::VERTEX) {
            bits |= 1 << 5;
        }
        if self.contains(Self::UNIFORM) {
            bits |= 1 << 6;
        }
        if self.contains(Self::STORAGE) {
            bits |= 1 << 7;
        }
        if self.contains(Self::INDIRECT) {
            bits |= 1 << 8;
        }
        if self.contains(Self::QUERY_RESOLVE) {
            bits |= 1 << 9;
        }
        bits
    }
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
pub struct GpuBufferDescriptor {
    pub size: u64,
    pub usage: GpuBufferUsage,
    #[component(name = "mapped-at-creation")]
    pub mapped_at_creation: Option<bool>,
    pub label: Option<String>,
}

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
pub struct GpuCommandEncoderDescriptor {
    pub label: Option<String>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuCommandBufferDescriptor {
    pub label: Option<String>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct RequestDeviceError {
    pub kind: RequestDeviceErrorKind,
    pub message: String,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuAddressMode {
    #[component(name = "clamp-to-edge")]
    ClampToEdge,
    #[component(name = "repeat")]
    Repeat,
    #[component(name = "mirror-repeat")]
    MirrorRepeat,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuFilterMode {
    #[component(name = "nearest")]
    Nearest,
    #[component(name = "linear")]
    Linear,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuMipmapFilterMode {
    #[component(name = "nearest")]
    Nearest,
    #[component(name = "linear")]
    Linear,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuCompareFunction {
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

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuSamplerDescriptor {
    #[component(name = "address-mode-u")]
    pub address_mode_u: Option<GpuAddressMode>,
    #[component(name = "address-mode-v")]
    pub address_mode_v: Option<GpuAddressMode>,
    #[component(name = "address-mode-w")]
    pub address_mode_w: Option<GpuAddressMode>,
    #[component(name = "mag-filter")]
    pub mag_filter: Option<GpuFilterMode>,
    #[component(name = "min-filter")]
    pub min_filter: Option<GpuFilterMode>,
    #[component(name = "mipmap-filter")]
    pub mipmap_filter: Option<GpuMipmapFilterMode>,
    #[component(name = "lod-min-clamp")]
    pub lod_min_clamp: Option<f32>,
    #[component(name = "lod-max-clamp")]
    pub lod_max_clamp: Option<f32>,
    pub compare: Option<GpuCompareFunction>,
    #[component(name = "max-anisotropy")]
    pub max_anisotropy: Option<u16>,
    pub label: Option<String>,
}

/// WIT `resource gpu-query-set` (S8 compute-pass descriptor graph).
#[derive(Debug)]
pub struct GpuQuerySet;

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuComputePassTimestampWrites {
    #[component(name = "query-set")]
    pub query_set: Resource<GpuQuerySet>,
    #[component(name = "beginning-of-pass-write-index")]
    pub beginning_of_pass_write_index: Option<u32>,
    #[component(name = "end-of-pass-write-index")]
    pub end_of_pass_write_index: Option<u32>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuComputePassDescriptor {
    #[component(name = "timestamp-writes")]
    pub timestamp_writes: Option<GpuComputePassTimestampWrites>,
    pub label: Option<String>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuTextureAspect {
    #[component(name = "all")]
    All,
    #[component(name = "stencil-only")]
    StencilOnly,
    #[component(name = "depth-only")]
    DepthOnly,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuTextureViewDimension {
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

impl GpuTextureUsage {
    /// WIT declaration order matches WebGPU / Dawn `GPUTextureUsage` bits.
    pub fn to_webgpu_u32(self) -> u32 {
        let mut bits = 0u32;
        if self.contains(Self::COPY_SRC) {
            bits |= 1 << 0;
        }
        if self.contains(Self::COPY_DST) {
            bits |= 1 << 1;
        }
        if self.contains(Self::TEXTURE_BINDING) {
            bits |= 1 << 2;
        }
        if self.contains(Self::STORAGE_BINDING) {
            bits |= 1 << 3;
        }
        if self.contains(Self::RENDER_ATTACHMENT) {
            bits |= 1 << 4;
        }
        if self.contains(Self::TRANSIENT_ATTACHMENT) {
            bits |= 1 << 5;
        }
        bits
    }
}

flags! {
    GpuMapMode {
        #[component(name = "read")]
        const READ;
        #[component(name = "write")]
        const WRITE;
    }
}

impl GpuMapMode {
    pub fn to_webgpu_u32(self) -> u32 {
        let mut bits = 0u32;
        if self.contains(Self::READ) {
            bits |= 1 << 0;
        }
        if self.contains(Self::WRITE) {
            bits |= 1 << 1;
        }
        bits
    }
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuTextureFormat {
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

impl GpuTextureFormat {
    /// `androidx.webgpu.TextureFormat` / Dawn value used by L2.
    /// This slice only needs RGBA8Unorm (`0x16` on alpha05).
    pub fn to_dawn_u32(self) -> u32 {
        match self {
            Self::Rgba8unorm => 0x16,
            _ => 0x16,
        }
    }
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuTextureViewDescriptor {
    pub format: Option<GpuTextureFormat>,
    pub dimension: Option<GpuTextureViewDimension>,
    pub usage: Option<GpuTextureUsage>,
    pub aspect: Option<GpuTextureAspect>,
    #[component(name = "base-mip-level")]
    pub base_mip_level: Option<u32>,
    #[component(name = "mip-level-count")]
    pub mip_level_count: Option<u32>,
    #[component(name = "base-array-layer")]
    pub base_array_layer: Option<u32>,
    #[component(name = "array-layer-count")]
    pub array_layer_count: Option<u32>,
    pub swizzle: Option<String>,
    pub label: Option<String>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuTextureDimension {
    #[component(name = "d1")]
    D1,
    #[component(name = "d2")]
    D2,
    #[component(name = "d3")]
    D3,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
pub struct GpuExtent3D {
    pub width: u32,
    pub height: Option<u32>,
    #[component(name = "depth-or-array-layers")]
    pub depth_or_array_layers: Option<u32>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
pub struct GpuTextureDescriptor {
    pub size: GpuExtent3D,
    #[component(name = "mip-level-count")]
    pub mip_level_count: Option<u32>,
    #[component(name = "sample-count")]
    pub sample_count: Option<u32>,
    pub dimension: Option<GpuTextureDimension>,
    pub format: GpuTextureFormat,
    pub usage: GpuTextureUsage,
    #[component(name = "view-formats")]
    pub view_formats: Option<Vec<GpuTextureFormat>>,
    #[component(name = "texture-binding-view-dimension")]
    pub texture_binding_view_dimension: Option<GpuTextureViewDimension>,
    pub label: Option<String>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
pub enum MapAsyncErrorKind {
    #[component(name = "operation-error")]
    OperationError,
    #[component(name = "range-error")]
    RangeError,
    #[component(name = "abort-error")]
    AbortError,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct MapAsyncError {
    pub kind: MapAsyncErrorKind,
    pub message: String,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
pub enum SetBindGroupErrorKind {
    #[component(name = "range-error")]
    RangeError,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct SetBindGroupError {
    pub kind: SetBindGroupErrorKind,
    pub message: String,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
pub enum UnmapErrorKind {
    #[component(name = "abort-error")]
    AbortError,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct UnmapError {
    pub kind: UnmapErrorKind,
    pub message: String,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
pub enum WriteBufferErrorKind {
    #[component(name = "operation-error")]
    OperationError,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct WriteBufferError {
    pub kind: WriteBufferErrorKind,
    pub message: String,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
pub struct GpuOrigin3D {
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub z: Option<u32>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
pub struct GpuTexelCopyTextureInfo {
    pub texture: Resource<GpuTexture>,
    #[component(name = "mip-level")]
    pub mip_level: Option<u32>,
    pub origin: Option<GpuOrigin3D>,
    pub aspect: Option<GpuTextureAspect>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
pub struct GpuTexelCopyBufferLayout {
    pub offset: Option<u64>,
    #[component(name = "bytes-per-row")]
    pub bytes_per_row: Option<u32>,
    #[component(name = "rows-per-image")]
    pub rows_per_image: Option<u32>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuLoadOp {
    #[component(name = "load")]
    Load,
    #[component(name = "clear")]
    Clear,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuStoreOp {
    #[component(name = "store")]
    Store,
    #[component(name = "discard")]
    Discard,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuColor {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuRenderPassColorAttachment {
    pub view: Resource<GpuTextureView>,
    #[component(name = "depth-slice")]
    pub depth_slice: Option<u32>,
    #[component(name = "resolve-target")]
    pub resolve_target: Option<Resource<GpuTextureView>>,
    #[component(name = "clear-value")]
    pub clear_value: Option<GpuColor>,
    #[component(name = "load-op")]
    pub load_op: GpuLoadOp,
    #[component(name = "store-op")]
    pub store_op: GpuStoreOp,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuRenderPassDepthStencilAttachment {
    pub view: Resource<GpuTextureView>,
    #[component(name = "depth-clear-value")]
    pub depth_clear_value: Option<f32>,
    #[component(name = "depth-load-op")]
    pub depth_load_op: Option<GpuLoadOp>,
    #[component(name = "depth-store-op")]
    pub depth_store_op: Option<GpuStoreOp>,
    #[component(name = "depth-read-only")]
    pub depth_read_only: Option<bool>,
    #[component(name = "stencil-clear-value")]
    pub stencil_clear_value: Option<u32>,
    #[component(name = "stencil-load-op")]
    pub stencil_load_op: Option<GpuLoadOp>,
    #[component(name = "stencil-store-op")]
    pub stencil_store_op: Option<GpuStoreOp>,
    #[component(name = "stencil-read-only")]
    pub stencil_read_only: Option<bool>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuRenderPassTimestampWrites {
    #[component(name = "query-set")]
    pub query_set: Resource<GpuQuerySet>,
    #[component(name = "beginning-of-pass-write-index")]
    pub beginning_of_pass_write_index: Option<u32>,
    #[component(name = "end-of-pass-write-index")]
    pub end_of_pass_write_index: Option<u32>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
pub struct GpuRenderPassDescriptor {
    #[component(name = "color-attachments")]
    pub color_attachments: Vec<Option<GpuRenderPassColorAttachment>>,
    #[component(name = "depth-stencil-attachment")]
    pub depth_stencil_attachment: Option<GpuRenderPassDepthStencilAttachment>,
    #[component(name = "occlusion-query-set")]
    pub occlusion_query_set: Option<Resource<GpuQuerySet>>,
    #[component(name = "timestamp-writes")]
    pub timestamp_writes: Option<GpuRenderPassTimestampWrites>,
    #[component(name = "max-draw-count")]
    pub max_draw_count: Option<u64>,
    pub label: Option<String>,
}

flags! {
    GpuShaderStage {
        #[component(name = "vertex")]
        const VERTEX;
        #[component(name = "fragment")]
        const FRAGMENT;
        #[component(name = "compute")]
        const COMPUTE;
    }
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuBufferBindingType {
    #[component(name = "uniform")]
    Uniform,
    #[component(name = "storage")]
    Storage,
    #[component(name = "read-only-storage")]
    ReadOnlyStorage,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuSamplerBindingType {
    #[component(name = "filtering")]
    Filtering,
    #[component(name = "non-filtering")]
    NonFiltering,
    #[component(name = "comparison")]
    Comparison,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuTextureSampleType {
    #[component(name = "float")]
    Float,
    #[component(name = "unfilterable-float")]
    UnfilterableFloat,
    #[component(name = "depth")]
    Depth,
    #[component(name = "sint")]
    Sint,
    #[component(name = "uint")]
    Uint,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuStorageTextureAccess {
    #[component(name = "write-only")]
    WriteOnly,
    #[component(name = "read-only")]
    ReadOnly,
    #[component(name = "read-write")]
    ReadWrite,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuBufferBindingLayout {
    #[component(name = "type")]
    pub ty: Option<GpuBufferBindingType>,
    #[component(name = "has-dynamic-offset")]
    pub has_dynamic_offset: Option<bool>,
    #[component(name = "min-binding-size")]
    pub min_binding_size: Option<u64>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuSamplerBindingLayout {
    #[component(name = "type")]
    pub ty: Option<GpuSamplerBindingType>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuTextureBindingLayout {
    #[component(name = "sample-type")]
    pub sample_type: Option<GpuTextureSampleType>,
    #[component(name = "view-dimension")]
    pub view_dimension: Option<GpuTextureViewDimension>,
    pub multisampled: Option<bool>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuStorageTextureBindingLayout {
    pub access: Option<GpuStorageTextureAccess>,
    pub format: GpuTextureFormat,
    #[component(name = "view-dimension")]
    pub view_dimension: Option<GpuTextureViewDimension>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuBindGroupLayoutEntry {
    pub binding: u32,
    pub visibility: GpuShaderStage,
    pub buffer: Option<GpuBufferBindingLayout>,
    pub sampler: Option<GpuSamplerBindingLayout>,
    pub texture: Option<GpuTextureBindingLayout>,
    #[component(name = "storage-texture")]
    pub storage_texture: Option<GpuStorageTextureBindingLayout>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
pub struct GpuBindGroupLayoutDescriptor {
    pub entries: Vec<GpuBindGroupLayoutEntry>,
    pub label: Option<String>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
pub struct GpuPipelineLayoutDescriptor {
    #[component(name = "bind-group-layouts")]
    pub bind_group_layouts: Vec<Option<Resource<GpuBindGroupLayout>>>,
    #[component(name = "immediate-size")]
    pub immediate_size: Option<u32>,
    pub label: Option<String>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
pub enum GpuLayoutMode {
    #[component(name = "specific")]
    Specific(Resource<GpuPipelineLayout>),
    #[component(name = "auto")]
    Auto,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuShaderModuleCompilationHint {
    #[component(name = "entry-point")]
    pub entry_point: String,
    pub layout: Option<GpuLayoutMode>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
pub struct GpuShaderModuleDescriptor {
    pub code: String,
    #[component(name = "compilation-hints")]
    pub compilation_hints: Option<Vec<GpuShaderModuleCompilationHint>>,
    pub label: Option<String>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuBufferBinding {
    pub buffer: Resource<GpuBuffer>,
    pub offset: Option<u64>,
    pub size: Option<u64>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
pub enum GpuBindingResource {
    #[component(name = "gpu-buffer")]
    GpuBuffer(Resource<GpuBuffer>),
    #[component(name = "gpu-buffer-binding")]
    GpuBufferBinding(GpuBufferBinding),
    #[component(name = "gpu-sampler")]
    GpuSampler(Resource<GpuSampler>),
    #[component(name = "gpu-texture")]
    GpuTexture(Resource<GpuTexture>),
    #[component(name = "gpu-texture-view")]
    GpuTextureView(Resource<GpuTextureView>),
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuBindGroupEntry {
    pub binding: u32,
    pub resource: GpuBindingResource,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
pub struct GpuBindGroupDescriptor {
    pub layout: Resource<GpuBindGroupLayout>,
    pub entries: Vec<GpuBindGroupEntry>,
    pub label: Option<String>,
}

/// WIT `resource record-gpu-pipeline-constant-value` (pipeline constant map).
#[derive(Debug)]
pub struct RecordGpuPipelineConstantValue;

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuBlendFactor {
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
pub enum GpuBlendOperation {
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
pub struct GpuBlendComponent {
    pub operation: Option<GpuBlendOperation>,
    #[component(name = "src-factor")]
    pub src_factor: Option<GpuBlendFactor>,
    #[component(name = "dst-factor")]
    pub dst_factor: Option<GpuBlendFactor>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuBlendState {
    pub color: GpuBlendComponent,
    pub alpha: GpuBlendComponent,
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
pub struct GpuColorTargetState {
    pub format: GpuTextureFormat,
    pub blend: Option<GpuBlendState>,
    #[component(name = "write-mask")]
    pub write_mask: Option<GpuColorWrite>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuPrimitiveTopology {
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
pub enum GpuIndexFormat {
    #[component(name = "uint16")]
    Uint16,
    #[component(name = "uint32")]
    Uint32,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuFrontFace {
    #[component(name = "ccw")]
    Ccw,
    #[component(name = "cw")]
    Cw,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuCullMode {
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
pub struct GpuPrimitiveState {
    pub topology: Option<GpuPrimitiveTopology>,
    #[component(name = "strip-index-format")]
    pub strip_index_format: Option<GpuIndexFormat>,
    #[component(name = "front-face")]
    pub front_face: Option<GpuFrontFace>,
    #[component(name = "cull-mode")]
    pub cull_mode: Option<GpuCullMode>,
    #[component(name = "unclipped-depth")]
    pub unclipped_depth: Option<bool>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuStencilOperation {
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
pub struct GpuStencilFaceState {
    pub compare: Option<GpuCompareFunction>,
    #[component(name = "fail-op")]
    pub fail_op: Option<GpuStencilOperation>,
    #[component(name = "depth-fail-op")]
    pub depth_fail_op: Option<GpuStencilOperation>,
    #[component(name = "pass-op")]
    pub pass_op: Option<GpuStencilOperation>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuDepthStencilState {
    pub format: GpuTextureFormat,
    #[component(name = "depth-write-enabled")]
    pub depth_write_enabled: Option<bool>,
    #[component(name = "depth-compare")]
    pub depth_compare: Option<GpuCompareFunction>,
    #[component(name = "stencil-front")]
    pub stencil_front: Option<GpuStencilFaceState>,
    #[component(name = "stencil-back")]
    pub stencil_back: Option<GpuStencilFaceState>,
    #[component(name = "stencil-read-mask")]
    pub stencil_read_mask: Option<u32>,
    #[component(name = "stencil-write-mask")]
    pub stencil_write_mask: Option<u32>,
    #[component(name = "depth-bias")]
    pub depth_bias: Option<i32>,
    #[component(name = "depth-bias-slope-scale")]
    pub depth_bias_slope_scale: Option<f32>,
    #[component(name = "depth-bias-clamp")]
    pub depth_bias_clamp: Option<f32>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuMultisampleState {
    pub count: Option<u32>,
    pub mask: Option<u32>,
    #[component(name = "alpha-to-coverage-enabled")]
    pub alpha_to_coverage_enabled: Option<bool>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuVertexFormat {
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
pub struct GpuVertexAttribute {
    pub format: GpuVertexFormat,
    pub offset: u64,
    #[component(name = "shader-location")]
    pub shader_location: u32,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuVertexStepMode {
    #[component(name = "vertex")]
    Vertex,
    #[component(name = "instance")]
    Instance,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuVertexBufferLayout {
    #[component(name = "array-stride")]
    pub array_stride: u64,
    #[component(name = "step-mode")]
    pub step_mode: Option<GpuVertexStepMode>,
    pub attributes: Vec<GpuVertexAttribute>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuProgrammableStage {
    pub module: Resource<GpuShaderModule>,
    #[component(name = "entry-point")]
    pub entry_point: Option<String>,
    pub constants: Option<Resource<RecordGpuPipelineConstantValue>>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuVertexState {
    pub buffers: Option<Vec<Option<GpuVertexBufferLayout>>>,
    pub module: Resource<GpuShaderModule>,
    #[component(name = "entry-point")]
    pub entry_point: Option<String>,
    pub constants: Option<Resource<RecordGpuPipelineConstantValue>>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuFragmentState {
    pub targets: Vec<Option<GpuColorTargetState>>,
    pub module: Resource<GpuShaderModule>,
    #[component(name = "entry-point")]
    pub entry_point: Option<String>,
    pub constants: Option<Resource<RecordGpuPipelineConstantValue>>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
pub struct GpuComputePipelineDescriptor {
    pub compute: GpuProgrammableStage,
    pub layout: GpuLayoutMode,
    pub label: Option<String>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
pub struct GpuRenderPipelineDescriptor {
    pub vertex: GpuVertexState,
    pub primitive: Option<GpuPrimitiveState>,
    #[component(name = "depth-stencil")]
    pub depth_stencil: Option<GpuDepthStencilState>,
    pub multisample: Option<GpuMultisampleState>,
    pub fragment: Option<GpuFragmentState>,
    pub layout: GpuLayoutMode,
    pub label: Option<String>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuPipelineErrorReason {
    #[component(name = "validation")]
    Validation,
    #[component(name = "internal")]
    Internal,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
pub enum CreatePipelineErrorKind {
    #[component(name = "gpu-pipeline-error")]
    GpuPipelineError(GpuPipelineErrorReason),
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct CreatePipelineError {
    pub kind: CreatePipelineErrorKind,
    pub message: String,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
pub enum GetMappedRangeErrorKind {
    #[component(name = "operation-error")]
    OperationError,
    #[component(name = "range-error")]
    RangeError,
    #[component(name = "type-error")]
    TypeError,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GetMappedRangeError {
    pub kind: GetMappedRangeErrorKind,
    pub message: String,
}
