//! S6+: `get-buffer` + `[method]gpu-buffer.map-async`
//! WIT: async `(borrow<gpu-buffer>, gpu-map-mode, option<u64>, option<u64>)
//!      -> result<_, map-async-error>`.
//! Guest passes mode=READ, offset/size=none; `run` returns harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures::channel::oneshot;
use wasmtime::component::{
    flags, Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuBuffer;

flags! {
    GpuMapMode {
        #[component(name = "read")]
        const READ;
        #[component(name = "write")]
        const WRITE;
    }
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum MapAsyncErrorKind {
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
struct MapAsyncError {
    kind: MapAsyncErrorKind,
    message: String,
}

struct TestHost {
    table: ResourceTable,
}

fn register_method_buffer_map_async(
    linker: &mut Linker<TestHost>,
    mapped: Arc<AtomicBool>,
) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    webgpu.resource(
        "gpu-buffer",
        ResourceType::host::<GpuBuffer>(),
        |mut store, rep| {
            let resource = Resource::<GpuBuffer>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-buffer", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuBuffer)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap_concurrent(
        "[method]gpu-buffer.map-async",
        move |accessor,
              (buffer, mode, offset, size): (
            Resource<GpuBuffer>,
            GpuMapMode,
            Option<u64>,
            Option<u64>,
        )| {
            let mapped = mapped.clone();
            Box::pin(async move {
                accessor.with(|mut access| access.data_mut().table.get(&buffer).map(|_| ()))?;
                assert!(
                    mode.contains(GpuMapMode::READ),
                    "guest must pass map-mode READ"
                );
                assert!(offset.is_none(), "guest must pass offset=none this slice");
                assert!(size.is_none(), "guest must pass size=none this slice");
                let (tx, rx) = oneshot::channel::<()>();
                std::thread::spawn(move || {
                    let _ = tx.send(());
                });
                let _ = rx.await;
                mapped.store(true, Ordering::SeqCst);
                Ok((Ok::<(), MapAsyncError>(()),))
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
fn wasi_webgpu_method_buffer_map_async_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_buffer_map_async.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mapped = Arc::new(AtomicBool::new(false));

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_buffer_map_async(&mut linker, mapped.clone())?;

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
    assert_eq!(v, 1, "guest run must return harness 1 after map-async ok");
    assert!(
        mapped.load(Ordering::SeqCst),
        "map-async must have been called"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_buffer_map_async_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_buffer_map_async.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mapped = Arc::new(AtomicBool::new(false));

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_buffer_map_async(&mut linker, mapped.clone())?;

    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 1, "guest run must return harness 1 via call_async");
    assert!(
        mapped.load(Ordering::SeqCst),
        "map-async must have been called"
    );
    Ok(())
}
