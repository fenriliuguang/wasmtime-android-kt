//! L2: `get-pass` + `get-bind-group` +
//! `[method]gpu-render-pass-encoder.set-bind-group`
//! WIT: `(borrow, index, option bind-group, option offsets, option start, option length)
//!      -> result<_, set-bind-group-error>`.
//! Guest passes index=0, bind-group=some, offsets none; `run` returns harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuRenderPassEncoder;

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

fn register_method_render_pass_set_bind_group(
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
        "gpu-bind-group",
        ResourceType::host::<GpuBindGroup>(),
        |mut store, rep| {
            let resource = Resource::<GpuBindGroup>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-pass", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuRenderPassEncoder)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-bind-group", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuBindGroup)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-render-pass-encoder.set-bind-group",
        move |mut caller,
              (pass, index, bind_group, offsets, start, length): (
            Resource<GpuRenderPassEncoder>,
            u32,
            Option<Resource<GpuBindGroup>>,
            Option<Vec<u32>>,
            Option<u64>,
            Option<u32>,
        )| {
            caller.data_mut().table.get(&pass).map(|_| ())?;
            assert_eq!(index, 0, "guest must pass index=0");
            let bind_group = bind_group.expect("guest must pass bind-group=some this slice");
            caller.data_mut().table.get(&bind_group).map(|_| ())?;
            assert!(offsets.is_none(), "guest must pass offsets=none this slice");
            assert!(start.is_none(), "guest must pass start=none this slice");
            assert!(length.is_none(), "guest must pass length=none this slice");
            set.store(true, Ordering::SeqCst);
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
fn wasi_webgpu_method_render_pass_set_bind_group_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_render_pass_set_bind_group.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let set = Arc::new(AtomicBool::new(false));

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_render_pass_set_bind_group(&mut linker, set.clone())?;

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
        "guest run must return harness 1 after set-bind-group ok"
    );
    assert!(
        set.load(Ordering::SeqCst),
        "set-bind-group must have been called"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_render_pass_set_bind_group_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_render_pass_set_bind_group.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let set = Arc::new(AtomicBool::new(false));

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_render_pass_set_bind_group(&mut linker, set.clone())?;

    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 1, "guest run must return harness 1 via call_async");
    assert!(
        set.load(Ordering::SeqCst),
        "set-bind-group must have been called"
    );
    Ok(())
}
