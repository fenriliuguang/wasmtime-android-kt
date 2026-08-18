//! S6+: `get-device` + `get-shader-module` + `[method]gpu-device.create-compute-pipeline-async`
//! WIT: async `(borrow<gpu-device>, gpu-compute-pipeline-descriptor)
//!      -> result<own<gpu-compute-pipeline>, create-pipeline-error>`.
//! Guest passes shader borrow, layout=auto; drops own on ok; `run` returns harness 1.
//! True CM async.

use futures::channel::oneshot;
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

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuPipelineErrorReason {
    #[component(name = "validation")]
    Validation,
    #[component(name = "internal")]
    Internal,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum CreatePipelineErrorKind {
    #[component(name = "gpu-pipeline-error")]
    GpuPipelineError(GpuPipelineErrorReason),
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct CreatePipelineError {
    kind: CreatePipelineErrorKind,
    message: String,
}

struct TestHost {
    table: ResourceTable,
}

fn register_method_create_compute_pipeline(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
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
    webgpu.func_wrap_concurrent(
        "[method]gpu-device.create-compute-pipeline-async",
        |accessor, (device, descriptor): (Resource<GpuDevice>, GpuComputePipelineDescriptor)| {
            Box::pin(async move {
                accessor.with(|mut access| access.data_mut().table.get(&device).map(|_| ()))?;
                accessor.with(|mut access| {
                    access
                        .data_mut()
                        .table
                        .get(&descriptor.compute.module)
                        .map(|_| ())
                })?;
                assert!(matches!(descriptor.layout, GpuLayoutMode::Auto));
                assert!(descriptor.compute.entry_point.is_none());
                assert!(descriptor.compute.constants.is_none());
                assert!(descriptor.label.is_none());
                let (tx, rx) = oneshot::channel::<()>();
                std::thread::spawn(move || {
                    let _ = tx.send(());
                });
                let _ = rx.await;
                let resource = accessor.with(|mut access| {
                    access.data_mut().table.push(GpuComputePipeline { rep: 73 })
                })?;
                Ok((Ok::<_, CreatePipelineError>(resource),))
            })
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
fn wasi_webgpu_method_create_compute_pipeline_async_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_compute_pipeline_async.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_compute_pipeline(&mut linker)?;
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
        "guest run must drop own<gpu-compute-pipeline> on ok and return harness 1"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_create_compute_pipeline_async_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_compute_pipeline_async.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_compute_pipeline(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 1,
        "guest run must drop own<gpu-compute-pipeline> on ok and return harness 1 via call_async"
    );
    Ok(())
}
