//! L2: `get-adapter` + `[method]gpu-adapter.request-device`
//! WIT: async (borrow<gpu-adapter>, option<gpu-device-descriptor>)
//!      -> result<own<gpu-device>, request-device-error>
//! Guest passes some(default-queue.label="l2"); drops own device on ok; harness 1.

use futures::channel::oneshot;
use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuAdapter;

#[derive(Debug)]
struct GpuDevice {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct RecordOptionGpuSize64;

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuFeatureName {
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

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuQueueDescriptor {
    label: Option<String>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuDeviceDescriptor {
    #[component(name = "required-features")]
    required_features: Option<Vec<GpuFeatureName>>,
    #[component(name = "required-limits")]
    required_limits: Option<Resource<RecordOptionGpuSize64>>,
    #[component(name = "default-queue")]
    default_queue: Option<GpuQueueDescriptor>,
    label: Option<String>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum RequestDeviceErrorKind {
    #[component(name = "type-error")]
    TypeError,
    #[component(name = "operation-error")]
    OperationError,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct RequestDeviceError {
    kind: RequestDeviceErrorKind,
    message: String,
}

struct TestHost {
    table: ResourceTable,
}

fn register_method_request_device(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    webgpu.resource(
        "gpu-adapter",
        ResourceType::host::<GpuAdapter>(),
        |mut store, rep| {
            let resource = Resource::<GpuAdapter>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "record-option-gpu-size64",
        ResourceType::host::<RecordOptionGpuSize64>(),
        |mut store, rep| {
            let resource = Resource::<RecordOptionGpuSize64>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-device",
        ResourceType::host::<GpuDevice>(),
        |mut store, rep| {
            let resource = Resource::<GpuDevice>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-adapter", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuAdapter)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap_concurrent(
        "[method]gpu-adapter.request-device",
        |accessor, (adapter, descriptor): (Resource<GpuAdapter>, Option<GpuDeviceDescriptor>)| {
            Box::pin(async move {
                accessor.with(|mut access| access.data_mut().table.get(&adapter).map(|_| ()))?;
                let desc = descriptor.expect("guest must pass descriptor=some this cut");
                assert!(desc.required_features.is_none());
                assert!(desc.required_limits.is_none());
                assert_eq!(
                    desc.default_queue.as_ref().and_then(|q| q.label.as_deref()),
                    Some("l2"),
                    "guest must pass default-queue.label=some(l2)"
                );
                assert!(desc.label.is_none());
                let (tx, rx) = oneshot::channel::<()>();
                std::thread::spawn(move || {
                    let _ = tx.send(());
                });
                let _ = rx.await;
                let resource = accessor
                    .with(|mut access| access.data_mut().table.push(GpuDevice { rep: 11 }))?;
                Ok((Ok::<_, RequestDeviceError>(resource),))
            })
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
fn wasi_webgpu_method_request_device_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_request_device.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_request_device(&mut linker)?;

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
        "guest run must drop result<own<gpu-device>, …> ok and return harness 1"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_request_device_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_request_device.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_request_device(&mut linker)?;

    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 1,
        "guest run must drop result ok and return harness 1 via call_async"
    );
    Ok(())
}
