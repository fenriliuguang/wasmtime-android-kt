//! L2: `get-texture` + `[method]gpu-texture.create-view`
//! WIT: `(borrow<gpu-texture>, option<gpu-texture-view-descriptor>) -> own<gpu-texture-view>`.
//! Guest passes some(descriptor) format=rgba8unorm, dimension=d2, aspect=all,
//! mip 0/1, array layers 0/1; drops own; `run` returns harness 1.

use wasmtime::component::{
    flags, Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, ComponentType, Lift, Lower)]
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
struct TestHost {
    table: ResourceTable,
}

fn register_method_texture_create_view(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
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
    webgpu.func_wrap("get-texture", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuTexture { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-texture.create-view",
        |mut caller,
         (texture, descriptor): (Resource<GpuTexture>, Option<GpuTextureViewDescriptor>)| {
            caller.data_mut().table.get(&texture).map(|_| ())?;
            let desc = descriptor.expect("guest must pass some(gpu-texture-view-descriptor)");
            assert!(
                matches!(desc.format, Some(GpuTextureFormat::Rgba8unorm)),
                "guest must pass format=rgba8unorm"
            );
            assert_eq!(
                desc.dimension,
                Some(GpuTextureViewDimension::D2),
                "guest must pass dimension=d2"
            );
            assert!(desc.usage.is_none());
            assert_eq!(
                desc.aspect,
                Some(GpuTextureAspect::All),
                "guest must pass aspect=all"
            );
            assert_eq!(
                desc.base_mip_level,
                Some(0),
                "guest must pass base-mip-level=some(0)"
            );
            assert_eq!(
                desc.mip_level_count,
                Some(1),
                "guest must pass mip-level-count=some(1)"
            );
            assert_eq!(
                desc.base_array_layer,
                Some(0),
                "guest must pass base-array-layer=some(0)"
            );
            assert_eq!(
                desc.array_layer_count,
                Some(1),
                "guest must pass array-layer-count=some(1)"
            );
            assert!(desc.swizzle.is_none());
            assert!(desc.label.is_none());
            let resource = caller.data_mut().table.push(GpuTextureView { rep: 41 })?;
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
fn wasi_webgpu_method_texture_create_view_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_texture_create_view.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_texture_create_view(&mut linker)?;
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
    Ok(())
}

#[test]
fn wasi_webgpu_method_texture_create_view_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_texture_create_view.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_texture_create_view(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 1,
        "guest run must drop owns and return harness 1 via call_async"
    );
    Ok(())
}
