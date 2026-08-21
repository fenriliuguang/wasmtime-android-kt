//! S6+: `get-device` + `[method]gpu-device.create-texture`
//! WIT: `(borrow<gpu-device>, gpu-texture-descriptor) -> own<gpu-texture>`.
//! Guest passes size 1x1x1, mip=2, sample=1, dimension=d2, rgba8unorm,
//! render-attachment; drops own; `run` returns harness 1.

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
struct GpuTexture {
    #[allow(dead_code)]
    rep: u32,
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

struct TestHost {
    table: ResourceTable,
}

fn register_method_create_texture(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
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
        "gpu-texture",
        ResourceType::host::<GpuTexture>(),
        |mut store, rep| {
            let resource = Resource::<GpuTexture>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-device", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuDevice { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-device.create-texture",
        |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuTextureDescriptor)| {
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
                Some(2),
                "guest must pass mip-level-count=some(2)"
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
            let resource = caller.data_mut().table.push(GpuTexture { rep: 37 })?;
            Ok((resource,))
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
fn wasi_webgpu_method_create_texture_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_texture.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_texture(&mut linker)?;
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
    assert_eq!(
        v, 1,
        "guest run must drop own<gpu-texture> and return harness 1"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_create_texture_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_texture.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_texture(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 1,
        "guest run must drop own<gpu-texture> and return harness 1 via call_async"
    );
    Ok(())
}
