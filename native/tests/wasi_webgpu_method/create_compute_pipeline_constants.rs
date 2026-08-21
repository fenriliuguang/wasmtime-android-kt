//! L2: `get-device` + `get-shader-module` + `[constructor]record-gpu-pipeline-constant-value`
//! + `add` + `[method]gpu-device.create-compute-pipeline` with `compute.constants = some(record)`.
//! Guest add key=`c` value=`1.0`; layout=auto; label=`l2`; drops own; `run` returns harness 1.

use std::collections::HashMap;
use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuDevice {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuShaderModule {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuPipelineLayout {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuComputePipeline {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct RecordGpuPipelineConstantValue;

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum GpuLayoutMode {
    #[component(name = "specific")]
    Specific(Resource<GpuPipelineLayout>),
    #[component(name = "auto")]
    Auto,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuProgrammableStage {
    module: Resource<GpuShaderModule>,
    #[component(name = "entry-point")]
    entry_point: Option<String>,
    constants: Option<Resource<RecordGpuPipelineConstantValue>>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuComputePipelineDescriptor {
    compute: GpuProgrammableStage,
    layout: GpuLayoutMode,
    label: Option<String>,
}

struct TestHost {
    table: ResourceTable,
    added: HashMap<u32, (String, f64)>,
}

fn register_method_create_compute_pipeline_constants(
    linker: &mut Linker<TestHost>,
) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    webgpu.resource(
        "gpu-device",
        ResourceType::host::<GpuDevice>(),
        |mut store, rep| {
            let resource = Resource::<GpuDevice>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-shader-module",
        ResourceType::host::<GpuShaderModule>(),
        |mut store, rep| {
            let resource = Resource::<GpuShaderModule>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-pipeline-layout",
        ResourceType::host::<GpuPipelineLayout>(),
        |mut store, rep| {
            let resource = Resource::<GpuPipelineLayout>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-compute-pipeline",
        ResourceType::host::<GpuComputePipeline>(),
        |mut store, rep| {
            let resource = Resource::<GpuComputePipeline>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "record-gpu-pipeline-constant-value",
        ResourceType::host::<RecordGpuPipelineConstantValue>(),
        |mut store, rep| {
            let resource = Resource::<RecordGpuPipelineConstantValue>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-device", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuDevice { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-shader-module", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuShaderModule { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("[constructor]record-gpu-pipeline-constant-value", |mut store, ()| {
        let resource = store
            .data_mut()
            .table
            .push(RecordGpuPipelineConstantValue)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]record-gpu-pipeline-constant-value.add",
        |mut caller, (record, key, value): (Resource<RecordGpuPipelineConstantValue>, String, f64)| {
            caller.data_mut().table.get(&record).map(|_| ())?;
            caller.data_mut().added.insert(record.rep(), (key, value));
            Ok(())
        },
    )?;
    webgpu.func_wrap(
        "[method]gpu-device.create-compute-pipeline",
        |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuComputePipelineDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            caller
                .data_mut()
                .table
                .get(&descriptor.compute.module)
                .map(|_| ())?;
            assert!(matches!(descriptor.layout, GpuLayoutMode::Auto));
            assert_eq!(descriptor.compute.entry_point.as_deref(), Some("main"));
            let rec = descriptor
                .compute
                .constants
                .as_ref()
                .expect("guest must pass compute.constants record");
            caller.data_mut().table.get(rec).map(|_| ())?;
            let (key, value) = caller
                .data()
                .added
                .get(&rec.rep())
                .cloned()
                .expect("guest must add before create-compute-pipeline");
            assert_eq!(key, "c");
            assert_eq!(value, 1.0);
            assert_eq!(descriptor.label.as_deref(), Some("l2"));
            let resource = caller
                .data_mut()
                .table
                .push(GpuComputePipeline { rep: 74 })?;
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
            added: HashMap::new(),
        },
    )
}

#[test]
fn wasi_webgpu_method_create_compute_pipeline_constants_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_compute_pipeline_constants.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_compute_pipeline_constants(&mut linker)?;
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
        "guest run must drop own<gpu-compute-pipeline> and return harness 1"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_create_compute_pipeline_constants_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_compute_pipeline_constants.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_compute_pipeline_constants(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 1,
        "guest run must drop own<gpu-compute-pipeline> and return harness 1 via call_async"
    );
    Ok(())
}
