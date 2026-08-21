//! Lane D: `get-device` + create-buffer + create-texture + create-view (none)
//! + create-command-encoder + queue + begin-render-pass + end + finish (none)
//! + submit.
//! Guest drops owns; `run` returns harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
struct GpuBuffer {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuTexture {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuTextureView {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuCommandEncoder {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuRenderPassEncoder {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuQueue {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuCommandBuffer {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuQuerySet;

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

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuBufferDescriptor {
    size: u64,
    usage: GpuBufferUsage,
    #[component(name = "mapped-at-creation")]
    mapped_at_creation: Option<bool>,
    label: Option<String>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuTextureViewDimension {
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

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuTextureDimension {
    #[component(name = "d1")]
    D1,
    #[component(name = "d2")]
    D2,
    #[component(name = "d3")]
    D3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuTextureAspect {
    #[component(name = "all")]
    All,
    #[component(name = "stencil-only")]
    StencilOnly,
    #[component(name = "depth-only")]
    DepthOnly,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuExtent3D {
    width: u32,
    height: Option<u32>,
    #[component(name = "depth-or-array-layers")]
    depth_or_array_layers: Option<u32>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuTextureDescriptor {
    size: GpuExtent3D,
    #[component(name = "mip-level-count")]
    mip_level_count: Option<u32>,
    #[component(name = "sample-count")]
    sample_count: Option<u32>,
    dimension: Option<GpuTextureDimension>,
    format: GpuTextureFormat,
    usage: GpuTextureUsage,
    #[component(name = "view-formats")]
    view_formats: Option<Vec<GpuTextureFormat>>,
    #[component(name = "texture-binding-view-dimension")]
    texture_binding_view_dimension: Option<GpuTextureViewDimension>,
    label: Option<String>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuTextureViewDescriptor {
    format: Option<GpuTextureFormat>,
    dimension: Option<GpuTextureViewDimension>,
    usage: Option<GpuTextureUsage>,
    aspect: Option<GpuTextureAspect>,
    #[component(name = "base-mip-level")]
    base_mip_level: Option<u32>,
    #[component(name = "mip-level-count")]
    mip_level_count: Option<u32>,
    #[component(name = "base-array-layer")]
    base_array_layer: Option<u32>,
    #[component(name = "array-layer-count")]
    array_layer_count: Option<u32>,
    swizzle: Option<String>,
    label: Option<String>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuCommandEncoderDescriptor {
    label: Option<String>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuLoadOp {
    #[component(name = "load")]
    Load,
    #[component(name = "clear")]
    Clear,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuStoreOp {
    #[component(name = "store")]
    Store,
    #[component(name = "discard")]
    Discard,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuColor {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuRenderPassColorAttachment {
    view: Resource<GpuTextureView>,
    #[component(name = "depth-slice")]
    depth_slice: Option<u32>,
    #[component(name = "resolve-target")]
    resolve_target: Option<Resource<GpuTextureView>>,
    #[component(name = "clear-value")]
    clear_value: Option<GpuColor>,
    #[component(name = "load-op")]
    load_op: GpuLoadOp,
    #[component(name = "store-op")]
    store_op: GpuStoreOp,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuRenderPassDepthStencilAttachment {
    view: Resource<GpuTextureView>,
    #[component(name = "depth-clear-value")]
    depth_clear_value: Option<f32>,
    #[component(name = "depth-load-op")]
    depth_load_op: Option<GpuLoadOp>,
    #[component(name = "depth-store-op")]
    depth_store_op: Option<GpuStoreOp>,
    #[component(name = "depth-read-only")]
    depth_read_only: Option<bool>,
    #[component(name = "stencil-clear-value")]
    stencil_clear_value: Option<u32>,
    #[component(name = "stencil-load-op")]
    stencil_load_op: Option<GpuLoadOp>,
    #[component(name = "stencil-store-op")]
    stencil_store_op: Option<GpuStoreOp>,
    #[component(name = "stencil-read-only")]
    stencil_read_only: Option<bool>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuRenderPassTimestampWrites {
    #[component(name = "query-set")]
    query_set: Resource<GpuQuerySet>,
    #[component(name = "beginning-of-pass-write-index")]
    beginning_of_pass_write_index: Option<u32>,
    #[component(name = "end-of-pass-write-index")]
    end_of_pass_write_index: Option<u32>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuRenderPassDescriptor {
    #[component(name = "color-attachments")]
    color_attachments: Vec<Option<GpuRenderPassColorAttachment>>,
    #[component(name = "depth-stencil-attachment")]
    depth_stencil_attachment: Option<GpuRenderPassDepthStencilAttachment>,
    #[component(name = "occlusion-query-set")]
    occlusion_query_set: Option<Resource<GpuQuerySet>>,
    #[component(name = "timestamp-writes")]
    timestamp_writes: Option<GpuRenderPassTimestampWrites>,
    #[component(name = "max-draw-count")]
    max_draw_count: Option<u64>,
    label: Option<String>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuCommandBufferDescriptor {
    label: Option<String>,
}

struct TestHost {
    table: ResourceTable,
}

fn register_dawn_render_slice(
    linker: &mut Linker<TestHost>,
    created_buffer: Arc<AtomicBool>,
    created_texture: Arc<AtomicBool>,
    created_view: Arc<AtomicBool>,
    began: Arc<AtomicBool>,
    ended: Arc<AtomicBool>,
    submitted: Arc<AtomicBool>,
) -> wasmtime::Result<()> {
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
        "gpu-buffer",
        ResourceType::host::<GpuBuffer>(),
        |mut store, rep| {
            let resource = Resource::<GpuBuffer>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-texture",
        ResourceType::host::<GpuTexture>(),
        |mut store, rep| {
            let resource = Resource::<GpuTexture>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-texture-view",
        ResourceType::host::<GpuTextureView>(),
        |mut store, rep| {
            let resource = Resource::<GpuTextureView>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-command-encoder",
        ResourceType::host::<GpuCommandEncoder>(),
        |mut store, rep| {
            let resource = Resource::<GpuCommandEncoder>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-render-pass-encoder",
        ResourceType::host::<GpuRenderPassEncoder>(),
        |mut store, rep| {
            let resource = Resource::<GpuRenderPassEncoder>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-queue",
        ResourceType::host::<GpuQueue>(),
        |mut store, rep| {
            let resource = Resource::<GpuQueue>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-command-buffer",
        ResourceType::host::<GpuCommandBuffer>(),
        |mut store, rep| {
            let resource = Resource::<GpuCommandBuffer>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-query-set",
        ResourceType::host::<GpuQuerySet>(),
        |mut store, rep| {
            let resource = Resource::<GpuQuerySet>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-device", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuDevice { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("[method]gpu-device.create-buffer", {
        let created_buffer = created_buffer.clone();
        move |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuBufferDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert_eq!(descriptor.size, 4, "guest must pass record size=4");
            assert!(
                descriptor.usage.contains(GpuBufferUsage::COPY_DST),
                "guest must pass COPY_DST"
            );
            assert!(
                descriptor.usage.contains(GpuBufferUsage::VERTEX),
                "guest must pass VERTEX"
            );
            assert!(descriptor.mapped_at_creation.is_none());
            assert!(descriptor.label.is_none());
            created_buffer.store(true, Ordering::SeqCst);
            let resource = caller.data_mut().table.push(GpuBuffer { rep: 31 })?;
            Ok((resource,))
        }
    })?;
    webgpu.func_wrap("[method]gpu-device.create-texture", {
        let created_texture = created_texture.clone();
        move |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuTextureDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert_eq!(descriptor.size.width, 1, "guest must pass width=1");
            assert_eq!(
                descriptor.size.height,
                Some(1),
                "guest must pass height=some(1)"
            );
            assert_eq!(
                descriptor.size.depth_or_array_layers,
                Some(1),
                "guest must pass depth=some(1)"
            );
            assert!(matches!(descriptor.format, GpuTextureFormat::Rgba8unorm));
            assert!(
                descriptor
                    .usage
                    .contains(GpuTextureUsage::RENDER_ATTACHMENT),
                "guest must pass RENDER_ATTACHMENT"
            );
            assert_eq!(
                descriptor.mip_level_count,
                Some(1),
                "guest must pass mip-level-count=some(1)"
            );
            assert_eq!(
                descriptor.sample_count,
                Some(1),
                "guest must pass sample-count=some(1)"
            );
            assert!(
                matches!(descriptor.dimension, Some(GpuTextureDimension::D2)),
                "guest must pass dimension=some(d2)"
            );
            assert!(descriptor.view_formats.is_none());
            assert!(descriptor.texture_binding_view_dimension.is_none());
            assert!(descriptor.label.is_none());
            created_texture.store(true, Ordering::SeqCst);
            let resource = caller.data_mut().table.push(GpuTexture { rep: 37 })?;
            Ok((resource,))
        }
    })?;
    webgpu.func_wrap("[method]gpu-texture.create-view", {
        let created_view = created_view.clone();
        move |mut caller,
              (texture, descriptor): (
            Resource<GpuTexture>,
            Option<GpuTextureViewDescriptor>,
        )| {
            caller.data_mut().table.get(&texture).map(|_| ())?;
            assert!(
                descriptor.is_none(),
                "guest must pass view descriptor none"
            );
            created_view.store(true, Ordering::SeqCst);
            let resource = caller
                .data_mut()
                .table
                .push(GpuTextureView { rep: 41 })?;
            Ok((resource,))
        }
    })?;
    webgpu.func_wrap(
        "[method]gpu-device.create-command-encoder",
        |mut caller,
         (device, descriptor): (Resource<GpuDevice>, Option<GpuCommandEncoderDescriptor>)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert!(
                descriptor.is_none(),
                "guest must pass encoder descriptor none"
            );
            let resource = caller
                .data_mut()
                .table
                .push(GpuCommandEncoder { rep: 17 })?;
            Ok((resource,))
        },
    )?;
    webgpu.func_wrap(
        "[method]gpu-device.queue",
        |mut caller, (device,): (Resource<GpuDevice>,)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            let resource = caller.data_mut().table.push(GpuQueue { rep: 3 })?;
            Ok((resource,))
        },
    )?;
    webgpu.func_wrap("[method]gpu-command-encoder.begin-render-pass", {
        let began = began.clone();
        move |mut caller,
              (encoder, descriptor): (Resource<GpuCommandEncoder>, GpuRenderPassDescriptor)| {
            caller.data_mut().table.get(&encoder).map(|_| ())?;
            assert_eq!(
                descriptor.color_attachments.len(),
                1,
                "guest must pass one color-attachment this slice"
            );
            let att = descriptor.color_attachments[0]
                .as_ref()
                .expect("guest color-attachment must be some");
            caller.data_mut().table.get(&att.view).map(|_| ())?;
            assert!(matches!(att.load_op, GpuLoadOp::Clear));
            assert!(matches!(att.store_op, GpuStoreOp::Store));
            assert!(att.depth_slice.is_none());
            assert!(att.resolve_target.is_none());
            let clear = att
                .clear_value
                .as_ref()
                .expect("guest must pass color clear-value");
            assert_eq!(clear.r, 0.0);
            assert_eq!(clear.g, 0.0);
            assert_eq!(clear.b, 0.0);
            assert_eq!(clear.a, 1.0);
            assert!(
                descriptor.depth_stencil_attachment.is_none(),
                "guest must omit depth-stencil attachment"
            );
            assert!(descriptor.occlusion_query_set.is_none());
            assert!(descriptor.timestamp_writes.is_none());
            assert!(descriptor.max_draw_count.is_none());
            assert!(descriptor.label.is_none());
            began.store(true, Ordering::SeqCst);
            let resource = caller
                .data_mut()
                .table
                .push(GpuRenderPassEncoder { rep: 29 })?;
            Ok((resource,))
        }
    })?;
    webgpu.func_wrap("[method]gpu-render-pass-encoder.end", {
        let ended = ended.clone();
        move |mut caller, (pass,): (Resource<GpuRenderPassEncoder>,)| {
            caller.data_mut().table.get(&pass).map(|_| ())?;
            ended.store(true, Ordering::SeqCst);
            Ok(())
        }
    })?;
    webgpu.func_wrap(
        "[method]gpu-command-encoder.finish",
        |mut caller,
         (encoder, descriptor): (
            Resource<GpuCommandEncoder>,
            Option<GpuCommandBufferDescriptor>,
        )| {
            caller.data_mut().table.get(&encoder).map(|_| ())?;
            assert!(
                descriptor.is_none(),
                "guest must pass finish descriptor none"
            );
            let resource = caller.data_mut().table.push(GpuCommandBuffer { rep: 19 })?;
            Ok((resource,))
        },
    )?;
    webgpu.func_wrap(
        "[method]gpu-queue.submit",
        move |mut caller,
              (queue, commands): (Resource<GpuQueue>, Vec<Resource<GpuCommandBuffer>>)| {
            caller.data_mut().table.get(&queue).map(|_| ())?;
            assert_eq!(
                commands.len(),
                1,
                "guest must pass a one-element command-buffer list"
            );
            caller.data_mut().table.get(&commands[0]).map(|_| ())?;
            submitted.store(true, Ordering::SeqCst);
            Ok(())
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
fn wasi_webgpu_method_dawn_render_slice_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_dawn_render_slice.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let created_buffer = Arc::new(AtomicBool::new(false));
    let created_texture = Arc::new(AtomicBool::new(false));
    let created_view = Arc::new(AtomicBool::new(false));
    let began = Arc::new(AtomicBool::new(false));
    let ended = Arc::new(AtomicBool::new(false));
    let submitted = Arc::new(AtomicBool::new(false));

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_dawn_render_slice(
        &mut linker,
        created_buffer.clone(),
        created_texture.clone(),
        created_view.clone(),
        began.clone(),
        ended.clone(),
        submitted.clone(),
    )?;

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
    assert_eq!(v, 1, "guest run must drop owns and return harness 1");
    assert!(
        created_buffer.load(Ordering::SeqCst),
        "create-buffer must run"
    );
    assert!(
        created_texture.load(Ordering::SeqCst),
        "create-texture must run"
    );
    assert!(created_view.load(Ordering::SeqCst), "create-view must run");
    assert!(began.load(Ordering::SeqCst), "begin-render-pass must run");
    assert!(ended.load(Ordering::SeqCst), "render-pass end must run");
    assert!(submitted.load(Ordering::SeqCst), "submit must run");
    Ok(())
}
