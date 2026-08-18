//! S6+: `get-pass` + `[method]gpu-render-pass-encoder.set-blend-constant`
//! WIT: `(borrow, gpu-color)`. Guest passes r=g=b=0 a=1; harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuRenderPassEncoder;

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuColor {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

struct TestHost {
    table: ResourceTable,
}

fn register(linker: &mut Linker<TestHost>, called: Arc<AtomicBool>) -> wasmtime::Result<()> {
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
    webgpu.func_wrap("get-pass", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuRenderPassEncoder)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-render-pass-encoder.set-blend-constant",
        move |mut caller, (pass, color): (Resource<GpuRenderPassEncoder>, GpuColor)| {
            caller.data_mut().table.get(&pass).map(|_| ())?;
            assert_eq!((color.r, color.g, color.b, color.a), (0.0, 0.0, 0.0, 1.0));
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
fn wasi_webgpu_method_render_pass_set_blend_constant_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_render_pass_set_blend_constant.wasm"
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
    assert_eq!(
        v, 1,
        "guest run must return harness 1 after set-blend-constant"
    );
    assert!(
        called.load(Ordering::SeqCst),
        "set-blend-constant must have been called"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_render_pass_set_blend_constant_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_render_pass_set_blend_constant.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let called = Arc::new(AtomicBool::new(false));
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register(&mut linker, called.clone())?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 1, "guest run must return harness 1 via call_async");
    assert!(
        called.load(Ordering::SeqCst),
        "set-blend-constant must have been called"
    );
    Ok(())
}
