//! P010-FIX / P010-GFXB: product-shaped linker omits fixture constructors
//! (`get-device`, `get-gpu-error`, `get-device-lost-info`). Pin WIT `get-gpu`
//! and `[method]gpu.request-adapter` stay on the product linker.

use futures::channel::oneshot;
use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct Gpu;

#[derive(Debug)]
struct GpuAdapter {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuPowerPreference {
    #[component(name = "low-power")]
    LowPower,
    #[component(name = "high-performance")]
    HighPerformance,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuRequestAdapterOptions {
    #[component(name = "feature-level")]
    feature_level: Option<String>,
    #[component(name = "power-preference")]
    power_preference: Option<GpuPowerPreference>,
    #[component(name = "force-fallback-adapter")]
    force_fallback_adapter: Option<bool>,
    #[component(name = "xr-compatible")]
    xr_compatible: Option<bool>,
}

struct TestHost {
    table: ResourceTable,
}

fn register_webgpu(
    linker: &mut Linker<TestHost>,
    fixture_get_device: bool,
) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    webgpu.resource("gpu", ResourceType::host::<Gpu>(), |mut store, rep| {
        let resource = Resource::<Gpu>::new_own(rep);
        store.data_mut().table.delete(resource)?;
        Ok(())
    })?;
    webgpu.resource(
        "gpu-adapter",
        ResourceType::host::<GpuAdapter>(),
        |mut store, rep| {
            let resource = Resource::<GpuAdapter>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-gpu", |mut store, ()| {
        let resource = store.data_mut().table.push(Gpu)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap_concurrent(
        "[method]gpu.request-adapter",
        |accessor, (gpu, _options): (Resource<Gpu>, Option<GpuRequestAdapterOptions>)| {
            Box::pin(async move {
                accessor.with(|mut access| access.data_mut().table.get(&gpu).map(|_| ()))?;
                let (tx, rx) = oneshot::channel::<()>();
                std::thread::spawn(move || {
                    let _ = tx.send(());
                });
                let _ = rx.await;
                Ok((None::<Resource<GpuAdapter>>,))
            })
        },
    )?;
    if fixture_get_device {
        webgpu.resource(
            "gpu-device",
            ResourceType::host::<Gpu>(),
            |mut store, rep| {
                let resource = Resource::<Gpu>::new_own(rep);
                store.data_mut().table.delete(resource)?;
                Ok(())
            },
        )?;
        webgpu.func_wrap("get-device", |mut store, ()| {
            let resource = store.data_mut().table.push(Gpu)?;
            Ok((resource,))
        })?;
    }
    Ok(())
}

fn load_method_request_adapter(engine: &Engine) -> wasmtime::Result<Component> {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_request_adapter.wasm"
    ))?;
    Ok(Component::new(engine, bytes)?)
}

fn new_store(engine: &Engine) -> Store<TestHost> {
    Store::new(
        engine,
        TestHost {
            table: ResourceTable::new(),
        },
    )
}

fn load_method_device_queue(engine: &Engine) -> wasmtime::Result<Component> {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_device_queue.wasm"
    ))?;
    Ok(Component::new(engine, bytes)?)
}

#[test]
fn product_linker_exports_pin_get_gpu() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let component = load_method_request_adapter(&engine)?;

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_webgpu(&mut linker, false)?;

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
    assert_eq!(v, 1, "product linker pin get-gpu chains request-adapter");
    Ok(())
}

#[test]
fn product_linker_rejects_get_device_fixture() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let component = load_method_device_queue(&engine)?;

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_webgpu(&mut linker, false)?;

    let mut store = new_store(&engine);
    let err = pollster::block_on(linker.instantiate_async(&mut store, &component))
        .expect_err("get-device guest must not instantiate on the product-shaped linker");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("get-device") || msg.contains("unknown import") || msg.contains("import"),
        "link error should mention missing get-device, got: {msg}"
    );
    Ok(())
}

#[test]
fn test_linker_still_chains_request_adapter() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let component = load_method_request_adapter(&engine)?;

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_webgpu(&mut linker, false)?;

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
        "product linker + pin get-gpu chains [method]gpu.request-adapter"
    );
    Ok(())
}
