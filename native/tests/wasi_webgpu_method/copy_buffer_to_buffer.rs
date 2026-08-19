//! L2: `get-encoder` + `get-buffer` +
//! `[method]gpu-command-encoder.copy-buffer-to-buffer`
//! WIT: `(borrow encoder, borrow src, option offset, borrow dst, option offset, option size)`.
//! Guest passes offsets some(0), size some(4); `run` returns harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{Component, Linker, Resource, ResourceTable, ResourceType};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuCommandEncoder;

#[derive(Debug)]
struct GpuBuffer;

struct TestHost {
    table: ResourceTable,
}

fn register_method_copy_buffer_to_buffer(
    linker: &mut Linker<TestHost>,
    copied: Arc<AtomicBool>,
) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
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
        "gpu-buffer",
        ResourceType::host::<GpuBuffer>(),
        |mut store, rep| {
            let resource = Resource::<GpuBuffer>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-encoder", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuCommandEncoder)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-buffer", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuBuffer)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-command-encoder.copy-buffer-to-buffer",
        move |mut caller,
              (encoder, source, source_offset, destination, destination_offset, size): (
            Resource<GpuCommandEncoder>,
            Resource<GpuBuffer>,
            Option<u64>,
            Resource<GpuBuffer>,
            Option<u64>,
            Option<u64>,
        )| {
            caller.data_mut().table.get(&encoder).map(|_| ())?;
            caller.data_mut().table.get(&source).map(|_| ())?;
            caller.data_mut().table.get(&destination).map(|_| ())?;
            assert_eq!(
                source_offset,
                Some(0),
                "guest must pass source-offset=some(0)"
            );
            assert_eq!(
                destination_offset,
                Some(0),
                "guest must pass destination-offset=some(0)"
            );
            assert_eq!(size, Some(4), "guest must pass size=some(4)");
            copied.store(true, Ordering::SeqCst);
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
fn wasi_webgpu_method_copy_buffer_to_buffer_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_copy_buffer_to_buffer.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let copied = Arc::new(AtomicBool::new(false));

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_copy_buffer_to_buffer(&mut linker, copied.clone())?;

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
    assert_eq!(v, 1, "guest run must return harness 1 after copy");
    assert!(
        copied.load(Ordering::SeqCst),
        "copy-buffer-to-buffer must have been called"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_copy_buffer_to_buffer_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_copy_buffer_to_buffer.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let copied = Arc::new(AtomicBool::new(false));

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_copy_buffer_to_buffer(&mut linker, copied.clone())?;

    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 1, "guest run must return harness 1 via call_async");
    assert!(
        copied.load(Ordering::SeqCst),
        "copy-buffer-to-buffer must have been called"
    );
    Ok(())
}
