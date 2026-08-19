//! L2: `get-compute-pass` + `[method]gpu-compute-pass-encoder.dispatch-workgroups`
//! WIT: `(borrow, x: u32, y: option<u32>, z: option<u32>)`.
//! Guest passes x=1, y=some(1), z=some(1); `run` returns harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{Component, Linker, Resource, ResourceTable, ResourceType};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuComputePassEncoder;

struct TestHost {
    table: ResourceTable,
}

fn register_method_compute_pass_dispatch_workgroups(
    linker: &mut Linker<TestHost>,
    dispatched: Arc<AtomicBool>,
) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    webgpu.resource(
        "gpu-compute-pass-encoder",
        ResourceType::host::<GpuComputePassEncoder>(),
        |mut store, rep| {
            let resource = Resource::<GpuComputePassEncoder>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-compute-pass", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuComputePassEncoder)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-compute-pass-encoder.dispatch-workgroups",
        move |mut caller,
              (pass, x, y, z): (
            Resource<GpuComputePassEncoder>,
            u32,
            Option<u32>,
            Option<u32>,
        )| {
            caller.data_mut().table.get(&pass).map(|_| ())?;
            assert_eq!(x, 1, "guest must pass workgroup-count-x=1");
            assert_eq!(y, Some(1), "guest must pass y=some(1) this slice");
            assert_eq!(z, Some(1), "guest must pass z=some(1) this slice");
            dispatched.store(true, Ordering::SeqCst);
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
fn wasi_webgpu_method_compute_pass_dispatch_workgroups_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_compute_pass_dispatch_workgroups.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let dispatched = Arc::new(AtomicBool::new(false));

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_compute_pass_dispatch_workgroups(&mut linker, dispatched.clone())?;

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
    assert_eq!(v, 1, "guest run must return harness 1 after dispatch");
    assert!(
        dispatched.load(Ordering::SeqCst),
        "dispatch-workgroups must have been called"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_compute_pass_dispatch_workgroups_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_compute_pass_dispatch_workgroups.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let dispatched = Arc::new(AtomicBool::new(false));

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_compute_pass_dispatch_workgroups(&mut linker, dispatched.clone())?;

    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 1, "guest run must return harness 1 via call_async");
    assert!(
        dispatched.load(Ordering::SeqCst),
        "dispatch-workgroups must have been called"
    );
    Ok(())
}
