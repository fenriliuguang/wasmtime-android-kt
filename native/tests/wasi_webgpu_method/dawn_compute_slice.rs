//! Lane D: `get-device` + create-buffer + create-command-encoder + queue +
//! begin-compute-pass (none) + end + finish (none) + submit.
//! Guest drops owns; `run` returns harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{
    flags, Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuDevice {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuBuffer {
    #[allow(dead_code)]
    rep: u32,
}

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
struct GpuQueue {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuCommandBuffer {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuQuerySet;

flags! {
    GpuBufferUsage {
        #[component(name = "map-read")]
        const MAP_READ;
        #[component(name = "map-write")]
        const MAP_WRITE;
        #[component(name = "copy-src")]
        const COPY_SRC;
        #[component(name = "copy-dst")]
        const COPY_DST;
        #[component(name = "index")]
        const INDEX;
        #[component(name = "vertex")]
        const VERTEX;
        #[component(name = "uniform")]
        const UNIFORM;
        #[component(name = "storage")]
        const STORAGE;
        #[component(name = "indirect")]
        const INDIRECT;
        #[component(name = "query-resolve")]
        const QUERY_RESOLVE;
    }
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuBufferDescriptor {
    size: u64,
    usage: GpuBufferUsage,
    #[component(name = "mapped-at-creation")]
    mapped_at_creation: Option<bool>,
    label: Option<String>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuCommandEncoderDescriptor {
    label: Option<String>,
}

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

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuCommandBufferDescriptor {
    label: Option<String>,
}

struct TestHost {
    table: ResourceTable,
}

fn register_dawn_compute_slice(
    linker: &mut Linker<TestHost>,
    created_buffer: Arc<AtomicBool>,
    began: Arc<AtomicBool>,
    ended: Arc<AtomicBool>,
    submitted: Arc<AtomicBool>,
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
        "gpu-buffer",
        ResourceType::host::<GpuBuffer>(),
        |mut store, rep| {
            let resource = Resource::<GpuBuffer>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
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
        "gpu-compute-pass-encoder",
        ResourceType::host::<GpuComputePassEncoder>(),
        |mut store, rep| {
            let resource = Resource::<GpuComputePassEncoder>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-queue",
        ResourceType::host::<GpuQueue>(),
        |mut store, rep| {
            let resource = Resource::<GpuQueue>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-command-buffer",
        ResourceType::host::<GpuCommandBuffer>(),
        |mut store, rep| {
            let resource = Resource::<GpuCommandBuffer>::new_own(rep);
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
        let resource = store.data_mut().table.push(GpuDevice { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("[method]gpu-device.create-buffer", {
        let created_buffer = created_buffer.clone();
        move |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuBufferDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert_eq!(descriptor.size, 4, "guest must pass record size=4");
            assert!(
                descriptor.usage.contains(GpuBufferUsage::COPY_DST),
                "guest must pass COPY_DST"
            );
            assert!(
                descriptor.usage.contains(GpuBufferUsage::VERTEX),
                "guest must pass VERTEX"
            );
            assert!(descriptor.mapped_at_creation.is_none());
            assert!(descriptor.label.is_none());
            created_buffer.store(true, Ordering::SeqCst);
            let resource = caller.data_mut().table.push(GpuBuffer { rep: 31 })?;
            Ok((resource,))
        }
    })?;
    webgpu.func_wrap(
        "[method]gpu-device.create-command-encoder",
        |mut caller,
         (device, descriptor): (Resource<GpuDevice>, Option<GpuCommandEncoderDescriptor>)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert!(
                descriptor.is_none(),
                "guest must pass encoder descriptor none"
            );
            let resource = caller
                .data_mut()
                .table
                .push(GpuCommandEncoder { rep: 17 })?;
            Ok((resource,))
        },
    )?;
    webgpu.func_wrap(
        "[method]gpu-device.queue",
        |mut caller, (device,): (Resource<GpuDevice>,)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            let resource = caller.data_mut().table.push(GpuQueue { rep: 3 })?;
            Ok((resource,))
        },
    )?;
    webgpu.func_wrap("[method]gpu-command-encoder.begin-compute-pass", {
        let began = began.clone();
        move |mut caller,
              (encoder, descriptor): (
            Resource<GpuCommandEncoder>,
            Option<GpuComputePassDescriptor>,
        )| {
            caller.data_mut().table.get(&encoder).map(|_| ())?;
            assert!(
                descriptor.is_none(),
                "guest must pass compute-pass descriptor none"
            );
            began.store(true, Ordering::SeqCst);
            let resource = caller
                .data_mut()
                .table
                .push(GpuComputePassEncoder { rep: 79 })?;
            Ok((resource,))
        }
    })?;
    webgpu.func_wrap("[method]gpu-compute-pass-encoder.end", {
        let ended = ended.clone();
        move |mut caller, (pass,): (Resource<GpuComputePassEncoder>,)| {
            caller.data_mut().table.get(&pass).map(|_| ())?;
            ended.store(true, Ordering::SeqCst);
            Ok(())
        }
    })?;
    webgpu.func_wrap(
        "[method]gpu-command-encoder.finish",
        |mut caller,
         (encoder, descriptor): (
            Resource<GpuCommandEncoder>,
            Option<GpuCommandBufferDescriptor>,
        )| {
            caller.data_mut().table.get(&encoder).map(|_| ())?;
            assert!(
                descriptor.is_none(),
                "guest must pass finish descriptor none"
            );
            let resource = caller.data_mut().table.push(GpuCommandBuffer { rep: 19 })?;
            Ok((resource,))
        },
    )?;
    webgpu.func_wrap(
        "[method]gpu-queue.submit",
        move |mut caller,
              (queue, commands): (Resource<GpuQueue>, Vec<Resource<GpuCommandBuffer>>)| {
            caller.data_mut().table.get(&queue).map(|_| ())?;
            assert_eq!(
                commands.len(),
                1,
                "guest must pass a one-element command-buffer list"
            );
            caller.data_mut().table.get(&commands[0]).map(|_| ())?;
            submitted.store(true, Ordering::SeqCst);
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
fn wasi_webgpu_method_dawn_compute_slice_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_dawn_compute_slice.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let created_buffer = Arc::new(AtomicBool::new(false));
    let began = Arc::new(AtomicBool::new(false));
    let ended = Arc::new(AtomicBool::new(false));
    let submitted = Arc::new(AtomicBool::new(false));

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_dawn_compute_slice(
        &mut linker,
        created_buffer.clone(),
        began.clone(),
        ended.clone(),
        submitted.clone(),
    )?;

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
    assert!(
        created_buffer.load(Ordering::SeqCst),
        "create-buffer must run"
    );
    assert!(began.load(Ordering::SeqCst), "begin-compute-pass must run");
    assert!(ended.load(Ordering::SeqCst), "compute-pass end must run");
    assert!(submitted.load(Ordering::SeqCst), "submit must run");
    Ok(())
}
