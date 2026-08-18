//! S6+: `get-queue` + `get-buffer` + `[method]gpu-queue.write-buffer-with-copy`
//! WIT: `(borrow queue, borrow buffer, u64, list<u8>, option u64, option u64)
//!      -> result<_, write-buffer-error>`.
//! Guest passes offset=0, empty data, offset/size none; `run` returns harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuQueue;

#[derive(Debug)]
struct GpuBuffer;

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum WriteBufferErrorKind {
    #[component(name = "operation-error")]
    OperationError,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct WriteBufferError {
    kind: WriteBufferErrorKind,
    message: String,
}

struct TestHost {
    table: ResourceTable,
}

fn register_method_write_buffer(
    linker: &mut Linker<TestHost>,
    written: Arc<AtomicBool>,
) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
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
        "gpu-buffer",
        ResourceType::host::<GpuBuffer>(),
        |mut store, rep| {
            let resource = Resource::<GpuBuffer>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-queue", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuQueue)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-buffer", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuBuffer)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-queue.write-buffer-with-copy",
        move |mut caller,
              (queue, buffer, offset, data, data_offset, size): (
            Resource<GpuQueue>,
            Resource<GpuBuffer>,
            u64,
            Vec<u8>,
            Option<u64>,
            Option<u64>,
        )| {
            caller.data_mut().table.get(&queue).map(|_| ())?;
            caller.data_mut().table.get(&buffer).map(|_| ())?;
            assert_eq!(offset, 0, "guest must pass buffer-offset=0");
            assert!(data.is_empty(), "guest must pass empty data this slice");
            assert!(
                data_offset.is_none(),
                "guest must pass data-offset=none this slice"
            );
            assert!(size.is_none(), "guest must pass size=none this slice");
            written.store(true, Ordering::SeqCst);
            Ok((Ok::<(), WriteBufferError>(()),))
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
fn wasi_webgpu_method_write_buffer_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_write_buffer.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let written = Arc::new(AtomicBool::new(false));
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_write_buffer(&mut linker, written.clone())?;
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
        "guest run must return harness 1 after write-buffer ok"
    );
    assert!(
        written.load(Ordering::SeqCst),
        "write-buffer-with-copy must have been called"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_write_buffer_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_write_buffer.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let written = Arc::new(AtomicBool::new(false));
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_write_buffer(&mut linker, written.clone())?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 1, "guest run must return harness 1 via call_async");
    assert!(
        written.load(Ordering::SeqCst),
        "write-buffer-with-copy must have been called"
    );
    Ok(())
}
