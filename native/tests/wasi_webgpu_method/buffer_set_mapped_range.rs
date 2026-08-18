//! S6+: `get-buffer` + `[method]gpu-buffer.get-mapped-range-set-with-copy`
//! WIT: `(borrow<gpu-buffer>, list<u8>, option<u64>, option<u64>)
//!      -> result<_, get-mapped-range-error>`.
//! Guest passes empty data, offset/size=none; `run` returns harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuBuffer;

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum GetMappedRangeErrorKind {
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
struct GetMappedRangeError {
    kind: GetMappedRangeErrorKind,
    message: String,
}

struct TestHost {
    table: ResourceTable,
}

fn register_method_buffer_set_mapped_range(
    linker: &mut Linker<TestHost>,
    called: Arc<AtomicBool>,
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
    webgpu.func_wrap(
        "[method]gpu-buffer.get-mapped-range-set-with-copy",
        move |mut caller,
              (buffer, data, offset, size): (
            Resource<GpuBuffer>,
            Vec<u8>,
            Option<u64>,
            Option<u64>,
        )| {
            caller.data_mut().table.get(&buffer).map(|_| ())?;
            assert!(data.is_empty(), "guest must pass empty data this slice");
            assert!(offset.is_none(), "guest must pass offset=none this slice");
            assert!(size.is_none(), "guest must pass size=none this slice");
            called.store(true, Ordering::SeqCst);
            Ok((Ok::<(), GetMappedRangeError>(()),))
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
fn wasi_webgpu_method_buffer_set_mapped_range_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_buffer_set_mapped_range.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let called = Arc::new(AtomicBool::new(false));
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_buffer_set_mapped_range(&mut linker, called.clone())?;
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
        "guest run must return harness 1 after set-mapped-range ok"
    );
    assert!(
        called.load(Ordering::SeqCst),
        "get-mapped-range-set-with-copy must have been called"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_buffer_set_mapped_range_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_buffer_set_mapped_range.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let called = Arc::new(AtomicBool::new(false));
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_buffer_set_mapped_range(&mut linker, called.clone())?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 1, "guest run must return harness 1 via call_async");
    assert!(
        called.load(Ordering::SeqCst),
        "get-mapped-range-set-with-copy must have been called"
    );
    Ok(())
}
