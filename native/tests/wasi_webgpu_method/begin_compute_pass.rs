//! L2: `get-encoder` + `get-query-set` + `[method]gpu-command-encoder.begin-compute-pass`
//! WIT: `(borrow<gpu-command-encoder>, option<gpu-compute-pass-descriptor>)
//!      -> own<gpu-compute-pass-encoder>`.
//! Guest passes some(descriptor) timestamp-writes beginning=0 end=1;
//! drops own pass + query-set; `run` returns harness 1.

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuCommandEncoder {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuComputePassEncoder {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuQuerySet;

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuComputePassTimestampWrites {
    #[component(name = "query-set")]
    query_set: Resource<GpuQuerySet>,
    #[component(name = "beginning-of-pass-write-index")]
    beginning_of_pass_write_index: Option<u32>,
    #[component(name = "end-of-pass-write-index")]
    end_of_pass_write_index: Option<u32>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuComputePassDescriptor {
    #[component(name = "timestamp-writes")]
    timestamp_writes: Option<GpuComputePassTimestampWrites>,
    label: Option<String>,
}

struct TestHost {
    table: ResourceTable,
}

fn register_method_begin_compute_pass(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
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
        "gpu-query-set",
        ResourceType::host::<GpuQuerySet>(),
        |mut store, rep| {
            let resource = Resource::<GpuQuerySet>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-compute-pass-encoder",
        ResourceType::host::<GpuComputePassEncoder>(),
        |mut store, rep| {
            let resource = Resource::<GpuComputePassEncoder>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-encoder", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuCommandEncoder { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-query-set", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuQuerySet)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-command-encoder.begin-compute-pass",
        |mut caller,
         (encoder, descriptor): (
            Resource<GpuCommandEncoder>,
            Option<GpuComputePassDescriptor>,
        )| {
            caller.data_mut().table.get(&encoder).map(|_| ())?;
            let desc = descriptor.expect("guest must pass some(descriptor) this slice");
            assert!(desc.label.is_none());
            let ts = desc
                .timestamp_writes
                .expect("guest must pass some(timestamp-writes)");
            caller.data_mut().table.get(&ts.query_set).map(|_| ())?;
            assert_eq!(ts.beginning_of_pass_write_index, Some(0));
            assert_eq!(ts.end_of_pass_write_index, Some(1));
            let resource = caller
                .data_mut()
                .table
                .push(GpuComputePassEncoder { rep: 79 })?;
            Ok((resource,))
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
fn wasi_webgpu_method_begin_compute_pass_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_begin_compute_pass.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_begin_compute_pass(&mut linker)?;

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
    assert_eq!(v, 1, "guest run must drop owns and return harness 1");
    Ok(())
}

#[test]
fn wasi_webgpu_method_begin_compute_pass_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_begin_compute_pass.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_begin_compute_pass(&mut linker)?;

    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 1,
        "guest run must drop owns and return harness 1 via call_async"
    );
    Ok(())
}
