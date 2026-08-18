//! S6+: `get-render-bundle-encoder` + `get-bind-group` +
//! `[method]gpu-render-bundle-encoder.set-bind-group`
//! WIT: `(borrow, index, option bind-group, option offsets, …) -> result<_, set-bind-group-error>`.
//! Guest passes index=0, bind-group=some, offsets none; harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuRenderBundleEncoder;

#[derive(Debug)]
struct GpuBindGroup;

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum SetBindGroupErrorKind {
    #[component(name = "range-error")]
    RangeError,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct SetBindGroupError {
    kind: SetBindGroupErrorKind,
    message: String,
}

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
        "gpu-bind-group",
        ResourceType::host::<GpuBindGroup>(),
        |mut store, rep| {
            let resource = Resource::<GpuBindGroup>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-render-bundle-encoder", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuRenderBundleEncoder)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-bind-group", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuBindGroup)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-render-bundle-encoder.set-bind-group",
        move |mut caller,
              (encoder, index, bind_group, offsets, start, length): (
            Resource<GpuRenderBundleEncoder>,
            u32,
            Option<Resource<GpuBindGroup>>,
            Option<Vec<u32>>,
            Option<u64>,
            Option<u32>,
        )| {
            caller.data_mut().table.get(&encoder).map(|_| ())?;
            assert_eq!(index, 0);
            let bind_group = bind_group.expect("guest must pass bind-group=some");
            caller.data_mut().table.get(&bind_group).map(|_| ())?;
            assert!(offsets.is_none());
            assert!(start.is_none());
            assert!(length.is_none());
            called.store(true, Ordering::SeqCst);
            Ok((Ok::<(), SetBindGroupError>(()),))
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
fn wasi_webgpu_method_render_bundle_encoder_set_bind_group_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_render_bundle_encoder_set_bind_group.wasm"
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
fn wasi_webgpu_method_render_bundle_encoder_set_bind_group_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_render_bundle_encoder_set_bind_group.wasm"
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
