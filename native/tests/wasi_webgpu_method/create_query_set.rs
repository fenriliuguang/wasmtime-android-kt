//! S6+: `get-device` + `[method]gpu-device.create-query-set`
//! WIT: `(borrow<gpu-device>, gpu-query-set-descriptor)
//!      -> result<own<gpu-query-set>, create-query-set-error>`.
//! Guest passes type=occlusion, count=1, label=none; drops own on ok; harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuDevice;

#[derive(Debug)]
struct GpuQuerySet;

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower, PartialEq, Eq)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuQueryType {
    #[component(name = "occlusion")]
    Occlusion,
    #[component(name = "timestamp")]
    Timestamp,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuQuerySetDescriptor {
    #[component(name = "type")]
    type_: GpuQueryType,
    count: u32,
    label: Option<String>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum CreateQuerySetErrorKind {
    #[component(name = "type-error")]
    TypeError,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct CreateQuerySetError {
    kind: CreateQuerySetErrorKind,
    message: String,
}

struct TestHost {
    table: ResourceTable,
}

fn register(linker: &mut Linker<TestHost>, called: Arc<AtomicBool>) -> wasmtime::Result<()> {
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
        "gpu-query-set",
        ResourceType::host::<GpuQuerySet>(),
        |mut store, rep| {
            let resource = Resource::<GpuQuerySet>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-device", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuDevice)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-device.create-query-set",
        move |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuQuerySetDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert_eq!(descriptor.type_, GpuQueryType::Occlusion);
            assert_eq!(descriptor.count, 1);
            assert!(descriptor.label.is_none());
            called.store(true, Ordering::SeqCst);
            let resource = caller.data_mut().table.push(GpuQuerySet)?;
            Ok((Ok::<_, CreateQuerySetError>(resource),))
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
fn wasi_webgpu_method_create_query_set_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_query_set.wasm"
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
fn wasi_webgpu_method_create_query_set_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_query_set.wasm"
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
