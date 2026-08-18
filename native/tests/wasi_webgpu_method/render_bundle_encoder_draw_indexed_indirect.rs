//! S6+: `get-render-bundle-encoder` + `get-buffer` +
//! `[method]gpu-render-bundle-encoder.draw-indexed-indirect`
//! WIT: `(borrow, borrow<gpu-buffer>, indirect-offset: u64)`.
//! Guest passes offset=0; harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{Component, Linker, Resource, ResourceTable, ResourceType};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuRenderBundleEncoder;

#[derive(Debug)]
struct GpuBuffer;

struct TestHost {
    table: ResourceTable,
}

fn register(linker: &mut Linker<TestHost>, called: Arc<AtomicBool>) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    webgpu.resource(
        "gpu-render-bundle-encoder",
        ResourceType::host::<GpuRenderBundleEncoder>(),
        |mut store, rep| {
            let resource = Resource::<GpuRenderBundleEncoder>::new_own(rep);
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
    webgpu.func_wrap("get-render-bundle-encoder", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuRenderBundleEncoder)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-buffer", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuBuffer)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-render-bundle-encoder.draw-indexed-indirect",
        move |mut caller,
              (encoder, buffer, offset): (
            Resource<GpuRenderBundleEncoder>,
            Resource<GpuBuffer>,
            u64,
        )| {
            caller.data_mut().table.get(&encoder).map(|_| ())?;
            caller.data_mut().table.get(&buffer).map(|_| ())?;
            assert_eq!(offset, 0);
            called.store(true, Ordering::SeqCst);
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
fn wasi_webgpu_method_render_bundle_encoder_draw_indexed_indirect_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_render_bundle_encoder_draw_indexed_indirect.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let called = Arc::new(AtomicBool::new(false));
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register(&mut linker, called.clone())?;
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
    assert_eq!(v, 1);
    assert!(called.load(Ordering::SeqCst));
    Ok(())
}

#[test]
fn wasi_webgpu_method_render_bundle_encoder_draw_indexed_indirect_call_async(
) -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_render_bundle_encoder_draw_indexed_indirect.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let called = Arc::new(AtomicBool::new(false));
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register(&mut linker, called.clone())?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 1);
    assert!(called.load(Ordering::SeqCst));
    Ok(())
}
