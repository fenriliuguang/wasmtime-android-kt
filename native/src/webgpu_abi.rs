//! WIT types for `wasi:webgpu@0.3.0-rc.2` used by canonical-shape slices.
//! S2: `gpu-request-adapter-options` + `gpu-power-preference`.
//! S3: `gpu-device-descriptor` + `request-device-error` (+ feature enum / queue descriptor).
//! S4: `gpu-buffer-descriptor` + `gpu-buffer-usage` flags.
//! S6: `gpu-command-encoder-descriptor`.
//! S7: `gpu-command-buffer-descriptor`.
//! S8: sampler / texture-view / compute-pass option descriptors (guest passes none).

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
