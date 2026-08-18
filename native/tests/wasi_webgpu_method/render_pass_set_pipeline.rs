//! S6+: `get-pass` + `get-render-pipeline` +
//! `[method]gpu-render-pass-encoder.set-pipeline`
//! WIT: `(borrow<gpu-render-pass-encoder>, borrow<gpu-render-pipeline>)`.
//! Guest borrows the pipeline; `run` returns harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{Component, Linker, Resource, ResourceTable, ResourceType};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuRenderPassEncoder;

#[derive(Debug)]
struct GpuRenderPipeline;

struct TestHost {
    table: ResourceTable,
}

fn register_method_render_pass_set_pipeline(
    linker: &mut Linker<TestHost>,
    set: Arc<AtomicBool>,
) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    webgpu.resource(
        "gpu-render-pass-encoder",
        ResourceType::host::<GpuRenderPassEncoder>(),
        |mut store, rep| {
            let resource = Resource::<GpuRenderPassEncoder>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-render-pipeline",
        ResourceType::host::<GpuRenderPipeline>(),
        |mut store, rep| {
            let resource = Resource::<GpuRenderPipeline>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-pass", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuRenderPassEncoder)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-render-pipeline", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuRenderPipeline)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-render-pass-encoder.set-pipeline",
        move |mut caller,
              (pass, pipeline): (Resource<GpuRenderPassEncoder>, Resource<GpuRenderPipeline>)| {
            caller.data_mut().table.get(&pass).map(|_| ())?;
            caller.data_mut().table.get(&pipeline).map(|_| ())?;
            set.store(true, Ordering::SeqCst);
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
fn wasi_webgpu_method_render_pass_set_pipeline_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_render_pass_set_pipeline.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let set = Arc::new(AtomicBool::new(false));

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_render_pass_set_pipeline(&mut linker, set.clone())?;

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
    assert_eq!(v, 1, "guest run must return harness 1 after set-pipeline");
    assert!(
        set.load(Ordering::SeqCst),
        "set-pipeline must have been called"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_render_pass_set_pipeline_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_render_pass_set_pipeline.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let set = Arc::new(AtomicBool::new(false));

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_render_pass_set_pipeline(&mut linker, set.clone())?;

    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 1, "guest run must return harness 1 via call_async");
    assert!(
        set.load(Ordering::SeqCst),
        "set-pipeline must have been called"
    );
    Ok(())
}
