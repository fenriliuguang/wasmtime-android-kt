//! WG-6: `get-device` + shader + storage buffer + BGL + pipeline-layout +
//! compute pipeline + bind-group + begin-compute-pass + set-pipeline +
//! set-bind-group + dispatch-workgroups + end + finish + submit.
//! Not empty `begin-compute-pass`. Not VectorAddScenario shader text.
//! Guest drops owns; `run` returns harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{
    flags, Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

use crate::texture_format::GpuTextureFormat;

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
struct GpuShaderModule {
    #[allow(dead_code)]
    rep: u32,
}
#[derive(Debug)]
struct GpuBindGroupLayout {
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
struct GpuBindGroup {
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
#[derive(Debug)]
struct GpuSampler;
#[derive(Debug)]
struct GpuTexture;
#[derive(Debug)]
struct GpuTextureView;
#[derive(Debug)]
struct RecordGpuPipelineConstantValue;

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

flags! {
    GpuShaderStage {
        #[component(name = "vertex")]
        const VERTEX;
        #[component(name = "fragment")]
        const FRAGMENT;
        #[component(name = "compute")]
        const COMPUTE;
    }
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuTextureViewDimension {
    #[component(name = "d1")]
    D1,
    #[component(name = "d2")]
    D2,
    #[component(name = "d2-array")]
    D2Array,
    #[component(name = "cube")]
    Cube,
    #[component(name = "cube-array")]
    CubeArray,
    #[component(name = "d3")]
    D3,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuBufferBindingType {
    #[component(name = "uniform")]
    Uniform,
    #[component(name = "storage")]
    Storage,
    #[component(name = "read-only-storage")]
    ReadOnlyStorage,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuSamplerBindingType {
    #[component(name = "filtering")]
    Filtering,
    #[component(name = "non-filtering")]
    NonFiltering,
    #[component(name = "comparison")]
    Comparison,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuTextureSampleType {
    #[component(name = "float")]
    Float,
    #[component(name = "unfilterable-float")]
    UnfilterableFloat,
    #[component(name = "depth")]
    Depth,
    #[component(name = "sint")]
    Sint,
    #[component(name = "uint")]
    Uint,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuStorageTextureAccess {
    #[component(name = "write-only")]
    WriteOnly,
    #[component(name = "read-only")]
    ReadOnly,
    #[component(name = "read-write")]
    ReadWrite,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuBufferBindingLayout {
    #[component(name = "type")]
    ty: Option<GpuBufferBindingType>,
    #[component(name = "has-dynamic-offset")]
    has_dynamic_offset: Option<bool>,
    #[component(name = "min-binding-size")]
    min_binding_size: Option<u64>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuSamplerBindingLayout {
    #[component(name = "type")]
    ty: Option<GpuSamplerBindingType>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuTextureBindingLayout {
    #[component(name = "sample-type")]
    sample_type: Option<GpuTextureSampleType>,
    #[component(name = "view-dimension")]
    view_dimension: Option<GpuTextureViewDimension>,
    multisampled: Option<bool>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuStorageTextureBindingLayout {
    access: Option<GpuStorageTextureAccess>,
    format: GpuTextureFormat,
    #[component(name = "view-dimension")]
    view_dimension: Option<GpuTextureViewDimension>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuBindGroupLayoutEntry {
    binding: u32,
    visibility: GpuShaderStage,
    buffer: Option<GpuBufferBindingLayout>,
    sampler: Option<GpuSamplerBindingLayout>,
    texture: Option<GpuTextureBindingLayout>,
    #[component(name = "storage-texture")]
    storage_texture: Option<GpuStorageTextureBindingLayout>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuBindGroupLayoutDescriptor {
    entries: Vec<GpuBindGroupLayoutEntry>,
    label: Option<String>,
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
struct GpuShaderModuleCompilationHint {
    #[component(name = "entry-point")]
    entry_point: String,
    layout: Option<GpuLayoutMode>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuShaderModuleDescriptor {
    code: String,
    #[component(name = "compilation-hints")]
    compilation_hints: Option<Vec<GpuShaderModuleCompilationHint>>,
    label: Option<String>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuPipelineLayoutDescriptor {
    #[component(name = "bind-group-layouts")]
    bind_group_layouts: Vec<Option<Resource<GpuBindGroupLayout>>>,
    #[component(name = "immediate-size")]
    immediate_size: Option<u32>,
    label: Option<String>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
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

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuBufferBinding {
    buffer: Resource<GpuBuffer>,
    offset: Option<u64>,
    size: Option<u64>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum GpuBindingResource {
    #[component(name = "gpu-buffer")]
    GpuBuffer(Resource<GpuBuffer>),
    #[component(name = "gpu-buffer-binding")]
    GpuBufferBinding(GpuBufferBinding),
    #[component(name = "gpu-sampler")]
    GpuSampler(Resource<GpuSampler>),
    #[component(name = "gpu-texture")]
    GpuTexture(Resource<GpuTexture>),
    #[component(name = "gpu-texture-view")]
    GpuTextureView(Resource<GpuTextureView>),
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuBindGroupEntry {
    binding: u32,
    resource: GpuBindingResource,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuBindGroupDescriptor {
    layout: Resource<GpuBindGroupLayout>,
    entries: Vec<GpuBindGroupEntry>,
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

struct Flags {
    shader: Arc<AtomicBool>,
    buffer: Arc<AtomicBool>,
    bgl: Arc<AtomicBool>,
    pipeline: Arc<AtomicBool>,
    bind_group: Arc<AtomicBool>,
    set_bind_group: Arc<AtomicBool>,
    dispatch: Arc<AtomicBool>,
    submitted: Arc<AtomicBool>,
}

fn push_res<T: Send + Sync + 'static>(
    webgpu: &mut wasmtime::component::LinkerInstance<'_, TestHost>,
    name: &str,
) -> wasmtime::Result<()> {
    webgpu.resource(name, ResourceType::host::<T>(), |mut store, rep| {
        let resource = Resource::<T>::new_own(rep);
        store.data_mut().table.delete(resource)?;
        Ok(())
    })?;
    Ok(())
}

fn clone_flags(flags: &Flags) -> Flags {
    Flags {
        shader: flags.shader.clone(),
        buffer: flags.buffer.clone(),
        bgl: flags.bgl.clone(),
        pipeline: flags.pipeline.clone(),
        bind_group: flags.bind_group.clone(),
        set_bind_group: flags.set_bind_group.clone(),
        dispatch: flags.dispatch.clone(),
        submitted: flags.submitted.clone(),
    }
}

fn register_dawn_guest_compute(
    linker: &mut Linker<TestHost>,
    flags: Flags,
) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    push_res::<GpuDevice>(&mut webgpu, "gpu-device")?;
    push_res::<GpuBuffer>(&mut webgpu, "gpu-buffer")?;
    push_res::<GpuShaderModule>(&mut webgpu, "gpu-shader-module")?;
    push_res::<GpuBindGroupLayout>(&mut webgpu, "gpu-bind-group-layout")?;
    push_res::<GpuPipelineLayout>(&mut webgpu, "gpu-pipeline-layout")?;
    push_res::<GpuComputePipeline>(&mut webgpu, "gpu-compute-pipeline")?;
    push_res::<GpuBindGroup>(&mut webgpu, "gpu-bind-group")?;
    push_res::<GpuCommandEncoder>(&mut webgpu, "gpu-command-encoder")?;
    push_res::<GpuComputePassEncoder>(&mut webgpu, "gpu-compute-pass-encoder")?;
    push_res::<GpuQueue>(&mut webgpu, "gpu-queue")?;
    push_res::<GpuCommandBuffer>(&mut webgpu, "gpu-command-buffer")?;
    push_res::<GpuQuerySet>(&mut webgpu, "gpu-query-set")?;
    push_res::<GpuSampler>(&mut webgpu, "gpu-sampler")?;
    push_res::<GpuTexture>(&mut webgpu, "gpu-texture")?;
    push_res::<GpuTextureView>(&mut webgpu, "gpu-texture-view")?;
    push_res::<RecordGpuPipelineConstantValue>(&mut webgpu, "record-gpu-pipeline-constant-value")?;
    webgpu.func_wrap("get-device", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuDevice { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("[method]gpu-device.create-shader-module", {
        let shader = flags.shader.clone();
        move |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuShaderModuleDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert!(
                descriptor.code.contains("read_write"),
                "guest must pass a storage read_write shader"
            );
            assert!(
                !descriptor.code.contains("inputA"),
                "guest must not use VectorAddScenario.SHADER"
            );
            assert!(descriptor.compilation_hints.is_none());
            assert_eq!(descriptor.label.as_deref(), Some("wg6"));
            shader.store(true, Ordering::SeqCst);
            let resource = caller.data_mut().table.push(GpuShaderModule { rep: 11 })?;
            Ok((resource,))
        }
    })?;
    webgpu.func_wrap("[method]gpu-device.create-buffer", {
        let buffer = flags.buffer.clone();
        move |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuBufferDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert_eq!(descriptor.size, 4);
            assert!(
                descriptor.usage.contains(GpuBufferUsage::STORAGE),
                "guest must pass STORAGE"
            );
            buffer.store(true, Ordering::SeqCst);
            let resource = caller.data_mut().table.push(GpuBuffer { rep: 31 })?;
            Ok((resource,))
        }
    })?;
    webgpu.func_wrap("[method]gpu-device.create-bind-group-layout", {
        let bgl = flags.bgl.clone();
        move |mut caller,
              (device, descriptor): (Resource<GpuDevice>, GpuBindGroupLayoutDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert_eq!(descriptor.entries.len(), 1, "guest must pass one BGL entry");
            assert_eq!(descriptor.entries[0].binding, 0);
            assert!(descriptor.entries[0]
                .visibility
                .contains(GpuShaderStage::COMPUTE));
            assert!(matches!(
                descriptor.entries[0].buffer.as_ref().and_then(|b| b.ty),
                Some(GpuBufferBindingType::Storage)
            ));
            bgl.store(true, Ordering::SeqCst);
            let resource = caller
                .data_mut()
                .table
                .push(GpuBindGroupLayout { rep: 59 })?;
            Ok((resource,))
        }
    })?;
    webgpu.func_wrap(
        "[method]gpu-device.create-pipeline-layout",
        |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuPipelineLayoutDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert_eq!(descriptor.bind_group_layouts.len(), 1);
            let layout = descriptor.bind_group_layouts[0]
                .as_ref()
                .expect("guest must pass some(bgl)");
            caller.data_mut().table.get(layout).map(|_| ())?;
            let resource = caller
                .data_mut()
                .table
                .push(GpuPipelineLayout { rep: 61 })?;
            Ok((resource,))
        },
    )?;
    webgpu.func_wrap("[method]gpu-device.create-compute-pipeline", {
        let pipeline = flags.pipeline.clone();
        move |mut caller,
              (device, descriptor): (Resource<GpuDevice>, GpuComputePipelineDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            caller
                .data_mut()
                .table
                .get(&descriptor.compute.module)
                .map(|_| ())?;
            assert_eq!(descriptor.compute.entry_point.as_deref(), Some("main"));
            assert!(matches!(descriptor.layout, GpuLayoutMode::Specific(_)));
            pipeline.store(true, Ordering::SeqCst);
            let resource = caller
                .data_mut()
                .table
                .push(GpuComputePipeline { rep: 73 })?;
            Ok((resource,))
        }
    })?;
    webgpu.func_wrap("[method]gpu-device.create-bind-group", {
        let bind_group = flags.bind_group.clone();
        move |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuBindGroupDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            caller
                .data_mut()
                .table
                .get(&descriptor.layout)
                .map(|_| ())?;
            assert_eq!(descriptor.entries.len(), 1);
            assert_eq!(descriptor.entries[0].binding, 0);
            assert!(matches!(
                descriptor.entries[0].resource,
                GpuBindingResource::GpuBuffer(_)
            ));
            bind_group.store(true, Ordering::SeqCst);
            let resource = caller.data_mut().table.push(GpuBindGroup { rep: 77 })?;
            Ok((resource,))
        }
    })?;
    webgpu.func_wrap(
        "[method]gpu-device.create-command-encoder",
        |mut caller,
         (device, descriptor): (Resource<GpuDevice>, Option<GpuCommandEncoderDescriptor>)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert!(descriptor.is_none());
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
    webgpu.func_wrap(
        "[method]gpu-command-encoder.begin-compute-pass",
        |mut caller,
         (encoder, descriptor): (
            Resource<GpuCommandEncoder>,
            Option<GpuComputePassDescriptor>,
        )| {
            caller.data_mut().table.get(&encoder).map(|_| ())?;
            assert!(descriptor.is_none());
            let resource = caller
                .data_mut()
                .table
                .push(GpuComputePassEncoder { rep: 79 })?;
            Ok((resource,))
        },
    )?;
    webgpu.func_wrap(
        "[method]gpu-compute-pass-encoder.set-pipeline",
        |mut caller,
         (pass, pipeline): (
            Resource<GpuComputePassEncoder>,
            Resource<GpuComputePipeline>,
        )| {
            caller.data_mut().table.get(&pass).map(|_| ())?;
            caller.data_mut().table.get(&pipeline).map(|_| ())?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("[method]gpu-compute-pass-encoder.set-bind-group", {
        let set_bind_group = flags.set_bind_group.clone();
        move |mut caller,
              (pass, index, bind_group, offsets, start, length): (
            Resource<GpuComputePassEncoder>,
            u32,
            Option<Resource<GpuBindGroup>>,
            Option<Vec<u32>>,
            Option<u64>,
            Option<u32>,
        )| {
            caller.data_mut().table.get(&pass).map(|_| ())?;
            assert_eq!(index, 0);
            let bind_group = bind_group.expect("guest must pass bind-group=some");
            caller.data_mut().table.get(&bind_group).map(|_| ())?;
            assert!(offsets.is_none());
            assert!(start.is_none());
            assert!(length.is_none());
            set_bind_group.store(true, Ordering::SeqCst);
            Ok((Ok::<(), SetBindGroupError>(()),))
        }
    })?;
    webgpu.func_wrap("[method]gpu-compute-pass-encoder.dispatch-workgroups", {
        let dispatch = flags.dispatch.clone();
        move |mut caller,
              (pass, x, y, z): (
            Resource<GpuComputePassEncoder>,
            u32,
            Option<u32>,
            Option<u32>,
        )| {
            caller.data_mut().table.get(&pass).map(|_| ())?;
            assert_eq!(x, 1);
            assert_eq!(y, Some(1));
            assert_eq!(z, Some(1));
            dispatch.store(true, Ordering::SeqCst);
            Ok(())
        }
    })?;
    webgpu.func_wrap(
        "[method]gpu-compute-pass-encoder.end",
        |mut caller, (pass,): (Resource<GpuComputePassEncoder>,)| {
            caller.data_mut().table.get(&pass).map(|_| ())?;
            Ok(())
        },
    )?;
    webgpu.func_wrap(
        "[method]gpu-command-encoder.finish",
        |mut caller,
         (encoder, descriptor): (
            Resource<GpuCommandEncoder>,
            Option<GpuCommandBufferDescriptor>,
        )| {
            caller.data_mut().table.get(&encoder).map(|_| ())?;
            assert!(descriptor.is_none());
            let resource = caller.data_mut().table.push(GpuCommandBuffer { rep: 19 })?;
            Ok((resource,))
        },
    )?;
    webgpu.func_wrap("[method]gpu-queue.submit", {
        let submitted = flags.submitted.clone();
        move |mut caller,
              (queue, commands): (Resource<GpuQueue>, Vec<Resource<GpuCommandBuffer>>)| {
            caller.data_mut().table.get(&queue).map(|_| ())?;
            assert_eq!(commands.len(), 1);
            caller.data_mut().table.get(&commands[0]).map(|_| ())?;
            submitted.store(true, Ordering::SeqCst);
            Ok(())
        }
    })?;
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

fn flags() -> Flags {
    Flags {
        shader: Arc::new(AtomicBool::new(false)),
        buffer: Arc::new(AtomicBool::new(false)),
        bgl: Arc::new(AtomicBool::new(false)),
        pipeline: Arc::new(AtomicBool::new(false)),
        bind_group: Arc::new(AtomicBool::new(false)),
        set_bind_group: Arc::new(AtomicBool::new(false)),
        dispatch: Arc::new(AtomicBool::new(false)),
        submitted: Arc::new(AtomicBool::new(false)),
    }
}

fn assert_chain(flags: &Flags) {
    assert!(flags.shader.load(Ordering::SeqCst), "create-shader-module");
    assert!(flags.buffer.load(Ordering::SeqCst), "create-buffer");
    assert!(flags.bgl.load(Ordering::SeqCst), "create-bind-group-layout");
    assert!(
        flags.pipeline.load(Ordering::SeqCst),
        "create-compute-pipeline"
    );
    assert!(flags.bind_group.load(Ordering::SeqCst), "create-bind-group");
    assert!(
        flags.set_bind_group.load(Ordering::SeqCst),
        "set-bind-group"
    );
    assert!(flags.dispatch.load(Ordering::SeqCst), "dispatch-workgroups");
    assert!(flags.submitted.load(Ordering::SeqCst), "submit");
}

#[test]
fn wasi_webgpu_method_dawn_guest_compute_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_dawn_guest_compute.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let flags = flags();
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_dawn_guest_compute(&mut linker, clone_flags(&flags))?;
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
    assert_chain(&flags);
    Ok(())
}

#[test]
fn wasi_webgpu_method_dawn_guest_compute_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_dawn_guest_compute.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let flags = flags();
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_dawn_guest_compute(&mut linker, clone_flags(&flags))?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 1, "guest run must return harness 1 via call_async");
    assert_chain(&flags);
    Ok(())
}
