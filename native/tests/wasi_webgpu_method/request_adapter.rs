//! S2: `get-gpu` + `[method]gpu.request-adapter`
//! WIT: async (borrow<gpu>, option<gpu-request-adapter-options>) -> option<own<gpu-adapter>>
//! Guest passes none; drops own adapter; `run` returns harness 1.

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

fn register_method_request_adapter(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
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
                let resource = accessor
                    .with(|mut access| access.data_mut().table.push(GpuAdapter { rep: 7 }))?;
                Ok((Some(resource),))
            })
        },
    )?;
    Ok(())
}

fn register_method_request_adapter_none(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
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
fn wasi_webgpu_method_request_adapter_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_request_adapter.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_request_adapter(&mut linker)?;

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
        "guest run must drop option<own<gpu-adapter>> and return harness 1"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_request_adapter_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_request_adapter.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_request_adapter(&mut linker)?;

    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 1,
        "guest run must drop option<own<gpu-adapter>> and return harness 1 via call_async"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_request_adapter_unwired_none() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_request_adapter.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_request_adapter_none(&mut linker)?;

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
        "guest must treat option none as success and return harness 1"
    );
    Ok(())
}
