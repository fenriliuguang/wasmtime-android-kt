//! Component Model JNI (M1 sync + M2 concurrent/async + P3 stream).

use crate::engine::new_engine;
use crate::error::{throw, throw_compile, throw_err, throw_link};
use crate::handles::{drop_handle, from_handle, to_handle};
use crate::host::{
    Gpu, GpuAdapter, GpuBindGroup, GpuBindGroupLayout, GpuBuffer, GpuCommandBuffer,
    GpuCommandEncoder, GpuComputePassEncoder, GpuComputePipeline, GpuDevice, GpuPipelineLayout,
    GpuQuerySet, GpuQueue, GpuRenderBundle, GpuRenderBundleEncoder, GpuRenderPassEncoder,
    GpuRenderPipeline, GpuSampler, GpuShaderModule, GpuTexture, GpuTextureView, HostState, Widget,
};
use crate::jvm;
use crate::webgpu_abi::{
    CreatePipelineError, CreatePipelineErrorKind, CreateQuerySetError, GetMappedRangeError,
    GpuAdapterInfo, GpuBindGroupDescriptor, GpuBindGroupLayoutDescriptor, GpuBindingResource,
    GpuBufferBindingType, GpuSamplerBindingType, GpuTextureSampleType,
    GpuBufferDescriptor, GpuBufferMapState, GpuBufferUsage, GpuCanvasConfiguration,
    GpuCanvasConfigurationOwned, GpuCanvasContext, GpuColor, GpuCommandBufferDescriptor,
    GpuCommandEncoderDescriptor, GpuCompilationInfo, GpuCompilationMessage,
    GpuCompilationMessageType, GpuComputePassDescriptor, GpuComputePipelineDescriptor,
    GpuDeviceDescriptor, GpuDeviceLostInfo, GpuDeviceLostReason, GpuError, GpuErrorFilter,
    GpuErrorKind, GpuExtent3D, GpuIndexFormat, GpuLayoutMode, GpuMapMode,
    GpuBlendFactor, GpuBlendOperation, GpuCompareFunction, GpuCullMode, GpuFrontFace,
    GpuMipmapFilterMode, GpuPipelineErrorReason, GpuPipelineLayoutDescriptor,
    GpuPrimitiveTopology,
    GpuQuerySetDescriptor, GpuQueryType, GpuRenderBundleDescriptor,
    GpuRenderBundleEncoderDescriptor, GpuRenderPassDescriptor, GpuRenderPipelineDescriptor,
    GpuRequestAdapterOptions, GpuSamplerDescriptor,
    GpuShaderModuleDescriptor, GpuShaderStage, GpuSupportedFeatures,
    GpuSupportedLimits, GpuTexelCopyBufferInfo, GpuTexelCopyBufferLayout, GpuTexelCopyTextureInfo,
    GpuTextureDescriptor,
    GpuTextureDimension, GpuTextureFormat, GpuTextureUsage, GpuTextureViewDescriptor,
    GpuTextureViewDimension, GpuUncapturedErrorEvent, GpuVertexFormat, GpuVertexStepMode, MapAsyncError, PopErrorScopeError,
    RecordGpuPipelineConstantValue, RecordOptionGpuSize64, RequestDeviceError,
    RequestDeviceErrorKind, SetBindGroupError, UnmapError, WgslLanguageFeatures, WriteBufferError,
};
use futures::channel::oneshot;
use jni::objects::{JByteArray, JClass, JObject, JString};
use jni::sys::{jint, jlong};
use jni::JNIEnv;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use wasmtime::component::{
    Component, FutureReader, Linker, Resource, ResourceType, Source, StreamConsumer, StreamReader,
    StreamResult,
};
use wasmtime::{Engine, Store, StoreContextMut};

type HostStore = Store<HostState>;

/// P3: pack guest `vertex.buffers` into parallel JNI int arrays (Dawn step/format values).
fn pack_vertex_buffers(
    buffers: &Option<Vec<Option<crate::webgpu_abi::GpuVertexBufferLayout>>>,
) -> (Vec<i32>, Vec<i32>, Vec<i32>, Vec<i32>, Vec<i32>, Vec<i32>) {
    let mut strides = Vec::new();
    let mut step_modes = Vec::new();
    let mut attr_index = Vec::new();
    let mut attr_formats = Vec::new();
    let mut attr_offsets = Vec::new();
    let mut attr_locations = Vec::new();
    let Some(slots) = buffers else {
        return (
            strides,
            step_modes,
            attr_index,
            attr_formats,
            attr_offsets,
            attr_locations,
        );
    };
    for slot in slots {
        let Some(layout) = slot else {
            continue;
        };
        let buf_i = strides.len() as i32;
        strides.push(layout.array_stride as i32);
        step_modes.push(match layout.step_mode {
            Some(GpuVertexStepMode::Instance) => 2,
            Some(GpuVertexStepMode::Vertex) | None => 1,
        });
        for attr in &layout.attributes {
            attr_index.push(buf_i);
            attr_formats.push(match attr.format {
                GpuVertexFormat::Float32x2 => 0x1d,
                GpuVertexFormat::Float32x3 => 0x1e,
                GpuVertexFormat::Float32x4 => 0x1f,
                GpuVertexFormat::Float32 => 0x1c,
                other => other as i32,
            });
            attr_offsets.push(attr.offset as i32);
            attr_locations.push(attr.shader_location as i32);
        }
    }
    (
        strides,
        step_modes,
        attr_index,
        attr_formats,
        attr_offsets,
        attr_locations,
    )
}

fn first_fragment_target_format(fragment: &Option<crate::webgpu_abi::GpuFragmentState>) -> i32 {
    fragment
        .as_ref()
        .and_then(|fs| fs.targets.iter().flatten().next())
        .map(|target| target.format.to_dawn_u32() as i32)
        .unwrap_or(0)
}

fn dawn_topology(t: GpuPrimitiveTopology) -> i32 {
    match t {
        GpuPrimitiveTopology::PointList => 1,
        GpuPrimitiveTopology::LineList => 2,
        GpuPrimitiveTopology::LineStrip => 3,
        GpuPrimitiveTopology::TriangleList => 4,
        GpuPrimitiveTopology::TriangleStrip => 5,
    }
}

fn dawn_cull(c: GpuCullMode) -> i32 {
    match c {
        GpuCullMode::None => 1,
        GpuCullMode::Front => 2,
        GpuCullMode::Back => 3,
    }
}

fn dawn_front_face(f: GpuFrontFace) -> i32 {
    match f {
        GpuFrontFace::Ccw => 1,
        GpuFrontFace::Cw => 2,
    }
}

fn dawn_index_format(f: GpuIndexFormat) -> i32 {
    match f {
        GpuIndexFormat::Uint16 => 1,
        GpuIndexFormat::Uint32 => 2,
    }
}

fn dawn_blend_op(op: GpuBlendOperation) -> i32 {
    match op {
        GpuBlendOperation::Add => 1,
        GpuBlendOperation::Subtract => 2,
        GpuBlendOperation::ReverseSubtract => 3,
        GpuBlendOperation::Min => 4,
        GpuBlendOperation::Max => 5,
    }
}

fn dawn_blend_factor(f: GpuBlendFactor) -> i32 {
    (f as i32) + 1
}

/// F1: primitive (topology/strip/front/cull) + multisample + per-target blend 7-tuples.
fn pack_render_pipeline_semantics(
    descriptor: &GpuRenderPipelineDescriptor,
) -> (Vec<i32>, Vec<i32>, Vec<i32>) {
    let mut primitive = vec![0, 0, 0, 0];
    if let Some(p) = &descriptor.primitive {
        primitive[0] = p.topology.map(dawn_topology).unwrap_or(0);
        primitive[1] = p.strip_index_format.map(dawn_index_format).unwrap_or(0);
        primitive[2] = p.front_face.map(dawn_front_face).unwrap_or(0);
        primitive[3] = p.cull_mode.map(dawn_cull).unwrap_or(0);
    }
    let mut multisample = Vec::new();
    if let Some(ms) = &descriptor.multisample {
        let count = ms.count.unwrap_or(0) as i32;
        let has_mask = if ms.mask.is_some() { 1 } else { 0 };
        let mask = ms.mask.unwrap_or(0) as i32;
        let alpha = match ms.alpha_to_coverage_enabled {
            Some(true) => 1,
            Some(false) => 0,
            None => -1,
        };
        multisample.extend_from_slice(&[count, has_mask, mask, alpha]);
    }
    let mut blend = Vec::new();
    if let Some(fragment) = &descriptor.fragment {
        for target in fragment.targets.iter().flatten() {
            match &target.blend {
                None => blend.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0]),
                Some(b) => {
                    blend.push(1);
                    blend.push(b.color.operation.map(dawn_blend_op).unwrap_or(0));
                    blend.push(b.color.src_factor.map(dawn_blend_factor).unwrap_or(0));
                    blend.push(b.color.dst_factor.map(dawn_blend_factor).unwrap_or(0));
                    blend.push(b.alpha.operation.map(dawn_blend_op).unwrap_or(0));
                    blend.push(b.alpha.src_factor.map(dawn_blend_factor).unwrap_or(0));
                    blend.push(b.alpha.dst_factor.map(dawn_blend_factor).unwrap_or(0));
                }
            }
        }
    }
    (primitive, multisample, blend)
}

fn pack_color_clear_bits(c: &GpuColor) -> [i32; 4] {
    [
        (c.r as f32).to_bits() as i32,
        (c.g as f32).to_bits() as i32,
        (c.b as f32).to_bits() as i32,
        (c.a as f32).to_bits() as i32,
    ]
}

/// Guest `option<record-gpu-pipeline-constant-value>` → host handle (0 = none).
fn pipeline_constant_rep(
    rec: &Option<Resource<RecordGpuPipelineConstantValue>>,
) -> i32 {
    rec.as_ref().map(|r| r.rep() as i32).unwrap_or(0)
}

/// P3-PRIM-5: collect guest `stream.write` bytes; complete oneshot on drop.
struct CollectConsumer {
    buf: Arc<Mutex<Vec<u8>>>,
    done: Option<oneshot::Sender<u32>>,
}

impl Drop for CollectConsumer {
    fn drop(&mut self) {
        if let Some(tx) = self.done.take() {
            let n = self.buf.lock().map(|b| b.len() as u32).unwrap_or(0);
            let _ = tx.send(n);
        }
    }
}

impl StreamConsumer<HostState> for CollectConsumer {
    type Item = u8;

    fn poll_consume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        store: StoreContextMut<HostState>,
        src: Source<'_, Self::Item>,
        finish: bool,
    ) -> Poll<wasmtime::Result<StreamResult>> {
        let this = self.get_mut();
        let mut src = src.as_direct(store);
        let chunk = src.remaining();
        if chunk.is_empty() {
            if finish {
                return Poll::Ready(Ok(StreamResult::Cancelled));
            }
            // Zero-length readiness probe (component-model#561). Completed-on-empty
            // traps. Do not wake_by_ref: that marks the task runnable while guest
            // stream.write is still on the stack, so the executor re-polls until
            // ART's ~1MiB instrument thread overflows (Vivo SIGSEGV).
            // Wasmtime keeps the waker and polls again when the guest writes.
            let _ = cx;
            return Poll::Pending;
        }
        let n = chunk.len();
        this.buf.lock().unwrap().extend_from_slice(chunk);
        src.mark_read(n);
        Poll::Ready(Ok(StreamResult::Completed))
    }
}


fn l2_supported_limits_handles(
    caller: &mut StoreContextMut<'_, HostState>,
    limits: &Resource<GpuSupportedLimits>,
) -> wasmtime::Result<(jni::objects::GlobalRef, u32, u32)> {
    let (adapter, device) = {
        let entry = caller.data_mut().table.get(limits)?;
        (entry.adapter, entry.device)
    };
    let cb = caller
        .data()
        .experimental_host_cb
        .as_ref()
        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
        .cloned()?;
    let l2_adapter = if adapter == 0 && device == 0 {
        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
    } else {
        adapter
    };
    Ok((cb, l2_adapter, device))
}

fn define_host(linker: &mut Linker<HostState>) -> Result<(), String> {
    linker
        .root()
        .resource(
            "widget",
            ResourceType::host::<Widget>(),
            |mut store, rep| {
                let resource = Resource::<Widget>::new_own(rep);
                store.data_mut().table.delete(resource)?;
                Ok(())
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .root()
        .func_wrap("make-widget", |mut store, (rep,): (u32,)| {
            let resource = store.data_mut().table.push(Widget { rep })?;
            Ok((resource,))
        })
        .map_err(|e| e.to_string())?;

    linker
        .root()
        .func_wrap("echo-widget", |mut store, (r,): (Resource<Widget>,)| {
            let w = store.data_mut().table.get(&r)?;
            Ok((w.rep,))
        })
        .map_err(|e| e.to_string())?;

    linker
        .root()
        .func_wrap("add", |caller, (a, b): (u32, u32)| {
            let cb = caller
                .data()
                .add_cb
                .as_ref()
                .ok_or_else(|| wasmtime::Error::msg("host add callback not set"))?
                .clone();
            let result = jvm::call_u32_u32_to_u32(&cb, a, b).map_err(wasmtime::Error::msg)?;
            Ok((result,))
        })
        .map_err(|e| e.to_string())?;

    // M2: true CM async host import via official concurrent API + FutureReader complete.
    linker
        .root()
        .func_wrap_concurrent("get", |accessor, ()| {
            Box::pin(async move {
                let (tx, rx) = oneshot::channel::<u32>();
                let mut reader = accessor.with(|mut access| {
                    FutureReader::new(&mut access, async move {
                        match rx.await {
                            Ok(v) => Ok(Some(v)),
                            Err(_) => Err(wasmtime::Error::msg("future rejected/canceled")),
                        }
                    })
                })?;
                // Complete then close so the producer is observed (not left pending).
                tx.send(42)
                    .map_err(|_| wasmtime::Error::msg("no future consumer"))?;
                accessor.with(|mut access| reader.close(&mut access))?;
                Ok((42u32,))
            })
        })
        .map_err(|e| e.to_string())?;

    // WASI 0.3: wasi:random/random@0.3.0 (get-random-u64 + get-random-bytes).
    {
        let mut random = linker
            .instance("wasi:random/random@0.3.0")
            .map_err(|e| e.to_string())?;
        random
            .func_wrap("get-random-u64", |_store, ()| {
                let mut bytes = [0u8; 8];
                getrandom::fill(&mut bytes).map_err(|e| wasmtime::Error::msg(e.to_string()))?;
                Ok((u64::from_ne_bytes(bytes),))
            })
            .map_err(|e| e.to_string())?;
        random
            .func_wrap("get-random-bytes", |_store, (len,): (u64,)| {
                let n = (len as usize).min(4096);
                let mut bytes = vec![0u8; n];
                if n > 0 {
                    getrandom::fill(&mut bytes).map_err(|e| wasmtime::Error::msg(e.to_string()))?;
                }
                Ok((bytes,))
            })
            .map_err(|e| e.to_string())?;
    }

    // WASI 0.3: wasi:clocks/monotonic-clock@0.3.0 (now + resolution + wait-for + wait-until).
    {
        use std::sync::OnceLock;
        use std::time::Instant;
        // Shared Instant epoch for now / wait-until (same process-wide mark).
        static MONOTONIC_START: OnceLock<Instant> = OnceLock::new();

        let mut clocks = linker
            .instance("wasi:clocks/monotonic-clock@0.3.0")
            .map_err(|e| e.to_string())?;
        clocks
            .func_wrap("now", |_store, ()| {
                let start = MONOTONIC_START.get_or_init(Instant::now);
                Ok((start.elapsed().as_nanos() as u64,))
            })
            .map_err(|e| e.to_string())?;
        clocks
            .func_wrap("resolution", |_store, ()| {
                // Instant is nanosecond-granularity on this host.
                Ok((1u64,))
            })
            .map_err(|e| e.to_string())?;
        // True CM async: yield on oneshot while a helper thread sleeps (no tokio).
        clocks
            .func_wrap_concurrent("wait-for", |_accessor, (ns,): (u64,)| {
                Box::pin(async move {
                    let capped = ns.min(1_000_000_000); // 1s host cap
                    let (tx, rx) = oneshot::channel::<()>();
                    std::thread::spawn(move || {
                        if capped > 0 {
                            std::thread::sleep(std::time::Duration::from_nanos(capped));
                        }
                        let _ = tx.send(());
                    });
                    let _ = rx.await;
                    Ok(())
                })
            })
            .map_err(|e| e.to_string())?;
        clocks
            .func_wrap_concurrent("wait-until", |_accessor, (when,): (u64,)| {
                Box::pin(async move {
                    let start = MONOTONIC_START.get_or_init(Instant::now);
                    let now = start.elapsed().as_nanos() as u64;
                    let sleep_ns = when.saturating_sub(now).min(1_000_000_000); // 1s host cap
                    let (tx, rx) = oneshot::channel::<()>();
                    std::thread::spawn(move || {
                        if sleep_ns > 0 {
                            std::thread::sleep(std::time::Duration::from_nanos(sleep_ns));
                        }
                        let _ = tx.send(());
                    });
                    let _ = rx.await;
                    Ok(())
                })
            })
            .map_err(|e| e.to_string())?;
    }

    // WASI 0.3: wasi:clocks/system-clock@0.3.0 (now + resolution).
    // now: transitional u64 unix seconds (official WIT is instant record; deferred).
    // resolution: transitional u64 ns (official WIT may be datetime record).
    {
        let mut clock = linker
            .instance("wasi:clocks/system-clock@0.3.0")
            .map_err(|e| e.to_string())?;
        clock
            .func_wrap("now", |_store, ()| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let secs = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                Ok((secs,))
            })
            .map_err(|e| e.to_string())?;
        clock
            .func_wrap("resolution", |_store, ()| {
                // Transitional u64 ns (official WIT may be datetime record).
                Ok((1u64,))
            })
            .map_err(|e| e.to_string())?;
    }

    // Pipe guest stream<u8> into CollectConsumer; complete future with byte count.
    // Shared by root `take` (P3 fixture) and wasi:cli stdout/stderr write-via-stream.
    fn pipe_stream_byte_count(
        store: &mut StoreContextMut<HostState>,
        reader: StreamReader<u8>,
    ) -> wasmtime::Result<FutureReader<u32>> {
        let (tx, rx) = oneshot::channel::<u32>();
        let buf = Arc::new(Mutex::new(Vec::new()));
        reader.pipe(
            &mut *store,
            CollectConsumer {
                buf: buf.clone(),
                done: Some(tx),
            },
        )?;
        let fut = FutureReader::new(store, async move {
            let n = match rx.await {
                Ok(n) => n,
                Err(_) => 0,
            };
            Ok::<_, wasmtime::Error>(n)
        })?;
        let _ = buf;
        Ok(fut)
    }

    // P3-PRIM-5: host consumes guest stream; returns future<u32> byte count.
    linker
        .root()
        .func_wrap(
            "take",
            |mut store: StoreContextMut<HostState>, (reader,): (StreamReader<u8>,)| {
                let fut = pipe_stream_byte_count(&mut store, reader)?;
                Ok((fut,))
            },
        )
        .map_err(|e| e.to_string())?;

    // WASI 0.3: wasi:cli/stdout@0.3.0 — transitional write-via-stream → future<u32>.
    // Official WIT: future<result<_, error-code>>; enum result deferred for hand-written WAT.
    linker
        .instance("wasi:cli/stdout@0.3.0")
        .map_err(|e| e.to_string())?
        .func_wrap(
            "write-via-stream",
            |mut store: StoreContextMut<HostState>, (reader,): (StreamReader<u8>,)| {
                let fut = pipe_stream_byte_count(&mut store, reader)?;
                Ok((fut,))
            },
        )
        .map_err(|e| e.to_string())?;

    // WASI 0.3: wasi:cli/stderr@0.3.0 — same transitional write-via-stream → future<u32>.
    linker
        .instance("wasi:cli/stderr@0.3.0")
        .map_err(|e| e.to_string())?
        .func_wrap(
            "write-via-stream",
            |mut store: StoreContextMut<HostState>, (reader,): (StreamReader<u8>,)| {
                let fut = pipe_stream_byte_count(&mut store, reader)?;
                Ok((fut,))
            },
        )
        .map_err(|e| e.to_string())?;

    // WASI 0.3: wasi:cli/stdin@0.3.0 — transitional read-via-stream → stream<u8>.
    // Official WIT: tuple<stream<u8>, future<result<_, error-code>>>; tuple/result deferred.
    linker
        .instance("wasi:cli/stdin@0.3.0")
        .map_err(|e| e.to_string())?
        .func_wrap(
            "read-via-stream",
            |mut store: StoreContextMut<HostState>, ()| {
                let reader = StreamReader::new(&mut store, b"IN\n".to_vec())?;
                Ok((reader,))
            },
        )
        .map_err(|e| e.to_string())?;

    // M3/M4: Track A experimental CM host (flat u32 reps) → L2 via Kotlin callbacks.
    // Scope ends before W1 wasi:webgpu dual-register (Linker::instance is once-per-name).
    {
        let mut exp = linker
            .instance("experimental:webgpu-cm/host@0.8.0")
            .map_err(|e| e.to_string())?;

        fn exp_cb(data: &HostState) -> Result<jni::objects::GlobalRef, wasmtime::Error> {
            data.experimental_host_cb
                .as_ref()
                .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                .cloned()
        }

        exp.func_wrap("request-adapter", |caller, ()| {
            let cb = exp_cb(caller.data())?;
            let rep = jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
            Ok((rep,))
        })
        .map_err(|e| e.to_string())?;

        exp.func_wrap("adapter-request-device", |caller, (adapter,): (u32,)| {
            let cb = exp_cb(caller.data())?;
            let rep =
                jvm::exp_adapter_request_device(&cb, adapter).map_err(wasmtime::Error::msg)?;
            Ok((rep,))
        })
        .map_err(|e| e.to_string())?;

        exp.func_wrap("device-get-queue", |caller, (device,): (u32,)| {
            let cb = exp_cb(caller.data())?;
            let rep = jvm::exp_device_get_queue(&cb, device).map_err(wasmtime::Error::msg)?;
            Ok((rep,))
        })
        .map_err(|e| e.to_string())?;

        exp.func_wrap(
            "create-surface-from-native-window",
            |caller, (window,): (u64,)| {
                let cb = exp_cb(caller.data())?;
                let rep = jvm::exp_create_surface(&cb, window).map_err(wasmtime::Error::msg)?;
                Ok((rep,))
            },
        )
        .map_err(|e| e.to_string())?;

        exp.func_wrap(
            "surface-configure",
            |caller, (surface, device, adapter, width, height): (u32, u32, u32, u32, u32)| {
                let cb = exp_cb(caller.data())?;
                let format =
                    jvm::exp_surface_configure(&cb, surface, device, adapter, width, height)
                        .map_err(wasmtime::Error::msg)?;
                Ok((format,))
            },
        )
        .map_err(|e| e.to_string())?;

        exp.func_wrap(
            "surface-get-current-texture-view",
            |caller, (surface,): (u32,)| {
                let cb = exp_cb(caller.data())?;
                let rep = jvm::exp_surface_get_view(&cb, surface).map_err(wasmtime::Error::msg)?;
                Ok((rep,))
            },
        )
        .map_err(|e| e.to_string())?;

        exp.func_wrap(
            "device-create-command-encoder",
            |caller, (device,): (u32,)| {
                let cb = exp_cb(caller.data())?;
                let rep =
                    jvm::exp_create_command_encoder(&cb, device).map_err(wasmtime::Error::msg)?;
                Ok((rep,))
            },
        )
        .map_err(|e| e.to_string())?;

        exp.func_wrap(
            "command-encoder-begin-render-pass-clear",
            |caller, (encoder, view): (u32, u32)| {
                let cb = exp_cb(caller.data())?;
                let rep = jvm::exp_begin_render_pass_clear(&cb, encoder, view)
                    .map_err(wasmtime::Error::msg)?;
                Ok((rep,))
            },
        )
        .map_err(|e| e.to_string())?;

        exp.func_wrap("render-pass-end", |caller, (pass,): (u32,)| {
            let cb = exp_cb(caller.data())?;
            jvm::exp_render_pass_end(&cb, pass).map_err(wasmtime::Error::msg)?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;

        exp.func_wrap("command-encoder-finish", |caller, (encoder,): (u32,)| {
            let cb = exp_cb(caller.data())?;
            let rep =
                jvm::exp_command_encoder_finish(&cb, encoder).map_err(wasmtime::Error::msg)?;
            Ok((rep,))
        })
        .map_err(|e| e.to_string())?;

        exp.func_wrap("queue-submit1", |caller, (queue, commands): (u32, u32)| {
            let cb = exp_cb(caller.data())?;
            jvm::exp_queue_submit1(&cb, queue, commands).map_err(wasmtime::Error::msg)?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;

        exp.func_wrap("surface-present", |caller, (surface,): (u32,)| {
            let cb = exp_cb(caller.data())?;
            jvm::exp_surface_present(&cb, surface).map_err(wasmtime::Error::msg)?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;

        exp.func_wrap("surface-unconfigure", |caller, (surface,): (u32,)| {
            let cb = exp_cb(caller.data())?;
            jvm::exp_surface_unconfigure(&cb, surface).map_err(wasmtime::Error::msg)?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    }

    // W2/W3: proposal instance transitional flat `request-adapter` /
    // `adapter-request-device` as true CM async (`func_wrap_concurrent` + oneshot
    // yield); W3 `device-get-queue`, `device-create-command-encoder`,
    // `command-encoder-finish`, `queue-submit1`,
    // `command-encoder-begin-render-pass-clear`, and `render-pass-end` are sync
    // `func_wrap` (same L2 as experimental). W3 also registers WIT `gpu` +
    // `get-gpu` + `[method]gpu.request-adapter` (S2: async
    // option<own<gpu-adapter>> + option<gpu-request-adapter-options>), `gpu-adapter`
    // + `get-adapter`
    // + `[method]gpu-adapter.request-device` (S3: async
    // result<own<gpu-device>, request-device-error> + option<gpu-device-descriptor>),
    // and `gpu-device` + `get-device`
    // + `[method]gpu-device.queue` (S1: sync getter → `own<gpu-queue>`)
    // and `[method]gpu-device.create-command-encoder` (S6: sync
    // (borrow, option<gpu-command-encoder-descriptor>) -> own<gpu-command-encoder>; L2 described label) and `[method]gpu-device.create-buffer`
    // (S4: sync (borrow, gpu-buffer-descriptor) -> own<gpu-buffer>) and
    // `gpu-buffer` + `get-buffer` + `[method]gpu-buffer.map-async` (S6+: true async
    // result<_, map-async-error>; guest mode/offset/size; L2 still host-fixed MAP_READ buffer)
    // and `[method]gpu-buffer.unmap` (S6+: result<_, unmap-error>; L2 described buffer rep)
    // and `[method]gpu-device.create-texture` (S6+: sync (borrow, gpu-texture-descriptor) -> own<gpu-texture>; L2 described size/format/usage/mip/sample/dimension + view-formats + label) and
    // `[method]gpu-device.create-sampler` (S8: sync (borrow, option<gpu-sampler-descriptor>) -> own<gpu-sampler>)
    // and S6+ `[method]gpu-device.create-shader-module` (sync (borrow, gpu-shader-module-descriptor) -> own<gpu-shader-module>; L2 described WGSL code + label + compilation-hints)
    // and `[method]gpu-queue.write-buffer-with-copy` (S6+: borrow buffer + list data → result; L2 described bytes + offset)
    // and S5 `[method]gpu-queue.submit` (sync void; list<borrow<gpu-command-buffer>>; L2 described handles)
    // and S7 `[method]gpu-command-encoder.finish` (sync (borrow, option<gpu-command-buffer-descriptor>) -> own<gpu-command-buffer>; L2 described label)
    // and `gpu-texture` + `get-texture` + S8 `[method]gpu-texture.create-view` (sync (borrow, option<gpu-texture-view-descriptor>) -> own<gpu-texture-view>)
    // and S6+ `[method]gpu-texture.*` info getters / label / set-label (L2 described extent: width/height/depth/mip; remaining still lift-only).
    // and S6+ `[method]record-gpu-pipeline-constant-value.*` map methods (L2 described mutate + iterate).
    // and S6+ `[method]gpu-device.create-bind-group-layout` (sync (borrow, gpu-bind-group-layout-descriptor) -> own<gpu-bind-group-layout>; L2 described all entries)
    // and S6+ `[method]gpu-device.create-pipeline-layout` (sync (borrow, gpu-pipeline-layout-descriptor) -> own<gpu-pipeline-layout>; L2 described BGL handles + label)
    // and S6+ `[method]gpu-device.create-bind-group` (sync (borrow, gpu-bind-group-descriptor) -> own<gpu-bind-group>; L2 described layout + entries + label)
    // and S6+ `[method]gpu-device.create-render-pipeline` (sync (borrow, gpu-render-pipeline-descriptor) -> own<gpu-render-pipeline>; L2 described vertex.buffers + guest color format)
    // and S6+ `[method]gpu-device.create-compute-pipeline` (sync (borrow, gpu-compute-pipeline-descriptor) -> own<gpu-compute-pipeline>; L2 described shader/entry/layout/label)
    // and `[method]gpu-queue.write-texture-with-copy` (S6+: texel copy info + list data; L2 described bytes + size)
    // and S8 `[method]gpu-command-encoder.begin-compute-pass` (sync (borrow, option<gpu-compute-pass-descriptor>) -> own<gpu-compute-pass-encoder>; L2 described timestamp-write indices)
    // and S6+ `[method]gpu-command-encoder.begin-render-pass` (sync (borrow, gpu-render-pass-descriptor) -> own<gpu-render-pass-encoder>; L2 described all color attachments + depth-stencil)
    // and `gpu-compute-pass-encoder` + `get-compute-pass` + `[method]gpu-compute-pass-encoder.end` (sync void; L2 described pass rep)
    // and `[method]gpu-compute-pass-encoder.set-pipeline` (S6+: borrow<gpu-compute-pipeline>; L2 described pass+pipeline reps)
    // and `[method]gpu-compute-pass-encoder.set-bind-group` (S6+: index + option bind-group + option offsets → result; L2 described JNI, offsets none → empty)
    // and `[method]gpu-compute-pass-encoder.dispatch-workgroups` (S6+: x + option y/z; L2 described JNI)
    // and S6+ remaining compute-pass recording: dispatch-workgroups-indirect (L2 described JNI) / set-immediates /
    // push-debug-group / pop-debug-group / insert-debug-marker
    // and S6+ render-pass debug: push-debug-group / pop-debug-group / insert-debug-marker
    // and S6+ remaining render-pass: begin-occlusion-query / end-occlusion-query /
    // execute-bundles / set-immediates
    // and S6+ render-bundle-encoder: finish / set-pipeline / set-bind-group / draw /
    // set-index-buffer / set-vertex-buffer / draw-indexed / draw-indirect /
    // draw-indexed-indirect / push-debug-group / pop-debug-group / insert-debug-marker /
    // set-immediates.
    // and S6+ remaining device create + destroy: create-render-bundle-encoder /
    // create-query-set / device.destroy / buffer.destroy / texture.destroy /
    // query-set.destroy / query-set.type / query-set.count.
    // and S6+ adapter info: adapter.features / limits / info + adapter-info getters.
    // and S6+ bind-group / bind-group-layout / buffer label + set-label and
    // buffer size / usage / map-state.
    // and S6+ command-buffer / encoder label + compilation-info.messages +
    // compilation-message getters.
    // and S6+ compute-pass-encoder / compute-pipeline label + set-label and
    // compute-pipeline.get-bind-group-layout.
    // and S6+ gpu-device adapter-info / features / limits / label / set-label /
    // lost / push-error-scope / pop-error-scope / on-uncaptured-error and
    // gpu-device-lost-info reason / message.
    // and S6+ render-bundle / render-bundle-encoder / render-pass-encoder label +
    // set-label and render-pipeline label / set-label / get-bind-group-layout.
    // and S6+ gpu-supported-limits max-* getters (lift-only stub numerics).
    // and `[method]gpu-render-pass-encoder.set-pipeline` (S6+: borrow<gpu-render-pipeline>; L2 described pass+pipeline reps)
    // and `[method]gpu-render-pass-encoder.draw` (S6+: vertex-count + option instance/first-*; L2 still host-fixed draw(3))
    // and `[method]gpu-render-pass-encoder.set-bind-group` (S6+: index + option bind-group + option offsets → result; L2 described JNI, offsets none → empty)
    // and `[method]gpu-render-pass-encoder.set-vertex-buffer` (S6+: slot + option buffer + option offset/size; L2 described JNI)
    // and `[method]gpu-render-pass-encoder.set-index-buffer` (S6+: buffer + index-format + option offset/size; L2 described JNI)
    // and `[method]gpu-command-encoder.copy-buffer-to-buffer` (S6+: borrow src/dst + option offsets/size; L2 still host-fixed 4-byte copy)
    // and S6+ remaining encoder recording: copy-buffer-to-texture / copy-texture-to-buffer /
    // copy-texture-to-texture / clear-buffer / resolve-query-set / push-debug-group /
    // pop-debug-group / insert-debug-marker.
    // Experimental stays sync.
    // S5: first canonical list is submit; other lists still later.
    {
        let mut webgpu = linker
            .instance("wasi:webgpu/webgpu@0.3.0-rc.2")
            .map_err(|e| e.to_string())?;
        webgpu
            .resource("gpu", ResourceType::host::<Gpu>(), |mut store, rep| {
                let resource = Resource::<Gpu>::new_own(rep);
                store.data_mut().table.delete(resource)?;
                Ok(())
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-gpu", |mut store, ()| {
                let resource = store.data_mut().table.push(Gpu)?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-adapter",
                ResourceType::host::<GpuAdapter>(),
                |mut store, rep| {
                    let resource = Resource::<GpuAdapter>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu.request-adapter",
                |accessor, (gpu, options): (Resource<Gpu>, Option<GpuRequestAdapterOptions>)| {
                    Box::pin(async move {
                        let cb = accessor.with(|mut access| -> wasmtime::Result<_> {
                            let _ = access.data_mut().table.get(&gpu)?;
                            Ok(access.data_mut().experimental_host_cb.clone())
                        })?;
                        // True CM async even when unwired (guest `none`, not a trap).
                        let (tx, rx) = oneshot::channel::<()>();
                        std::thread::spawn(move || {
                            let _ = tx.send(());
                        });
                        let _ = rx.await;
                        let Some(cb) = cb else {
                            return Ok((None,));
                        };
                        // L2: `power-preference` 0=none/undefined, 1=low-power, 2=high-performance.
                        // `force-fallback-adapter` 0=none/false, 1=true.
                        // `xr-compatible` -1=none, 0=false, 1=true.
                        let (power_preference, force_fallback, feature_level, xr_compatible) =
                            match options.as_ref()
                        {
                            None => (0i32, 0i32, String::new(), -1i32),
                            Some(opts) => {
                                let power = match opts.power_preference {
                                    None => 0,
                                    Some(p) => p as u8 as i32 + 1,
                                };
                                let fallback =
                                    i32::from(opts.force_fallback_adapter.unwrap_or(false));
                                let feature =
                                    opts.feature_level.clone().unwrap_or_default();
                                let xr = match opts.xr_compatible {
                                    None => -1,
                                    Some(false) => 0,
                                    Some(true) => 1,
                                };
                                (power, fallback, feature, xr)
                            }
                        };
                        let adapter_rep = jvm::exp_request_adapter_described(
                            &cb,
                            power_preference,
                            force_fallback,
                            feature_level,
                            xr_compatible,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        if adapter_rep == 0 {
                            return Ok((None,));
                        }
                        let resource = accessor.with(|mut access| {
                            access
                                .data_mut()
                                .table
                                .push(GpuAdapter { rep: adapter_rep })
                        })?;
                        Ok((Some(resource),))
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "wgsl-language-features",
                ResourceType::host::<WgslLanguageFeatures>(),
                |mut store, rep| {
                    let resource = Resource::<WgslLanguageFeatures>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu.get-preferred-canvas-format",
                |mut caller, (gpu,): (Resource<Gpu>,)| {
                    let _ = caller.data_mut().table.get(&gpu)?;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let dawn = jvm::exp_gpu_get_preferred_canvas_format_described(&cb)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuTextureFormat::from_dawn_u32(dawn),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu.wgsl-language-features",
                |mut caller, (gpu,): (Resource<Gpu>,)| {
                    let _ = caller.data_mut().table.get(&gpu)?;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    jvm::exp_gpu_wgsl_language_features_described(&cb)
                        .map_err(wasmtime::Error::msg)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(WgslLanguageFeatures { gpu: 0 })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]wgsl-language-features.has",
                |mut caller, (features, value): (Resource<WgslLanguageFeatures>, String)| {
                    let _features_gpu = caller.data_mut().table.get(&features)?.gpu;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let has = jvm::exp_wgsl_language_features_has_described(&cb, value)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((has != 0,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-adapter", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuAdapter { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-supported-features",
                ResourceType::host::<GpuSupportedFeatures>(),
                |mut store, rep| {
                    let resource = Resource::<GpuSupportedFeatures>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-supported-limits",
                ResourceType::host::<GpuSupportedLimits>(),
                |mut store, rep| {
                    let resource = Resource::<GpuSupportedLimits>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-adapter-info",
                ResourceType::host::<GpuAdapterInfo>(),
                |mut store, rep| {
                    let resource = Resource::<GpuAdapterInfo>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter.features",
                |mut caller, (adapter,): (Resource<GpuAdapter>,)| {
                    let adapter_rep = caller.data_mut().table.get(&adapter)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_adapter = if adapter_rep == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        adapter_rep
                    };
                    jvm::exp_adapter_features_described(&cb, l2_adapter)
                        .map_err(wasmtime::Error::msg)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuSupportedFeatures {
                            adapter: l2_adapter,
                        })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-features.has",
                |mut caller, (features, value): (Resource<GpuSupportedFeatures>, String)| {
                    let features_adapter = caller.data_mut().table.get(&features)?.adapter;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_adapter = if features_adapter == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        features_adapter
                    };
                    let has = jvm::exp_supported_features_has_described(&cb, l2_adapter, value)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((has != 0,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter.limits",
                |mut caller, (adapter,): (Resource<GpuAdapter>,)| {
                    let adapter_rep = caller.data_mut().table.get(&adapter)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_adapter = if adapter_rep == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        adapter_rep
                    };
                    jvm::exp_adapter_limits_described(&cb, l2_adapter)
                        .map_err(wasmtime::Error::msg)?;
                    let resource = caller.data_mut().table.push(GpuSupportedLimits {
                        adapter: l2_adapter,
                        device: 0,
                    })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-supported-limits", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuSupportedLimits {
                    adapter: 0,
                    device: 0,
                })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-bind-groups",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (limits_adapter, limits_device) = {
                        let entry = caller.data_mut().table.get(&limits)?;
                        (entry.adapter, entry.device)
                    };
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_adapter = if limits_adapter == 0 && limits_device == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        limits_adapter
                    };
                    let value = jvm::exp_supported_limits_max_bind_groups_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-bind-groups-plus-vertex-buffers",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (limits_adapter, limits_device) = {
                        let entry = caller.data_mut().table.get(&limits)?;
                        (entry.adapter, entry.device)
                    };
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_adapter = if limits_adapter == 0 && limits_device == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        limits_adapter
                    };
                    let value =
                        jvm::exp_supported_limits_max_bind_groups_plus_vertex_buffers_described(
                            &cb,
                            l2_adapter,
                            limits_device,
                        )
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-bindings-per-bind-group",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (limits_adapter, limits_device) = {
                        let entry = caller.data_mut().table.get(&limits)?;
                        (entry.adapter, entry.device)
                    };
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_adapter = if limits_adapter == 0 && limits_device == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        limits_adapter
                    };
                    let value = jvm::exp_supported_limits_max_bindings_per_bind_group_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-buffer-size",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (limits_adapter, limits_device) = {
                        let entry = caller.data_mut().table.get(&limits)?;
                        (entry.adapter, entry.device)
                    };
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_adapter = if limits_adapter == 0 && limits_device == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        limits_adapter
                    };
                    let value = jvm::exp_supported_limits_max_buffer_size_described(
                        &cb,
                        l2_adapter,
                        limits_device,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-color-attachment-bytes-per-sample",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_color_attachment_bytes_per_sample_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-color-attachments",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_color_attachments_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-invocations-per-workgroup",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_compute_invocations_per_workgroup_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-workgroup-size-x",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_compute_workgroup_size_x_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-workgroup-size-y",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_compute_workgroup_size_y_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-workgroup-size-z",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_compute_workgroup_size_z_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-workgroups-per-dimension",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_compute_workgroups_per_dimension_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-workgroup-storage-size",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_compute_workgroup_storage_size_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-dynamic-storage-buffers-per-pipeline-layout",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_dynamic_storage_buffers_per_pipeline_layout_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-dynamic-uniform-buffers-per-pipeline-layout",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_dynamic_uniform_buffers_per_pipeline_layout_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-immediate-size",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_immediate_size_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-inter-stage-shader-variables",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_inter_stage_shader_variables_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-sampled-textures-per-shader-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_sampled_textures_per_shader_stage_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-samplers-per-shader-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_samplers_per_shader_stage_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-buffer-binding-size",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_storage_buffer_binding_size_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-buffers-in-fragment-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_storage_buffers_in_fragment_stage_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-buffers-in-vertex-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_storage_buffers_in_vertex_stage_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-buffers-per-shader-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_storage_buffers_per_shader_stage_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-textures-in-fragment-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_storage_textures_in_fragment_stage_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-textures-in-vertex-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_storage_textures_in_vertex_stage_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-textures-per-shader-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_storage_textures_per_shader_stage_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-texture-array-layers",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_texture_array_layers_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-texture-dimension1-d",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_texture_dimension1_d_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-texture-dimension2-d",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_texture_dimension2_d_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-texture-dimension3-d",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_texture_dimension3_d_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-uniform-buffer-binding-size",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_uniform_buffer_binding_size_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-uniform-buffers-per-shader-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_uniform_buffers_per_shader_stage_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-vertex-attributes",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_vertex_attributes_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-vertex-buffer-array-stride",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_vertex_buffer_array_stride_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-vertex-buffers",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_max_vertex_buffers_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.min-storage-buffer-offset-alignment",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_min_storage_buffer_offset_alignment_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.min-uniform-buffer-offset-alignment",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let (cb, l2_adapter, limits_device) =
                        l2_supported_limits_handles(&mut caller, &limits)?;
                    let value = jvm::exp_supported_limits_min_uniform_buffer_offset_alignment_described(&cb, l2_adapter, limits_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((value,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter.info",
                |mut caller, (adapter,): (Resource<GpuAdapter>,)| {
                    let adapter_rep = caller.data_mut().table.get(&adapter)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_adapter = if adapter_rep == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        adapter_rep
                    };
                    jvm::exp_adapter_info_described(&cb, l2_adapter)
                        .map_err(wasmtime::Error::msg)?;
                    let resource = caller.data_mut().table.push(GpuAdapterInfo {
                        adapter: l2_adapter,
                    })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-adapter-info", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuAdapterInfo { adapter: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.vendor",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    let info_adapter = caller.data_mut().table.get(&info)?.adapter;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_adapter = if info_adapter == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        info_adapter
                    };
                    let vendor = jvm::exp_adapter_info_vendor_described(&cb, l2_adapter)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((vendor,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.architecture",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    let info_adapter = caller.data_mut().table.get(&info)?.adapter;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_adapter = if info_adapter == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        info_adapter
                    };
                    let architecture =
                        jvm::exp_adapter_info_architecture_described(&cb, l2_adapter)
                            .map_err(wasmtime::Error::msg)?;
                    Ok((architecture,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.device",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    let info_adapter = caller.data_mut().table.get(&info)?.adapter;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_adapter = if info_adapter == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        info_adapter
                    };
                    let device = jvm::exp_adapter_info_device_described(&cb, l2_adapter)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((device,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.description",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    let info_adapter = caller.data_mut().table.get(&info)?.adapter;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_adapter = if info_adapter == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        info_adapter
                    };
                    let description = jvm::exp_adapter_info_description_described(&cb, l2_adapter)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((description,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.subgroup-min-size",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    let info_adapter = caller.data_mut().table.get(&info)?.adapter;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_adapter = if info_adapter == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        info_adapter
                    };
                    let size = jvm::exp_adapter_info_subgroup_min_size_described(&cb, l2_adapter)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((size,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.subgroup-max-size",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    let info_adapter = caller.data_mut().table.get(&info)?.adapter;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_adapter = if info_adapter == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        info_adapter
                    };
                    let size = jvm::exp_adapter_info_subgroup_max_size_described(&cb, l2_adapter)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((size,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.is-fallback-adapter",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    let info_adapter = caller.data_mut().table.get(&info)?.adapter;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_adapter = if info_adapter == 0 {
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                    } else {
                        info_adapter
                    };
                    let fallback =
                        jvm::exp_adapter_info_is_fallback_adapter_described(&cb, l2_adapter)
                            .map_err(wasmtime::Error::msg)?;
                    Ok((fallback != 0,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "record-option-gpu-size64",
                ResourceType::host::<RecordOptionGpuSize64>(),
                |mut store, rep| {
                    let resource = Resource::<RecordOptionGpuSize64>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("[constructor]record-option-gpu-size64", |mut store, ()| {
                let resource = store.data_mut().table.push(RecordOptionGpuSize64)?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.add",
                |mut caller,
                 (record, key, value): (Resource<RecordOptionGpuSize64>, String, Option<u64>)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let (has_value, raw) = match value {
                        None => (0i32, 0u64),
                        Some(v) => (1i32, v),
                    };
                    jvm::exp_record_option_gpu_size64_add_described(
                        &cb, handle, key, has_value, raw,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.get",
                |mut caller, (record, key): (Resource<RecordOptionGpuSize64>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let state = jvm::exp_record_option_gpu_size64_get_state_described(
                        &cb,
                        handle,
                        key.clone(),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    match state {
                        0 => Ok((None,)),
                        1 => Ok((Some(None),)),
                        _ => {
                            let raw = jvm::exp_record_option_gpu_size64_get_value_described(
                                &cb, handle, key,
                            )
                            .map_err(wasmtime::Error::msg)?;
                            Ok((Some(Some(raw)),))
                        }
                    }
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.has",
                |mut caller, (record, key): (Resource<RecordOptionGpuSize64>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let has = jvm::exp_record_option_gpu_size64_has_described(&cb, handle, key)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((has != 0,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.remove",
                |mut caller, (record, key): (Resource<RecordOptionGpuSize64>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    jvm::exp_record_option_gpu_size64_remove_described(&cb, handle, key)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.keys",
                |mut caller, (record,): (Resource<RecordOptionGpuSize64>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let count = jvm::exp_record_option_gpu_size64_keys_count_described(&cb, handle)
                        .map_err(wasmtime::Error::msg)?;
                    let mut keys = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        keys.push(
                            jvm::exp_record_option_gpu_size64_keys_get_described(&cb, handle, i)
                                .map_err(wasmtime::Error::msg)?,
                        );
                    }
                    Ok((keys,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.values",
                |mut caller, (record,): (Resource<RecordOptionGpuSize64>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let count =
                        jvm::exp_record_option_gpu_size64_values_count_described(&cb, handle)
                            .map_err(wasmtime::Error::msg)?;
                    let mut values = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        let state = jvm::exp_record_option_gpu_size64_values_get_state_described(
                            &cb, handle, i,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        if state == 0 {
                            values.push(None);
                        } else {
                            let raw = jvm::exp_record_option_gpu_size64_values_get_value_described(
                                &cb, handle, i,
                            )
                            .map_err(wasmtime::Error::msg)?;
                            values.push(Some(raw));
                        }
                    }
                    Ok((values,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.entries",
                |mut caller, (record,): (Resource<RecordOptionGpuSize64>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let count =
                        jvm::exp_record_option_gpu_size64_entries_count_described(&cb, handle)
                            .map_err(wasmtime::Error::msg)?;
                    let mut entries = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        let key = jvm::exp_record_option_gpu_size64_entries_get_key_described(
                            &cb, handle, i,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        let state = jvm::exp_record_option_gpu_size64_entries_get_state_described(
                            &cb, handle, i,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        let value = if state == 0 {
                            None
                        } else {
                            Some(
                                jvm::exp_record_option_gpu_size64_entries_get_value_described(
                                    &cb, handle, i,
                                )
                                .map_err(wasmtime::Error::msg)?,
                            )
                        };
                        entries.push((key, value));
                    }
                    Ok((entries,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-device",
                ResourceType::host::<GpuDevice>(),
                |mut store, rep| {
                    let resource = Resource::<GpuDevice>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu-adapter.request-device",
                |accessor, (adapter, descriptor): (
                    Resource<GpuAdapter>,
                    Option<GpuDeviceDescriptor>,
                )| {
                    Box::pin(async move {
                        let (
                            cb,
                            adapter_rep,
                            required_features,
                            required_limits,
                            label,
                            default_queue_label,
                        ) = accessor.with(|mut access| {
                            let adapter_rep = access.data_mut().table.get(&adapter)?.rep;
                            let (required_features, required_limits, label, default_queue_label) =
                                match descriptor.as_ref() {
                                    None => (Vec::new(), 0i32, String::new(), String::new()),
                                    Some(d) => {
                                        let required_features = d
                                            .required_features
                                            .as_ref()
                                            .map(|v| {
                                                v.iter().map(|f| *f as u8 as i32).collect()
                                            })
                                            .unwrap_or_default();
                                        let required_limits = d
                                            .required_limits
                                            .as_ref()
                                            .map(|r| r.rep() as i32)
                                            .unwrap_or(0);
                                        let label = d.label.clone().unwrap_or_default();
                                        let default_queue_label = d
                                            .default_queue
                                            .as_ref()
                                            .and_then(|q| q.label.clone())
                                            .unwrap_or_default();
                                        (
                                            required_features,
                                            required_limits,
                                            label,
                                            default_queue_label,
                                        )
                                    }
                                };
                            let cb = access
                                .data_mut()
                                .experimental_host_cb
                                .as_ref()
                                .ok_or_else(|| {
                                    wasmtime::Error::msg("experimental host callback not set")
                                })
                                .cloned()?;
                            Ok::<_, wasmtime::Error>((
                                cb,
                                adapter_rep,
                                required_features,
                                required_limits,
                                label,
                                default_queue_label,
                            ))
                        })?;
                        let (tx, rx) = oneshot::channel::<()>();
                        std::thread::spawn(move || {
                            let _ = tx.send(());
                        });
                        let _ = rx.await;
                        let l2_adapter = if adapter_rep == 0 {
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?
                        } else {
                            adapter_rep
                        };
                        let device_rep = jvm::exp_adapter_request_device_described(
                            &cb,
                            l2_adapter,
                            required_features,
                            required_limits,
                            label,
                            default_queue_label,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        if device_rep == 0 {
                            return Ok((Err(RequestDeviceError {
                                kind: RequestDeviceErrorKind::OperationError,
                                message: "adapter-request-device returned 0".into(),
                            }),));
                        }
                        let resource = accessor.with(|mut access| {
                            access
                                .data_mut()
                                .table
                                .push(GpuDevice { rep: device_rep })
                        })?;
                        Ok((Ok(resource),))
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-device", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuDevice { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-queue",
                ResourceType::host::<GpuQueue>(),
                |mut store, rep| {
                    let resource = Resource::<GpuQueue>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.queue",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let queue_rep = jvm::exp_device_get_queue_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    if queue_rep == 0 {
                        return Err(wasmtime::Error::msg("device-queue returned 0"));
                    }
                    let resource = caller.data_mut().table.push(GpuQueue { rep: queue_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.destroy",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    jvm::exp_device_destroy_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-device-lost-info",
                ResourceType::host::<GpuDeviceLostInfo>(),
                |mut store, rep| {
                    let resource = Resource::<GpuDeviceLostInfo>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-error",
                ResourceType::host::<GpuError>(),
                |mut store, rep| {
                    let resource = Resource::<GpuError>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-gpu-error", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuError { device: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-error.message",
                |mut caller, (error,): (Resource<GpuError>,)| {
                    let error_device = caller.data_mut().table.get(&error)?.device;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if error_device == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        error_device
                    };
                    let message = jvm::exp_gpu_error_message_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((message,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-error.kind",
                |mut caller, (error,): (Resource<GpuError>,)| {
                    let error_device = caller.data_mut().table.get(&error)?.device;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if error_device == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        error_device
                    };
                    let kind = jvm::exp_gpu_error_kind_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuErrorKind::from_host_u32(kind),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.features",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    jvm::exp_device_features_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    let adapter_rep = jvm::exp_device_adapter_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuSupportedFeatures {
                            adapter: adapter_rep,
                        })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.limits",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    jvm::exp_device_limits_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    let resource = caller.data_mut().table.push(GpuSupportedLimits {
                        adapter: 0,
                        device: l2_device,
                    })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.adapter-info",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    jvm::exp_device_adapter_info_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    let adapter_rep = jvm::exp_device_adapter_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    let resource = caller.data_mut().table.push(GpuAdapterInfo {
                        adapter: adapter_rep,
                    })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.label",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let label = jvm::exp_device_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.set-label",
                |mut caller, (device, label): (Resource<GpuDevice>, String)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    jvm::exp_device_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.lost",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    jvm::exp_device_lost_described(&cb, l2_device).map_err(wasmtime::Error::msg)?;
                    let info = caller
                        .data_mut()
                        .table
                        .push(GpuDeviceLostInfo { device: l2_device })?;
                    let fut = FutureReader::new(&mut caller, async move {
                        Ok::<Resource<GpuDeviceLostInfo>, wasmtime::Error>(info)
                    })?;
                    Ok((fut,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.push-error-scope",
                |mut caller, (device, filter): (Resource<GpuDevice>, GpuErrorFilter)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    jvm::exp_device_push_error_scope_described(
                        &cb,
                        l2_device,
                        filter.to_host_u32(),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu-device.pop-error-scope",
                |accessor, (device,): (Resource<GpuDevice>,)| {
                    Box::pin(async move {
                        let (cb, device_rep) =
                            accessor.with(|mut access| -> wasmtime::Result<_> {
                                let device_rep = access.data_mut().table.get(&device)?.rep;
                                let cb = access
                                    .data_mut()
                                    .experimental_host_cb
                                    .as_ref()
                                    .ok_or_else(|| {
                                        wasmtime::Error::msg("experimental host callback not set")
                                    })
                                    .cloned()?;
                                Ok((cb, device_rep))
                            })?;
                        let l2_device = if device_rep == 0 {
                            let adapter_rep =
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                            jvm::exp_adapter_request_device(&cb, adapter_rep)
                                .map_err(wasmtime::Error::msg)?
                        } else {
                            device_rep
                        };
                        let _ = jvm::exp_device_pop_error_scope_described(&cb, l2_device)
                            .map_err(wasmtime::Error::msg)?;
                        Ok((Ok::<Option<Resource<GpuError>>, PopErrorScopeError>(None),))
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.on-uncaptured-error",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    jvm::exp_device_on_uncaptured_error_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    let reader = StreamReader::<Resource<GpuError>>::new(&mut caller, vec![])?;
                    Ok((reader,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-uncaptured-error-event",
                ResourceType::host::<GpuUncapturedErrorEvent>(),
                |mut store, rep| {
                    let resource = Resource::<GpuUncapturedErrorEvent>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-uncaptured-error-event", |mut store, ()| {
                let resource = store
                    .data_mut()
                    .table
                    .push(GpuUncapturedErrorEvent { device: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-uncaptured-error-event.error",
                |mut caller, (event,): (Resource<GpuUncapturedErrorEvent>,)| {
                    let event_device = caller.data_mut().table.get(&event)?.device;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if event_device == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        event_device
                    };
                    jvm::exp_uncaptured_error_event_error_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuError { device: l2_device })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-device-lost-info", |mut store, ()| {
                let resource = store
                    .data_mut()
                    .table
                    .push(GpuDeviceLostInfo { device: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device-lost-info.reason",
                |mut caller, (info,): (Resource<GpuDeviceLostInfo>,)| {
                    let info_device = caller.data_mut().table.get(&info)?.device;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if info_device == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        info_device
                    };
                    let reason = jvm::exp_device_lost_info_reason_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuDeviceLostReason::from_host_u32(reason),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device-lost-info.message",
                |mut caller, (info,): (Resource<GpuDeviceLostInfo>,)| {
                    let info_device = caller.data_mut().table.get(&info)?.device;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if info_device == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        info_device
                    };
                    let message = jvm::exp_device_lost_info_message_described(&cb, l2_device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((message,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-command-encoder",
                ResourceType::host::<GpuCommandEncoder>(),
                |mut store, rep| {
                    let resource = Resource::<GpuCommandEncoder>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-command-encoder",
                |mut caller,
                 (device, descriptor): (
                    Resource<GpuDevice>,
                    Option<GpuCommandEncoderDescriptor>,
                )| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let label = descriptor
                        .as_ref()
                        .and_then(|d| d.label.clone())
                        .unwrap_or_default();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let encoder_rep =
                        jvm::exp_create_command_encoder_described(&cb, l2_device, label)
                            .map_err(wasmtime::Error::msg)?;
                    if encoder_rep == 0 {
                        return Err(wasmtime::Error::msg(
                            "device-create-command-encoder returned 0",
                        ));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuCommandEncoder { rep: encoder_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-buffer",
                |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuBufferDescriptor)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let mapped = match descriptor.mapped_at_creation {
                        None => -1,
                        Some(false) => 0,
                        Some(true) => 1,
                    };
                    let label = descriptor.label.clone().unwrap_or_default();
                    let buffer_rep = jvm::exp_create_buffer_described(
                        &cb,
                        l2_device,
                        descriptor.size,
                        descriptor.usage.to_webgpu_u32(),
                        mapped,
                        label,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if buffer_rep == 0 {
                        return Err(wasmtime::Error::msg("device-create-buffer returned 0"));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuBuffer { rep: buffer_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-texture",
                ResourceType::host::<GpuTexture>(),
                |mut store, rep| {
                    let resource = Resource::<GpuTexture>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-texture",
                |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuTextureDescriptor)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let width = descriptor.size.width;
                    let height = descriptor.size.height.unwrap_or(1);
                    let depth = descriptor.size.depth_or_array_layers.unwrap_or(1);
                    let mip = descriptor.mip_level_count.unwrap_or(1);
                    let sample = descriptor.sample_count.unwrap_or(1);
                    // WIT d1/d2/d3 → Dawn TextureDimension 1D=1, 2D=2, 3D=3 (none → 2D).
                    let dimension = match descriptor.dimension {
                        Some(GpuTextureDimension::D1) => 1u32,
                        Some(GpuTextureDimension::D3) => 3,
                        Some(GpuTextureDimension::D2) | None => 2,
                    };
                    let view_formats: Vec<i32> = descriptor
                        .view_formats
                        .as_ref()
                        .map(|v| v.iter().map(|f| f.to_dawn_u32() as i32).collect())
                        .unwrap_or_default();
                    let label = descriptor.label.clone().unwrap_or_default();
                    let texture_rep = jvm::exp_create_texture_described(
                        &cb,
                        l2_device,
                        width,
                        height,
                        depth,
                        descriptor.format.to_dawn_u32(),
                        descriptor.usage.to_webgpu_u32(),
                        mip,
                        sample,
                        dimension,
                        view_formats,
                        label,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if texture_rep == 0 {
                        return Err(wasmtime::Error::msg("device-create-texture returned 0"));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuTexture { rep: texture_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-texture", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuTexture { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-canvas-context",
                ResourceType::host::<GpuCanvasContext>(),
                |mut store, rep| {
                    let resource = Resource::<GpuCanvasContext>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-canvas-context", |mut store, ()| {
                let resource = store
                    .data_mut()
                    .table
                    .push(GpuCanvasContext { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-canvas-context.configure",
                |mut caller, (ctx, config): (Resource<GpuCanvasContext>, GpuCanvasConfiguration)| {
                    let ctx_rep = caller.data_mut().table.get(&ctx)?.rep;
                    let device_rep = caller.data_mut().table.get(&config.device)?.rep;
                    let format = config.format.to_dawn_u32();
                    let usage = config.usage.map(|u| u.to_webgpu_u32()).unwrap_or(0);
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| {
                            wasmtime::Error::msg("experimental host callback not set")
                        })
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let handle = jvm::exp_canvas_context_configure_described(
                        &cb, ctx_rep, l2_device, format, usage,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if handle == 0 {
                        return Err(wasmtime::Error::msg(
                            "canvas-context-configure returned 0",
                        ));
                    }
                    caller.data_mut().table.get_mut(&ctx)?.rep = handle;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-canvas-context.unconfigure",
                |mut caller, (ctx,): (Resource<GpuCanvasContext>,)| {
                    let ctx_rep = caller.data_mut().table.get(&ctx)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| {
                            wasmtime::Error::msg("experimental host callback not set")
                        })
                        .cloned()?;
                    jvm::exp_canvas_context_unconfigure_described(&cb, ctx_rep)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-canvas-context.get-configuration",
                |mut caller, (ctx,): (Resource<GpuCanvasContext>,)| {
                    let ctx_rep = caller.data_mut().table.get(&ctx)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| {
                            wasmtime::Error::msg("experimental host callback not set")
                        })
                        .cloned()?;
                    let has = jvm::exp_canvas_context_has_configuration_described(&cb, ctx_rep)
                        .map_err(wasmtime::Error::msg)?;
                    if has == 0 {
                        return Ok((Option::<GpuCanvasConfigurationOwned>::None,));
                    }
                    let device_rep =
                        jvm::exp_canvas_context_configuration_device_described(&cb, ctx_rep)
                            .map_err(wasmtime::Error::msg)?;
                    let format =
                        jvm::exp_canvas_context_configuration_format_described(&cb, ctx_rep)
                            .map_err(wasmtime::Error::msg)?;
                    let usage =
                        jvm::exp_canvas_context_configuration_usage_described(&cb, ctx_rep)
                            .map_err(wasmtime::Error::msg)?;
                    let device = caller
                        .data_mut()
                        .table
                        .push(GpuDevice { rep: device_rep })?;
                    let usage_opt = if usage == 0 {
                        None
                    } else {
                        Some(GpuTextureUsage::from_webgpu_u32(usage))
                    };
                    Ok((
                        Some(GpuCanvasConfigurationOwned {
                            device,
                            format: GpuTextureFormat::from_dawn_u32(format),
                            usage: usage_opt,
                            view_formats: None,
                            color_space: None,
                            tone_mapping: None,
                            alpha_mode: None,
                        }),
                    ))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-canvas-context.get-current-texture",
                |mut caller, (ctx,): (Resource<GpuCanvasContext>,)| {
                    let ctx_rep = caller.data_mut().table.get(&ctx)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| {
                            wasmtime::Error::msg("experimental host callback not set")
                        })
                        .cloned()?;
                    let texture_rep =
                        jvm::exp_canvas_context_get_current_texture_described(&cb, ctx_rep)
                            .map_err(wasmtime::Error::msg)?;
                    if texture_rep == 0 {
                        return Err(wasmtime::Error::msg(
                            "canvas-context-get-current-texture returned 0",
                        ));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuTexture { rep: texture_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-texture-view",
                ResourceType::host::<GpuTextureView>(),
                |mut store, rep| {
                    let resource = Resource::<GpuTextureView>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.create-view",
                |mut caller,
                 (texture, descriptor): (
                    Resource<GpuTexture>,
                    Option<GpuTextureViewDescriptor>,
                )| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let (dimension, aspect, format, base_mip, mip_count, base_layer, layer_count) =
                        match &descriptor {
                            None => (0, 0, 0, 0, -1, 0, -1),
                            Some(d) => (
                                d.dimension.map(|m| m.to_dawn_u32()).unwrap_or(0),
                                d.aspect.map(|m| m.to_dawn_u32()).unwrap_or(0),
                                d.format.map(|m| m.to_dawn_u32()).unwrap_or(0),
                                d.base_mip_level.unwrap_or(0) as i32,
                                d.mip_level_count.map(|v| v as i32).unwrap_or(-1),
                                d.base_array_layer.unwrap_or(0) as i32,
                                d.array_layer_count.map(|v| v as i32).unwrap_or(-1),
                            ),
                        };
                    let view_rep = jvm::exp_texture_create_view_described(
                        &cb,
                        l2_texture,
                        dimension,
                        aspect,
                        format,
                        base_mip,
                        mip_count,
                        base_layer,
                        layer_count,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if view_rep == 0 {
                        return Err(wasmtime::Error::msg("texture-create-view returned 0"));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuTextureView { rep: view_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-texture-view", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuTextureView { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture-view.label",
                |mut caller, (view,): (Resource<GpuTextureView>,)| {
                    let view_rep = caller.data_mut().table.get(&view)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if view_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let texture_rep = jvm::exp_create_texture(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_texture_create_view_described(
                            &cb, texture_rep, 0, 0, 0, 0, -1, 0, -1,
                        )
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        view_rep
                    };
                    let label = jvm::exp_texture_view_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture-view.set-label",
                |mut caller, (view, label): (Resource<GpuTextureView>, String)| {
                    let view_rep = caller.data_mut().table.get(&view)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if view_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let texture_rep = jvm::exp_create_texture(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_texture_create_view_described(
                            &cb, texture_rep, 0, 0, 0, 0, -1, 0, -1,
                        )
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        view_rep
                    };
                    jvm::exp_texture_view_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.destroy",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    jvm::exp_texture_destroy_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.width",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let width = jvm::exp_texture_width_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((width,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.height",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let height = jvm::exp_texture_height_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((height,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.depth-or-array-layers",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let depth = jvm::exp_texture_depth_or_array_layers_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((depth,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.mip-level-count",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let mip = jvm::exp_texture_mip_level_count_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((mip,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.sample-count",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let sample = jvm::exp_texture_sample_count_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((sample,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.dimension",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let dawn = jvm::exp_texture_dimension_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuTextureDimension::from_dawn_u32(dawn),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.format",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let dawn = jvm::exp_texture_format_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuTextureFormat::from_dawn_u32(dawn),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.usage",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let bits = jvm::exp_texture_usage_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuTextureUsage::from_webgpu_u32(bits),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.texture-binding-view-dimension",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_texture = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let dawn = jvm::exp_texture_binding_view_dimension_described(&cb, l2_texture)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuTextureViewDimension::from_dawn_u32(dawn),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.label",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    let label = jvm::exp_texture_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.set-label",
                |mut caller, (texture, label): (Resource<GpuTexture>, String)| {
                    let texture_rep = caller.data_mut().table.get(&texture)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if texture_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    jvm::exp_texture_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-buffer",
                ResourceType::host::<GpuBuffer>(),
                |mut store, rep| {
                    let resource = Resource::<GpuBuffer>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-buffer", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuBuffer { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.size",
                |mut caller, (buffer,): (Resource<GpuBuffer>,)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    let size = jvm::exp_buffer_size_described(&cb, l2_buffer)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((size,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.usage",
                |mut caller, (buffer,): (Resource<GpuBuffer>,)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    let bits = jvm::exp_buffer_usage_described(&cb, l2_buffer)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuBufferUsage::from_webgpu_u32(bits),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.map-state",
                |mut caller, (buffer,): (Resource<GpuBuffer>,)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    let state = jvm::exp_buffer_map_state_described(&cb, l2_buffer)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuBufferMapState::from_host_u32(state),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.label",
                |mut caller, (buffer,): (Resource<GpuBuffer>,)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    let label = jvm::exp_buffer_label_described(&cb, l2_buffer)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.set-label",
                |mut caller, (buffer, label): (Resource<GpuBuffer>, String)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    jvm::exp_buffer_set_label_described(&cb, l2_buffer, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu-buffer.map-async",
                |accessor,
                 (buffer, mode, offset, size): (
                    Resource<GpuBuffer>,
                    GpuMapMode,
                    Option<u64>,
                    Option<u64>,
                )| {
                    Box::pin(async move {
                        let (cb, buffer_rep) =
                            accessor.with(|mut access| -> wasmtime::Result<_> {
                                let buffer_rep = access.data_mut().table.get(&buffer)?.rep;
                                let cb = access
                                    .data_mut()
                                    .experimental_host_cb
                                    .as_ref()
                                    .ok_or_else(|| {
                                        wasmtime::Error::msg("experimental host callback not set")
                                    })
                                    .cloned()?;
                                Ok((cb, buffer_rep))
                            })?;
                        let (tx, rx) = oneshot::channel::<()>();
                        std::thread::spawn(move || {
                            let _ = tx.send(());
                        });
                        let _ = rx.await;
                        let l2_buffer = if buffer_rep == 0 {
                            let adapter_rep =
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                            let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                                .map_err(wasmtime::Error::msg)?;
                            jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                        } else {
                            buffer_rep
                        };
                        jvm::exp_buffer_map_async_described(
                            &cb,
                            l2_buffer,
                            mode.to_webgpu_u32(),
                            offset.unwrap_or(0),
                            size.unwrap_or(4),
                        )
                        .map_err(wasmtime::Error::msg)?;
                        Ok((Ok::<(), MapAsyncError>(()),))
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.unmap",
                |mut caller, (buffer,): (Resource<GpuBuffer>,)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    jvm::exp_buffer_unmap_described(&cb, l2_buffer)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((Ok::<(), UnmapError>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.get-mapped-range-get-with-copy",
                |mut caller, (buffer, offset, size): (
                    Resource<GpuBuffer>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    let data = jvm::exp_buffer_get_mapped_range_described(
                        &cb,
                        l2_buffer,
                        offset.unwrap_or(0),
                        size.unwrap_or(4),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((Ok::<Vec<u8>, GetMappedRangeError>(data),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.get-mapped-range-set-with-copy",
                |mut caller,
                 (buffer, data, offset, size): (
                    Resource<GpuBuffer>,
                    Vec<u8>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    let _ = size;
                    jvm::exp_buffer_set_mapped_range_described(
                        &cb,
                        l2_buffer,
                        data,
                        offset.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((Ok::<(), GetMappedRangeError>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.destroy",
                |mut caller, (buffer,): (Resource<GpuBuffer>,)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_buffer = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    jvm::exp_buffer_destroy_described(&cb, l2_buffer)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-sampler",
                ResourceType::host::<GpuSampler>(),
                |mut store, rep| {
                    let resource = Resource::<GpuSampler>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-sampler",
                |mut caller, (device, descriptor): (
                    Resource<GpuDevice>,
                    Option<GpuSamplerDescriptor>,
                )| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let (
                        mag_filter,
                        min_filter,
                        address_mode_u,
                        address_mode_v,
                        address_mode_w,
                        mipmap_filter,
                        compare,
                        has_lod_min,
                        lod_min,
                        has_lod_max,
                        lod_max,
                    ) = match &descriptor {
                        None => (0, 0, 0, 0, 0, 0, 0, 0, 0.0, 0, 0.0),
                        Some(d) => (
                            d.mag_filter.map(|m| m.to_dawn_u32()).unwrap_or(0),
                            d.min_filter.map(|m| m.to_dawn_u32()).unwrap_or(0),
                            d.address_mode_u.map(|m| m.to_dawn_u32()).unwrap_or(0),
                            d.address_mode_v.map(|m| m.to_dawn_u32()).unwrap_or(0),
                            d.address_mode_w.map(|m| m.to_dawn_u32()).unwrap_or(0),
                            d.mipmap_filter
                                .map(|m| match m {
                                    GpuMipmapFilterMode::Nearest => 1u32,
                                    GpuMipmapFilterMode::Linear => 2,
                                })
                                .unwrap_or(0),
                            d.compare
                                .map(|c| match c {
                                    GpuCompareFunction::Never => 1u32,
                                    GpuCompareFunction::Less => 2,
                                    GpuCompareFunction::Equal => 3,
                                    GpuCompareFunction::LessEqual => 4,
                                    GpuCompareFunction::Greater => 5,
                                    GpuCompareFunction::NotEqual => 6,
                                    GpuCompareFunction::GreaterEqual => 7,
                                    GpuCompareFunction::Always => 8,
                                })
                                .unwrap_or(0),
                            if d.lod_min_clamp.is_some() { 1i32 } else { 0 },
                            d.lod_min_clamp.unwrap_or(0.0),
                            if d.lod_max_clamp.is_some() { 1i32 } else { 0 },
                            d.lod_max_clamp.unwrap_or(0.0),
                        ),
                    };
                    let sampler_rep = jvm::exp_create_sampler_described(
                        &cb,
                        l2_device,
                        mag_filter,
                        min_filter,
                        address_mode_u,
                        address_mode_v,
                        address_mode_w,
                        mipmap_filter,
                        compare,
                        has_lod_min,
                        lod_min,
                        has_lod_max,
                        lod_max,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if sampler_rep == 0 {
                        return Err(wasmtime::Error::msg("device-create-sampler returned 0"));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuSampler { rep: sampler_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-sampler", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuSampler { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-sampler.label",
                |mut caller, (sampler,): (Resource<GpuSampler>,)| {
                    let sampler_rep = caller.data_mut().table.get(&sampler)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if sampler_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_sampler_described(
                            &cb, device_rep, 0, 0, 0, 0, 0, 0, 0, 0, 0.0, 0, 0.0,
                        )
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        sampler_rep
                    };
                    let label = jvm::exp_sampler_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-sampler.set-label",
                |mut caller, (sampler, label): (Resource<GpuSampler>, String)| {
                    let sampler_rep = caller.data_mut().table.get(&sampler)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if sampler_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_sampler_described(
                            &cb, device_rep, 0, 0, 0, 0, 0, 0, 0, 0, 0.0, 0, 0.0,
                        )
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        sampler_rep
                    };
                    jvm::exp_sampler_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-pipeline-layout",
                ResourceType::host::<GpuPipelineLayout>(),
                |mut store, rep| {
                    let resource = Resource::<GpuPipelineLayout>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-shader-module",
                ResourceType::host::<GpuShaderModule>(),
                |mut store, rep| {
                    let resource = Resource::<GpuShaderModule>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-shader-module",
                |mut caller, (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuShaderModuleDescriptor,
                )| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let code = descriptor.code;
                    let label = descriptor.label.clone().unwrap_or_default();
                    let mut hint_layouts = Vec::new();
                    let mut hint_entries = String::new();
                    if let Some(hints) = &descriptor.compilation_hints {
                        for (i, h) in hints.iter().enumerate() {
                            if i > 0 {
                                hint_entries.push('\n');
                            }
                            hint_entries.push_str(&h.entry_point);
                            let layout = match &h.layout {
                                None => -1,
                                Some(GpuLayoutMode::Auto) => 0,
                                Some(GpuLayoutMode::Specific(layout)) => layout.rep() as i32,
                            };
                            hint_layouts.push(layout);
                        }
                    }
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let shader_rep = jvm::exp_create_shader_module_described(
                        &cb,
                        l2_device,
                        code,
                        label,
                        hint_layouts,
                        hint_entries,
                    )
                        .map_err(wasmtime::Error::msg)?;
                    if shader_rep == 0 {
                        return Err(wasmtime::Error::msg(
                            "device-create-shader-module returned 0",
                        ));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuShaderModule { rep: shader_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-shader-module", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuShaderModule { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu-shader-module.get-compilation-info",
                |accessor, (shader,): (Resource<GpuShaderModule>,)| {
                    Box::pin(async move {
                        let (cb, shader_rep) =
                            accessor.with(|mut access| -> wasmtime::Result<_> {
                                let shader_rep = access.data_mut().table.get(&shader)?.rep;
                                let cb = access
                                    .data_mut()
                                    .experimental_host_cb
                                    .as_ref()
                                    .ok_or_else(|| {
                                        wasmtime::Error::msg("experimental host callback not set")
                                    })
                                    .cloned()?;
                                Ok((cb, shader_rep))
                            })?;
                        let l2_shader = if shader_rep == 0 {
                            let adapter_rep =
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                            let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                                .map_err(wasmtime::Error::msg)?;
                            jvm::exp_create_shader_module(&cb, device_rep)
                                .map_err(wasmtime::Error::msg)?
                        } else {
                            shader_rep
                        };
                        jvm::exp_shader_module_get_compilation_info_described(&cb, l2_shader)
                            .map_err(wasmtime::Error::msg)?;
                        let resource = accessor
                            .with(|mut access| {
                                access.data_mut().table.push(GpuCompilationInfo {
                                    shader_module: l2_shader,
                                })
                            })?;
                        Ok((resource,))
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-shader-module.label",
                |mut caller, (shader,): (Resource<GpuShaderModule>,)| {
                    let shader_rep = caller.data_mut().table.get(&shader)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if shader_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        shader_rep
                    };
                    let label = jvm::exp_shader_module_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-shader-module.set-label",
                |mut caller, (shader, label): (Resource<GpuShaderModule>, String)| {
                    let shader_rep = caller.data_mut().table.get(&shader)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if shader_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        shader_rep
                    };
                    jvm::exp_shader_module_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "record-gpu-pipeline-constant-value",
                ResourceType::host::<RecordGpuPipelineConstantValue>(),
                |mut store, rep| {
                    let resource = Resource::<RecordGpuPipelineConstantValue>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[constructor]record-gpu-pipeline-constant-value",
                |mut store, ()| {
                    let resource = store
                        .data_mut()
                        .table
                        .push(RecordGpuPipelineConstantValue)?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.add",
                |mut caller,
                 (record, key, value): (
                    Resource<RecordGpuPipelineConstantValue>,
                    String,
                    f64,
                )| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    jvm::exp_record_pipeline_constant_value_add_described(&cb, handle, key, value)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.get",
                |mut caller, (record, key): (Resource<RecordGpuPipelineConstantValue>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let has = jvm::exp_record_pipeline_constant_value_has_described(
                        &cb,
                        handle,
                        key.clone(),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if has == 0 {
                        return Ok((None,));
                    }
                    let value = jvm::exp_record_pipeline_constant_value_get_value_described(
                        &cb, handle, key,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((Some(value),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.has",
                |mut caller, (record, key): (Resource<RecordGpuPipelineConstantValue>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let has =
                        jvm::exp_record_pipeline_constant_value_has_described(&cb, handle, key)
                            .map_err(wasmtime::Error::msg)?;
                    Ok((has != 0,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.remove",
                |mut caller, (record, key): (Resource<RecordGpuPipelineConstantValue>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    jvm::exp_record_pipeline_constant_value_remove_described(&cb, handle, key)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.keys",
                |mut caller, (record,): (Resource<RecordGpuPipelineConstantValue>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let count = jvm::exp_record_pipeline_constant_value_keys_count_described(
                        &cb, handle,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    let mut keys = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        keys.push(
                            jvm::exp_record_pipeline_constant_value_keys_get_described(
                                &cb, handle, i,
                            )
                            .map_err(wasmtime::Error::msg)?,
                        );
                    }
                    Ok((keys,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.values",
                |mut caller, (record,): (Resource<RecordGpuPipelineConstantValue>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let count = jvm::exp_record_pipeline_constant_value_values_count_described(
                        &cb, handle,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    let mut values = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        values.push(
                            jvm::exp_record_pipeline_constant_value_values_get_described(
                                &cb, handle, i,
                            )
                            .map_err(wasmtime::Error::msg)?,
                        );
                    }
                    Ok((values,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.entries",
                |mut caller, (record,): (Resource<RecordGpuPipelineConstantValue>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    let handle = record.rep();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let count = jvm::exp_record_pipeline_constant_value_entries_count_described(
                        &cb, handle,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    let mut entries = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        let key = jvm::exp_record_pipeline_constant_value_entries_get_key_described(
                            &cb, handle, i,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        let value =
                            jvm::exp_record_pipeline_constant_value_entries_get_value_described(
                                &cb, handle, i,
                            )
                            .map_err(wasmtime::Error::msg)?;
                        entries.push((key, value));
                    }
                    Ok((entries,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-bind-group-layout",
                ResourceType::host::<GpuBindGroupLayout>(),
                |mut store, rep| {
                    let resource = Resource::<GpuBindGroupLayout>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-bind-group-layout", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuBindGroupLayout { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-bind-group-layout.label",
                |mut caller, (layout,): (Resource<GpuBindGroupLayout>,)| {
                    let layout_rep = caller.data_mut().table.get(&layout)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if layout_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_bind_group_layout(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        layout_rep
                    };
                    let label = jvm::exp_bind_group_layout_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-bind-group-layout.set-label",
                |mut caller, (layout, label): (Resource<GpuBindGroupLayout>, String)| {
                    let layout_rep = caller.data_mut().table.get(&layout)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if layout_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_bind_group_layout(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        layout_rep
                    };
                    jvm::exp_bind_group_layout_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-bind-group-layout",
                |mut caller, (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuBindGroupLayoutDescriptor,
                )| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let mut bindings = Vec::with_capacity(descriptor.entries.len());
                    let mut visibilities = Vec::with_capacity(descriptor.entries.len());
                    let mut buffer_types = Vec::with_capacity(descriptor.entries.len());
                    let mut sampler_types = Vec::with_capacity(descriptor.entries.len());
                    let mut texture_sample_types = Vec::with_capacity(descriptor.entries.len());
                    for entry in &descriptor.entries {
                        bindings.push(entry.binding as i32);
                        let mut visibility = 0i32;
                        if entry.visibility.contains(GpuShaderStage::VERTEX) {
                            visibility |= 1;
                        }
                        if entry.visibility.contains(GpuShaderStage::FRAGMENT) {
                            visibility |= 2;
                        }
                        if entry.visibility.contains(GpuShaderStage::COMPUTE) {
                            visibility |= 4;
                        }
                        visibilities.push(visibility);
                        buffer_types.push(match entry.buffer.as_ref().and_then(|b| b.ty) {
                            Some(GpuBufferBindingType::Uniform) => 0,
                            Some(GpuBufferBindingType::Storage) => 1,
                            Some(GpuBufferBindingType::ReadOnlyStorage) => 2,
                            None => -1,
                        });
                        sampler_types.push(match &entry.sampler {
                            None => -1,
                            Some(sampler) => match sampler.ty {
                                Some(GpuSamplerBindingType::NonFiltering) => 1,
                                Some(GpuSamplerBindingType::Comparison) => 2,
                                Some(GpuSamplerBindingType::Filtering) | None => 0,
                            },
                        });
                        texture_sample_types.push(match &entry.texture {
                            None => -1,
                            Some(texture) => match texture.sample_type {
                                Some(GpuTextureSampleType::UnfilterableFloat) => 1,
                                Some(GpuTextureSampleType::Depth) => 2,
                                Some(GpuTextureSampleType::Sint) => 3,
                                Some(GpuTextureSampleType::Uint) => 4,
                                Some(GpuTextureSampleType::Float) | None => 0,
                            },
                        });
                    }
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let layout_rep = jvm::exp_create_bind_group_layout_described(
                        &cb,
                        l2_device,
                        bindings,
                        visibilities,
                        buffer_types,
                        sampler_types,
                        texture_sample_types,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if layout_rep == 0 {
                        return Err(wasmtime::Error::msg(
                            "device-create-bind-group-layout returned 0",
                        ));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuBindGroupLayout { rep: layout_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-pipeline-layout",
                |mut caller, (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuPipelineLayoutDescriptor,
                )| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let mut layouts = Vec::with_capacity(descriptor.bind_group_layouts.len());
                    for opt in &descriptor.bind_group_layouts {
                        match opt {
                            Some(layout) => {
                                layouts.push(caller.data_mut().table.get(layout)?.rep as i32);
                            }
                            None => layouts.push(0),
                        }
                    }
                    let label = descriptor.label.clone().unwrap_or_default();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let mut l2_layouts = Vec::with_capacity(layouts.len());
                    for layout_rep in layouts {
                        if layout_rep != 0 {
                            l2_layouts.push(layout_rep);
                            continue;
                        }
                        l2_layouts.push(
                            jvm::exp_create_bind_group_layout(&cb, l2_device)
                                .map_err(wasmtime::Error::msg)? as i32,
                        );
                    }
                    let layout_rep = jvm::exp_create_pipeline_layout_described(
                        &cb, l2_device, l2_layouts, label,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if layout_rep == 0 {
                        return Err(wasmtime::Error::msg(
                            "device-create-pipeline-layout returned 0",
                        ));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuPipelineLayout { rep: layout_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-pipeline-layout", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuPipelineLayout { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-pipeline-layout.label",
                |mut caller, (layout,): (Resource<GpuPipelineLayout>,)| {
                    let layout_rep = caller.data_mut().table.get(&layout)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if layout_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_pipeline_layout(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        layout_rep
                    };
                    let label = jvm::exp_pipeline_layout_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-pipeline-layout.set-label",
                |mut caller, (layout, label): (Resource<GpuPipelineLayout>, String)| {
                    let layout_rep = caller.data_mut().table.get(&layout)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if layout_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_pipeline_layout(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        layout_rep
                    };
                    jvm::exp_pipeline_layout_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-bind-group",
                ResourceType::host::<GpuBindGroup>(),
                |mut store, rep| {
                    let resource = Resource::<GpuBindGroup>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-bind-group",
                |mut caller, (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuBindGroupDescriptor,
                )| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let layout_rep = caller.data_mut().table.get(&descriptor.layout)?.rep;
                    let label = descriptor.label.clone().unwrap_or_default();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let l2_layout = if layout_rep == 0 {
                        jvm::exp_create_bind_group_layout(&cb, l2_device)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        layout_rep
                    };
                    let mut bindings = Vec::with_capacity(descriptor.entries.len());
                    let mut kinds = Vec::with_capacity(descriptor.entries.len());
                    let mut handles = Vec::with_capacity(descriptor.entries.len());
                    for entry in &descriptor.entries {
                        bindings.push(entry.binding as i32);
                        let (kind, raw) = match &entry.resource {
                            GpuBindingResource::GpuBuffer(buffer) => {
                                (0, caller.data_mut().table.get(buffer)?.rep)
                            }
                            GpuBindingResource::GpuBufferBinding(binding) => {
                                (0, caller.data_mut().table.get(&binding.buffer)?.rep)
                            }
                            GpuBindingResource::GpuSampler(sampler) => {
                                (1, caller.data_mut().table.get(sampler)?.rep)
                            }
                            GpuBindingResource::GpuTexture(texture) => {
                                (2, caller.data_mut().table.get(texture)?.rep)
                            }
                            GpuBindingResource::GpuTextureView(view) => {
                                (2, caller.data_mut().table.get(view)?.rep)
                            }
                        };
                        let handle = if raw != 0 {
                            raw
                        } else if kind == 0 {
                            jvm::exp_create_buffer(&cb, l2_device)
                                .map_err(wasmtime::Error::msg)?
                        } else {
                            raw
                        };
                        kinds.push(kind);
                        handles.push(handle as i32);
                    }
                    let bg_rep = jvm::exp_create_bind_group_described(
                        &cb, l2_device, l2_layout, label, bindings, kinds, handles,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if bg_rep == 0 {
                        return Err(wasmtime::Error::msg("device-create-bind-group returned 0"));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuBindGroup { rep: bg_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-bind-group", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuBindGroup { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-bind-group.label",
                |mut caller, (bind_group,): (Resource<GpuBindGroup>,)| {
                    let bind_group_rep = caller.data_mut().table.get(&bind_group)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if bind_group_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_bind_group(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        bind_group_rep
                    };
                    let label = jvm::exp_bind_group_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-bind-group.set-label",
                |mut caller, (bind_group, label): (Resource<GpuBindGroup>, String)| {
                    let bind_group_rep = caller.data_mut().table.get(&bind_group)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if bind_group_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_bind_group(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        bind_group_rep
                    };
                    jvm::exp_bind_group_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-render-pipeline",
                ResourceType::host::<GpuRenderPipeline>(),
                |mut store, rep| {
                    let resource = Resource::<GpuRenderPipeline>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-render-pipeline", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuRenderPipeline { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pipeline.label",
                |mut caller, (pipeline,): (Resource<GpuRenderPipeline>,)| {
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if pipeline_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_pipeline(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        pipeline_rep
                    };
                    let label = jvm::exp_render_pipeline_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pipeline.set-label",
                |mut caller, (pipeline, label): (Resource<GpuRenderPipeline>, String)| {
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if pipeline_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_pipeline(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        pipeline_rep
                    };
                    jvm::exp_render_pipeline_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pipeline.get-bind-group-layout",
                |mut caller, (pipeline, index): (Resource<GpuRenderPipeline>, u32)| {
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let layout_rep = jvm::exp_render_pipeline_get_bind_group_layout_described(
                        &cb,
                        pipeline_rep,
                        index,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuBindGroupLayout { rep: layout_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-compute-pipeline",
                ResourceType::host::<GpuComputePipeline>(),
                |mut store, rep| {
                    let resource = Resource::<GpuComputePipeline>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-compute-pipeline", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuComputePipeline { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pipeline.label",
                |mut caller, (pipeline,): (Resource<GpuComputePipeline>,)| {
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if pipeline_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_compute_pipeline(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        pipeline_rep
                    };
                    let label = jvm::exp_compute_pipeline_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pipeline.set-label",
                |mut caller, (pipeline, label): (Resource<GpuComputePipeline>, String)| {
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if pipeline_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_compute_pipeline(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        pipeline_rep
                    };
                    jvm::exp_compute_pipeline_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pipeline.get-bind-group-layout",
                |mut caller, (pipeline, index): (Resource<GpuComputePipeline>, u32)| {
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let layout_rep = jvm::exp_compute_pipeline_get_bind_group_layout_described(
                        &cb,
                        pipeline_rep,
                        index,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuBindGroupLayout { rep: layout_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-render-pipeline",
                |mut caller, (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuRenderPipelineDescriptor,
                )| {
                    let vertex_shader = caller
                        .data_mut()
                        .table
                        .get(&descriptor.vertex.module)?
                        .rep;
                    let vertex_entry = descriptor.vertex.entry_point.clone().unwrap_or_default();
                    let (fragment_shader, fragment_entry) = match &descriptor.fragment {
                        Some(fragment) => {
                            let fs = caller.data_mut().table.get(&fragment.module)?.rep as i32;
                            (fs, fragment.entry_point.clone().unwrap_or_default())
                        }
                        None => (0, String::new()),
                    };
                    let layout_rep = match &descriptor.layout {
                        GpuLayoutMode::Specific(layout) => {
                            caller.data_mut().table.get(layout)?.rep as i32
                        }
                        GpuLayoutMode::Auto => 0,
                    };
                    let format = first_fragment_target_format(&descriptor.fragment);
                    let (
                        vb_strides,
                        vb_step_modes,
                        attr_index,
                        attr_formats,
                        attr_offsets,
                        attr_locations,
                    ) = pack_vertex_buffers(&descriptor.vertex.buffers);
                    let label = descriptor.label.clone().unwrap_or_default();
                    let vertex_constants = pipeline_constant_rep(&descriptor.vertex.constants);
                    let fragment_constants = match &descriptor.fragment {
                        Some(fragment) => pipeline_constant_rep(&fragment.constants),
                        None => 0,
                    };
                    let (primitive, multisample, blend) =
                        pack_render_pipeline_semantics(&descriptor);
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let pipeline_rep = jvm::exp_create_render_pipeline_described(
                        &cb,
                        l2_device,
                        vertex_shader,
                        vertex_entry,
                        fragment_shader,
                        fragment_entry,
                        format,
                        layout_rep,
                        label,
                        vb_strides,
                        vb_step_modes,
                        attr_index,
                        attr_formats,
                        attr_offsets,
                        attr_locations,
                        vertex_constants,
                        fragment_constants,
                        primitive,
                        multisample,
                        blend,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if pipeline_rep == 0 {
                        return Err(wasmtime::Error::msg(
                            "device-create-render-pipeline returned 0",
                        ));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuRenderPipeline { rep: pipeline_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-compute-pipeline",
                |mut caller, (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuComputePipelineDescriptor,
                )| {
                    let shader_rep = caller
                        .data_mut()
                        .table
                        .get(&descriptor.compute.module)?
                        .rep;
                    let layout_rep = match &descriptor.layout {
                        GpuLayoutMode::Specific(layout) => {
                            caller.data_mut().table.get(layout)?.rep as i32
                        }
                        GpuLayoutMode::Auto => 0,
                    };
                    let entry_point = descriptor.compute.entry_point.clone().unwrap_or_default();
                    let label = descriptor.label.clone().unwrap_or_default();
                    let constants = pipeline_constant_rep(&descriptor.compute.constants);
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let pipeline_rep = jvm::exp_create_compute_pipeline_described(
                        &cb,
                        l2_device,
                        shader_rep,
                        entry_point,
                        layout_rep,
                        label,
                        constants,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if pipeline_rep == 0 {
                        return Err(wasmtime::Error::msg(
                            "device-create-compute-pipeline returned 0",
                        ));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuComputePipeline { rep: pipeline_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu-device.create-render-pipeline-async",
                |accessor, (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuRenderPipelineDescriptor,
                )| {
                    Box::pin(async move {
                        let (
                            cb,
                            device_rep,
                            vertex_shader,
                            vertex_entry,
                            fragment_shader,
                            fragment_entry,
                            format,
                            layout_rep,
                            label,
                            vb_strides,
                            vb_step_modes,
                            attr_index,
                            attr_formats,
                            attr_offsets,
                            attr_locations,
                            vertex_constants,
                            fragment_constants,
                            primitive,
                            multisample,
                            blend,
                        ) = accessor.with(|mut access| -> wasmtime::Result<_> {
                            let vertex_shader = access
                                .data_mut()
                                .table
                                .get(&descriptor.vertex.module)?
                                .rep;
                            let vertex_entry =
                                descriptor.vertex.entry_point.clone().unwrap_or_default();
                            let (fragment_shader, fragment_entry) = match &descriptor.fragment {
                                Some(fragment) => {
                                    let fs =
                                        access.data_mut().table.get(&fragment.module)?.rep as i32;
                                    (fs, fragment.entry_point.clone().unwrap_or_default())
                                }
                                None => (0, String::new()),
                            };
                            let layout_rep = match &descriptor.layout {
                                GpuLayoutMode::Specific(layout) => {
                                    access.data_mut().table.get(layout)?.rep as i32
                                }
                                GpuLayoutMode::Auto => 0,
                            };
                            let format = first_fragment_target_format(&descriptor.fragment);
                            let packed = pack_vertex_buffers(&descriptor.vertex.buffers);
                            let label = descriptor.label.clone().unwrap_or_default();
                            let vertex_constants =
                                pipeline_constant_rep(&descriptor.vertex.constants);
                            let fragment_constants = match &descriptor.fragment {
                                Some(fragment) => pipeline_constant_rep(&fragment.constants),
                                None => 0,
                            };
                            let (primitive, multisample, blend) =
                                pack_render_pipeline_semantics(&descriptor);
                            let device_rep = access.data_mut().table.get(&device)?.rep;
                            let cb = access
                                .data_mut()
                                .experimental_host_cb
                                .as_ref()
                                .ok_or_else(|| {
                                    wasmtime::Error::msg("experimental host callback not set")
                                })
                                .cloned()?;
                            Ok((
                                cb,
                                device_rep,
                                vertex_shader,
                                vertex_entry,
                                fragment_shader,
                                fragment_entry,
                                format,
                                layout_rep,
                                label,
                                packed.0,
                                packed.1,
                                packed.2,
                                packed.3,
                                packed.4,
                                packed.5,
                                vertex_constants,
                                fragment_constants,
                                primitive,
                                multisample,
                                blend,
                            ))
                        })?;
                        let (tx, rx) = oneshot::channel::<()>();
                        std::thread::spawn(move || {
                            let _ = tx.send(());
                        });
                        let _ = rx.await;
                        let l2_device = if device_rep == 0 {
                            let adapter_rep =
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                            jvm::exp_adapter_request_device(&cb, adapter_rep)
                                .map_err(wasmtime::Error::msg)?
                        } else {
                            device_rep
                        };
                        let pipeline_rep = jvm::exp_create_render_pipeline_described(
                            &cb,
                            l2_device,
                            vertex_shader,
                            vertex_entry,
                            fragment_shader,
                            fragment_entry,
                            format,
                            layout_rep,
                            label,
                            vb_strides,
                            vb_step_modes,
                            attr_index,
                            attr_formats,
                            attr_offsets,
                            attr_locations,
                            vertex_constants,
                            fragment_constants,
                            primitive,
                            multisample,
                            blend,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        if pipeline_rep == 0 {
                            return Ok((Err(CreatePipelineError {
                                kind: CreatePipelineErrorKind::GpuPipelineError(
                                    GpuPipelineErrorReason::Internal,
                                ),
                                message: "device-create-render-pipeline returned 0".into(),
                            }),));
                        }
                        let resource = accessor.with(|mut access| {
                            access
                                .data_mut()
                                .table
                                .push(GpuRenderPipeline { rep: pipeline_rep })
                        })?;
                        Ok((Ok(resource),))
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu-device.create-compute-pipeline-async",
                |accessor, (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuComputePipelineDescriptor,
                )| {
                    Box::pin(async move {
                        let (cb, device_rep, shader_rep, entry_point, layout_rep, label, constants) =
                            accessor.with(|mut access| -> wasmtime::Result<_> {
                                let shader_rep = access
                                    .data_mut()
                                    .table
                                    .get(&descriptor.compute.module)?
                                    .rep;
                                let layout_rep = match &descriptor.layout {
                                    GpuLayoutMode::Specific(layout) => {
                                        access.data_mut().table.get(layout)?.rep as i32
                                    }
                                    GpuLayoutMode::Auto => 0,
                                };
                                let entry_point =
                                    descriptor.compute.entry_point.clone().unwrap_or_default();
                                let label = descriptor.label.clone().unwrap_or_default();
                                let constants =
                                    pipeline_constant_rep(&descriptor.compute.constants);
                                let device_rep = access.data_mut().table.get(&device)?.rep;
                                let cb = access
                                    .data_mut()
                                    .experimental_host_cb
                                    .as_ref()
                                    .ok_or_else(|| {
                                        wasmtime::Error::msg("experimental host callback not set")
                                    })
                                    .cloned()?;
                                Ok((
                                    cb,
                                    device_rep,
                                    shader_rep,
                                    entry_point,
                                    layout_rep,
                                    label,
                                    constants,
                                ))
                            })?;
                        let (tx, rx) = oneshot::channel::<()>();
                        std::thread::spawn(move || {
                            let _ = tx.send(());
                        });
                        let _ = rx.await;
                        let l2_device = if device_rep == 0 {
                            let adapter_rep =
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                            jvm::exp_adapter_request_device(&cb, adapter_rep)
                                .map_err(wasmtime::Error::msg)?
                        } else {
                            device_rep
                        };
                        let pipeline_rep = jvm::exp_create_compute_pipeline_described(
                            &cb,
                            l2_device,
                            shader_rep,
                            entry_point,
                            layout_rep,
                            label,
                            constants,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        if pipeline_rep == 0 {
                            return Ok((Err(CreatePipelineError {
                                kind: CreatePipelineErrorKind::GpuPipelineError(
                                    GpuPipelineErrorReason::Internal,
                                ),
                                message: "device-create-compute-pipeline returned 0".into(),
                            }),));
                        }
                        let resource = accessor.with(|mut access| {
                            access
                                .data_mut()
                                .table
                                .push(GpuComputePipeline { rep: pipeline_rep })
                        })?;
                        Ok((Ok(resource),))
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-encoder", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuCommandEncoder { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.label",
                |mut caller, (encoder,): (Resource<GpuCommandEncoder>,)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    let label = jvm::exp_command_encoder_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.set-label",
                |mut caller, (encoder, label): (Resource<GpuCommandEncoder>, String)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_command_encoder_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-query-set",
                ResourceType::host::<GpuQuerySet>(),
                |mut store, rep| {
                    let resource = Resource::<GpuQuerySet>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-query-set", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuQuerySet { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-query-set",
                |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuQuerySetDescriptor)| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let query_rep = jvm::exp_create_query_set_described(
                        &cb,
                        l2_device,
                        descriptor.type_.to_host_u32(),
                        descriptor.count,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuQuerySet { rep: query_rep })?;
                    Ok((Ok::<_, CreateQuerySetError>(resource),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-query-set.destroy",
                |mut caller, (query_set,): (Resource<GpuQuerySet>,)| {
                    let query_rep = caller.data_mut().table.get(&query_set)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_query = if query_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_query_set(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        query_rep
                    };
                    jvm::exp_query_set_destroy_described(&cb, l2_query)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-query-set.type",
                |mut caller, (query_set,): (Resource<GpuQuerySet>,)| {
                    let query_rep = caller.data_mut().table.get(&query_set)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_query = if query_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_query_set(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        query_rep
                    };
                    let ty = jvm::exp_query_set_type_described(&cb, l2_query)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuQueryType::from_host_u32(ty),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-query-set.count",
                |mut caller, (query_set,): (Resource<GpuQuerySet>,)| {
                    let query_rep = caller.data_mut().table.get(&query_set)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_query = if query_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_query_set(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        query_rep
                    };
                    let count = jvm::exp_query_set_count_described(&cb, l2_query)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((count,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-query-set.label",
                |mut caller, (query_set,): (Resource<GpuQuerySet>,)| {
                    let query_set_rep = caller.data_mut().table.get(&query_set)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if query_set_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_query_set(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        query_set_rep
                    };
                    let label = jvm::exp_query_set_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-query-set.set-label",
                |mut caller, (query_set, label): (Resource<GpuQuerySet>, String)| {
                    let query_set_rep = caller.data_mut().table.get(&query_set)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if query_set_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_query_set(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        query_set_rep
                    };
                    jvm::exp_query_set_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-render-pass-encoder",
                ResourceType::host::<GpuRenderPassEncoder>(),
                |mut store, rep| {
                    let resource = Resource::<GpuRenderPassEncoder>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.begin-render-pass",
                |mut caller, (encoder, descriptor): (Resource<GpuCommandEncoder>, GpuRenderPassDescriptor)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let mut color_views = Vec::new();
                    let mut color_loads = Vec::new();
                    let mut color_stores = Vec::new();
                    let mut color_has_clears = Vec::new();
                    let mut color_clear_bits = Vec::new();
                    for att in &descriptor.color_attachments {
                        match att {
                            None => {
                                color_views.push(0);
                                color_loads.push(0);
                                color_stores.push(0);
                                color_has_clears.push(0);
                                color_clear_bits.extend_from_slice(&[0, 0, 0, 0]);
                            }
                            Some(att) => {
                                let view_rep = caller.data_mut().table.get(&att.view)?.rep as i32;
                                color_views.push(view_rep);
                                color_loads.push(att.load_op.to_dawn_u32() as i32);
                                color_stores.push(att.store_op.to_dawn_u32() as i32);
                                match &att.clear_value {
                                    Some(c) => {
                                        color_has_clears.push(1);
                                        color_clear_bits.extend_from_slice(&pack_color_clear_bits(c));
                                    }
                                    None => {
                                        color_has_clears.push(0);
                                        color_clear_bits.extend_from_slice(&[0, 0, 0, 0]);
                                    }
                                }
                            }
                        }
                    }
                    let (depth_view, depth_load, depth_store, has_depth_clear, depth_clear) =
                        match &descriptor.depth_stencil_attachment {
                            Some(ds) => {
                                let view = caller.data_mut().table.get(&ds.view)?.rep;
                                let load = ds
                                    .depth_load_op
                                    .map(|op| op.to_dawn_u32() as i32)
                                    .unwrap_or(-1);
                                let store = ds
                                    .depth_store_op
                                    .map(|op| op.to_dawn_u32() as i32)
                                    .unwrap_or(-1);
                                let (has, v) = match ds.depth_clear_value {
                                    Some(c) => (1, c),
                                    None => (0, 1.0),
                                };
                                (view, load, store, has, v)
                            }
                            None => (0, -1, -1, 0, 1.0),
                        };
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    let pass_rep = jvm::exp_begin_render_pass_described(
                        &cb,
                        l2_encoder,
                        color_views,
                        color_loads,
                        color_stores,
                        color_has_clears,
                        color_clear_bits,
                        depth_view,
                        depth_load,
                        depth_store,
                        has_depth_clear,
                        depth_clear,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    if pass_rep == 0 {
                        return Err(wasmtime::Error::msg("begin-render-pass returned 0"));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuRenderPassEncoder { rep: pass_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-compute-pass-encoder",
                ResourceType::host::<GpuComputePassEncoder>(),
                |mut store, rep| {
                    let resource = Resource::<GpuComputePassEncoder>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.begin-compute-pass",
                |mut caller,
                 (encoder, descriptor): (
                    Resource<GpuCommandEncoder>,
                    Option<GpuComputePassDescriptor>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let (begin_idx, end_idx) = match descriptor
                        .as_ref()
                        .and_then(|d| d.timestamp_writes.as_ref())
                    {
                        Some(ts) => {
                            let begin_idx = ts.beginning_of_pass_write_index.unwrap_or(0);
                            let end_idx = ts.end_of_pass_write_index.unwrap_or(0);
                            let _ = caller.data_mut().table.get(&ts.query_set)?;
                            (begin_idx, end_idx)
                        }
                        None => (0, 0),
                    };
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    let pass_rep =
                        jvm::exp_begin_compute_pass_described(&cb, l2_encoder, begin_idx, end_idx)
                            .map_err(wasmtime::Error::msg)?;
                    if pass_rep == 0 {
                        return Err(wasmtime::Error::msg("begin-compute-pass returned 0"));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuComputePassEncoder { rep: pass_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.copy-buffer-to-buffer",
                |mut caller,
                 (encoder, source, source_offset, destination, destination_offset, size): (
                    Resource<GpuCommandEncoder>,
                    Resource<GpuBuffer>,
                    Option<u64>,
                    Resource<GpuBuffer>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let source_rep = caller.data_mut().table.get(&source)?.rep;
                    let dest_rep = caller.data_mut().table.get(&destination)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_copy_buffer_to_buffer_described(
                        &cb,
                        l2_encoder,
                        source_rep,
                        source_offset.unwrap_or(0),
                        dest_rep,
                        destination_offset.unwrap_or(0),
                        size.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.copy-buffer-to-texture",
                |mut caller,
                 (encoder, source, destination, copy_size): (
                    Resource<GpuCommandEncoder>,
                    GpuTexelCopyBufferInfo,
                    GpuTexelCopyTextureInfo,
                    GpuExtent3D,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let source_rep = caller.data_mut().table.get(&source.buffer)?.rep;
                    let dest_rep = caller.data_mut().table.get(&destination.texture)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_copy_buffer_to_texture_described(
                        &cb,
                        l2_encoder,
                        source_rep,
                        dest_rep,
                        copy_size.width,
                        copy_size.height.unwrap_or(1),
                        copy_size.depth_or_array_layers.unwrap_or(1),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.copy-texture-to-buffer",
                |mut caller,
                 (encoder, source, destination, copy_size): (
                    Resource<GpuCommandEncoder>,
                    GpuTexelCopyTextureInfo,
                    GpuTexelCopyBufferInfo,
                    GpuExtent3D,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let source_rep = caller.data_mut().table.get(&source.texture)?.rep;
                    let dest_rep = caller.data_mut().table.get(&destination.buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_copy_texture_to_buffer_described(
                        &cb,
                        l2_encoder,
                        source_rep,
                        dest_rep,
                        copy_size.width,
                        copy_size.height.unwrap_or(1),
                        copy_size.depth_or_array_layers.unwrap_or(1),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.copy-texture-to-texture",
                |mut caller,
                 (encoder, source, destination, copy_size): (
                    Resource<GpuCommandEncoder>,
                    GpuTexelCopyTextureInfo,
                    GpuTexelCopyTextureInfo,
                    GpuExtent3D,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let source_rep = caller.data_mut().table.get(&source.texture)?.rep;
                    let dest_rep = caller.data_mut().table.get(&destination.texture)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_copy_texture_to_texture_described(
                        &cb,
                        l2_encoder,
                        source_rep,
                        dest_rep,
                        copy_size.width,
                        copy_size.height.unwrap_or(1),
                        copy_size.depth_or_array_layers.unwrap_or(1),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.clear-buffer",
                |mut caller,
                 (encoder, buffer, offset, size): (
                    Resource<GpuCommandEncoder>,
                    Resource<GpuBuffer>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_clear_buffer_described(
                        &cb,
                        l2_encoder,
                        buffer_rep,
                        offset.unwrap_or(0),
                        size.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.resolve-query-set",
                |mut caller,
                 (
                    encoder,
                    query_set,
                    first_query,
                    query_count,
                    destination,
                    destination_offset,
                ): (
                    Resource<GpuCommandEncoder>,
                    Resource<GpuQuerySet>,
                    u32,
                    u32,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let query_rep = caller.data_mut().table.get(&query_set)?.rep;
                    let dest_rep = caller.data_mut().table.get(&destination)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_resolve_query_set_described(
                        &cb,
                        l2_encoder,
                        query_rep,
                        first_query,
                        query_count,
                        dest_rep,
                        destination_offset,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.push-debug-group",
                |mut caller, (encoder, group_label): (Resource<GpuCommandEncoder>, String)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_push_debug_group_described(&cb, l2_encoder, group_label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.pop-debug-group",
                |mut caller, (encoder,): (Resource<GpuCommandEncoder>,)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_pop_debug_group_described(&cb, l2_encoder)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.insert-debug-marker",
                |mut caller, (encoder, marker_label): (Resource<GpuCommandEncoder>, String)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_insert_debug_marker_described(&cb, l2_encoder, marker_label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-command-buffer",
                ResourceType::host::<GpuCommandBuffer>(),
                |mut store, rep| {
                    let resource = Resource::<GpuCommandBuffer>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.finish",
                |mut caller,
                 (encoder, descriptor): (
                    Resource<GpuCommandEncoder>,
                    Option<GpuCommandBufferDescriptor>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let label = descriptor
                        .as_ref()
                        .and_then(|d| d.label.clone())
                        .unwrap_or_default();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    let buffer_rep =
                        jvm::exp_command_encoder_finish_described(&cb, l2_encoder, label)
                            .map_err(wasmtime::Error::msg)?;
                    if buffer_rep == 0 {
                        return Err(wasmtime::Error::msg("command-encoder-finish returned 0"));
                    }
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuCommandBuffer { rep: buffer_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-queue", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuQueue { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-command-buffer", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuCommandBuffer { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-buffer.label",
                |mut caller, (buffer,): (Resource<GpuCommandBuffer>,)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_command_encoder_finish(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    let label = jvm::exp_command_buffer_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-buffer.set-label",
                |mut caller, (buffer, label): (Resource<GpuCommandBuffer>, String)| {
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if buffer_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_command_encoder_finish(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    jvm::exp_command_buffer_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-compilation-message",
                ResourceType::host::<GpuCompilationMessage>(),
                |mut store, rep| {
                    let resource = Resource::<GpuCompilationMessage>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-compilation-info",
                ResourceType::host::<GpuCompilationInfo>(),
                |mut store, rep| {
                    let resource = Resource::<GpuCompilationInfo>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-compilation-info", |mut store, ()| {
                let resource = store
                    .data_mut()
                    .table
                    .push(GpuCompilationInfo {
                        shader_module: 0,
                    })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-compilation-message", |mut store, ()| {
                let resource = store
                    .data_mut()
                    .table
                    .push(GpuCompilationMessage {
                        shader_module: 0,
                    })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-info.messages",
                |mut caller, (info,): (Resource<GpuCompilationInfo>,)| {
                    let info_shader = caller.data_mut().table.get(&info)?.shader_module;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_shader = if info_shader == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        info_shader
                    };
                    let count =
                        jvm::exp_compilation_info_messages_count_described(&cb, l2_shader)
                            .map_err(wasmtime::Error::msg)?;
                    let mut messages = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        messages.push(
                            caller
                                .data_mut()
                                .table
                                .push(GpuCompilationMessage {
                                    shader_module: l2_shader,
                                })?,
                        );
                    }
                    Ok((messages,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.message",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let msg_shader = caller.data_mut().table.get(&msg)?.shader_module;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_shader = if msg_shader == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        msg_shader
                    };
                    let message =
                        jvm::exp_compilation_message_message_described(&cb, l2_shader)
                            .map_err(wasmtime::Error::msg)?;
                    Ok((message,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.type",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let msg_shader = caller.data_mut().table.get(&msg)?.shader_module;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_shader = if msg_shader == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        msg_shader
                    };
                    let ty = jvm::exp_compilation_message_type_described(&cb, l2_shader)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((GpuCompilationMessageType::from_host_u32(ty),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.line-num",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let msg_shader = caller.data_mut().table.get(&msg)?.shader_module;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_shader = if msg_shader == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        msg_shader
                    };
                    let line_num =
                        jvm::exp_compilation_message_line_num_described(&cb, l2_shader)
                            .map_err(wasmtime::Error::msg)?;
                    Ok((line_num,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.line-pos",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let msg_shader = caller.data_mut().table.get(&msg)?.shader_module;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_shader = if msg_shader == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        msg_shader
                    };
                    let line_pos =
                        jvm::exp_compilation_message_line_pos_described(&cb, l2_shader)
                            .map_err(wasmtime::Error::msg)?;
                    Ok((line_pos,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.offset",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let msg_shader = caller.data_mut().table.get(&msg)?.shader_module;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_shader = if msg_shader == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        msg_shader
                    };
                    let offset =
                        jvm::exp_compilation_message_offset_described(&cb, l2_shader)
                            .map_err(wasmtime::Error::msg)?;
                    Ok((offset,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.length",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let msg_shader = caller.data_mut().table.get(&msg)?.shader_module;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_shader = if msg_shader == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_shader_module_described(
                            &cb,
                            device_rep,
                            "@compute @workgroup_size(1) fn main() {}".to_string(),
                            String::new(),
                            Vec::new(),
                            String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        msg_shader
                    };
                    let length =
                        jvm::exp_compilation_message_length_described(&cb, l2_shader)
                            .map_err(wasmtime::Error::msg)?;
                    Ok((length,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-queue.label",
                |mut caller, (queue,): (Resource<GpuQueue>,)| {
                    let queue_rep = caller.data_mut().table.get(&queue)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if queue_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_device_get_queue(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        queue_rep
                    };
                    let label = jvm::exp_queue_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-queue.set-label",
                |mut caller, (queue, label): (Resource<GpuQueue>, String)| {
                    let queue_rep = caller.data_mut().table.get(&queue)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if queue_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_device_get_queue(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        queue_rep
                    };
                    jvm::exp_queue_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu-queue.on-submitted-work-done",
                |accessor, (queue,): (Resource<GpuQueue>,)| {
                    Box::pin(async move {
                        let (cb, queue_rep) =
                            accessor.with(|mut access| -> wasmtime::Result<_> {
                                let queue_rep = access.data_mut().table.get(&queue)?.rep;
                                let cb = access
                                    .data_mut()
                                    .experimental_host_cb
                                    .as_ref()
                                    .ok_or_else(|| {
                                        wasmtime::Error::msg("experimental host callback not set")
                                    })
                                    .cloned()?;
                                Ok((cb, queue_rep))
                            })?;
                        let l2_queue = if queue_rep == 0 {
                            let adapter_rep =
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                            let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                                .map_err(wasmtime::Error::msg)?;
                            jvm::exp_device_get_queue(&cb, device_rep)
                                .map_err(wasmtime::Error::msg)?
                        } else {
                            queue_rep
                        };
                        jvm::exp_queue_on_submitted_work_done_described(&cb, l2_queue)
                            .map_err(wasmtime::Error::msg)?;
                        Ok(())
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-queue.submit",
                |mut caller, (queue, commands): (
                    Resource<GpuQueue>,
                    Vec<Resource<GpuCommandBuffer>>,
                )| {
                    let queue_rep = caller.data_mut().table.get(&queue)?.rep;
                    let mut command_reps = Vec::with_capacity(commands.len());
                    for command in &commands {
                        command_reps.push(caller.data_mut().table.get(command)?.rep);
                    }
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let mut device_rep = 0u32;
                    let l2_queue = if queue_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_device_get_queue(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        queue_rep
                    };
                    let mut l2_commands = Vec::with_capacity(command_reps.len());
                    for command_rep in command_reps {
                        if command_rep != 0 {
                            l2_commands.push(command_rep as i32);
                            continue;
                        }
                        if device_rep == 0 {
                            let adapter_rep =
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                            device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                                .map_err(wasmtime::Error::msg)?;
                        }
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let finished = jvm::exp_command_encoder_finish(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?;
                        l2_commands.push(finished as i32);
                    }
                    jvm::exp_queue_submit_described(&cb, l2_queue, l2_commands)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-queue.write-buffer-with-copy",
                |mut caller,
                 (queue, buffer, offset, data, data_offset, size): (
                    Resource<GpuQueue>,
                    Resource<GpuBuffer>,
                    u64,
                    Vec<u8>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let queue_rep = caller.data_mut().table.get(&queue)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let start = data_offset.unwrap_or(0) as usize;
                    let copy_len = size
                        .map(|s| s as usize)
                        .unwrap_or_else(|| data.len().saturating_sub(start));
                    let payload = if start >= data.len() {
                        Vec::new()
                    } else {
                        let end = (start + copy_len).min(data.len());
                        data[start..end].to_vec()
                    };
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let mut device_rep = 0u32;
                    let l2_queue = if queue_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_device_get_queue(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        queue_rep
                    };
                    let l2_buffer = if buffer_rep == 0 {
                        if device_rep == 0 {
                            let adapter_rep =
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                            device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                                .map_err(wasmtime::Error::msg)?;
                        }
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        buffer_rep
                    };
                    jvm::exp_queue_write_buffer_described(
                        &cb, l2_queue, l2_buffer, offset, payload,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((Ok::<(), WriteBufferError>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-queue.write-texture-with-copy",
                |mut caller,
                 (queue, destination, data, layout, size): (
                    Resource<GpuQueue>,
                    GpuTexelCopyTextureInfo,
                    Vec<u8>,
                    GpuTexelCopyBufferLayout,
                    GpuExtent3D,
                )| {
                    let queue_rep = caller.data_mut().table.get(&queue)?.rep;
                    let texture_rep = caller.data_mut().table.get(&destination.texture)?.rep;
                    let start = layout.offset.unwrap_or(0) as usize;
                    let payload = if start >= data.len() {
                        Vec::new()
                    } else {
                        data[start..].to_vec()
                    };
                    let width = size.width.max(1);
                    let height = size.height.unwrap_or(1).max(1);
                    let bytes_per_row = layout
                        .bytes_per_row
                        .unwrap_or(width.saturating_mul(4))
                        .max(1);
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let mut device_rep = 0u32;
                    let l2_queue = if queue_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_device_get_queue(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        queue_rep
                    };
                    let l2_texture = if texture_rep == 0 {
                        if device_rep == 0 {
                            let adapter_rep =
                                jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                            device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                                .map_err(wasmtime::Error::msg)?;
                        }
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?
                    } else {
                        texture_rep
                    };
                    jvm::exp_queue_write_texture_described(
                        &cb,
                        l2_queue,
                        l2_texture,
                        payload,
                        width,
                        height,
                        bytes_per_row,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-pass", |mut store, ()| {
                let resource = store
                    .data_mut()
                    .table
                    .push(GpuRenderPassEncoder { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.end",
                |mut caller, (pass,): (Resource<GpuRenderPassEncoder>,)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_end_described(&cb, l2_pass)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-pipeline",
                |mut caller,
                 (pass, pipeline): (
                    Resource<GpuRenderPassEncoder>,
                    Resource<GpuRenderPipeline>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_set_pipeline_described(&cb, l2_pass, pipeline_rep)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.draw",
                |mut caller,
                 (pass, vertex_count, instance_count, first_vertex, first_instance): (
                    Resource<GpuRenderPassEncoder>,
                    u32,
                    Option<u32>,
                    Option<u32>,
                    Option<u32>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_draw_described(
                        &cb,
                        l2_pass,
                        vertex_count,
                        instance_count.unwrap_or(1),
                        first_vertex.unwrap_or(0),
                        first_instance.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-bind-group",
                |mut caller,
                 (pass, index, bind_group, _offsets, _start, _length): (
                    Resource<GpuRenderPassEncoder>,
                    u32,
                    Option<Resource<GpuBindGroup>>,
                    Option<Vec<u32>>,
                    Option<u64>,
                    Option<u32>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let bind_group_rep = match bind_group {
                        Some(ref g) => caller.data_mut().table.get(g)?.rep,
                        None => 0,
                    };
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_set_bind_group_described(
                        &cb,
                        l2_pass,
                        index,
                        bind_group_rep,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((Ok::<(), SetBindGroupError>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-vertex-buffer",
                |mut caller,
                 (pass, slot, buffer, offset, size): (
                    Resource<GpuRenderPassEncoder>,
                    u32,
                    Option<Resource<GpuBuffer>>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let buffer_rep = match buffer {
                        Some(ref b) => caller.data_mut().table.get(b)?.rep,
                        None => 0,
                    };
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_set_vertex_buffer_described(
                        &cb,
                        l2_pass,
                        slot,
                        buffer_rep,
                        offset.unwrap_or(0),
                        size.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-viewport",
                |mut caller,
                 (pass, x, y, width, height, min_depth, max_depth): (
                    Resource<GpuRenderPassEncoder>,
                    f32,
                    f32,
                    f32,
                    f32,
                    f32,
                    f32,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_set_viewport_described(
                        &cb, l2_pass, x, y, width, height, min_depth, max_depth,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-scissor-rect",
                |mut caller,
                 (pass, x, y, width, height): (
                    Resource<GpuRenderPassEncoder>,
                    u32,
                    u32,
                    u32,
                    u32,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_set_scissor_rect_described(
                        &cb, l2_pass, x, y, width, height,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-blend-constant",
                |mut caller, (pass, color): (Resource<GpuRenderPassEncoder>, GpuColor)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_set_blend_constant_described(
                        &cb, l2_pass, color.r, color.g, color.b, color.a,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-stencil-reference",
                |mut caller, (pass, reference): (Resource<GpuRenderPassEncoder>, u32)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_set_stencil_reference_described(&cb, l2_pass, reference)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-index-buffer",
                |mut caller,
                 (pass, buffer, format, offset, size): (
                    Resource<GpuRenderPassEncoder>,
                    Resource<GpuBuffer>,
                    GpuIndexFormat,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let format_u32 = match format {
                        GpuIndexFormat::Uint16 => 1,
                        GpuIndexFormat::Uint32 => 2,
                    };
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_set_index_buffer_described(
                        &cb,
                        l2_pass,
                        buffer_rep,
                        format_u32,
                        offset.unwrap_or(0),
                        size.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.draw-indexed",
                |mut caller,
                 (
                    pass,
                    index_count,
                    instance_count,
                    first_index,
                    base_vertex,
                    first_instance,
                ): (
                    Resource<GpuRenderPassEncoder>,
                    u32,
                    Option<u32>,
                    Option<u32>,
                    Option<i32>,
                    Option<u32>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_draw_indexed_described(
                        &cb,
                        l2_pass,
                        index_count,
                        instance_count.unwrap_or(1),
                        first_index.unwrap_or(0),
                        base_vertex.unwrap_or(0),
                        first_instance.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.draw-indirect",
                |mut caller,
                 (pass, buffer, offset): (
                    Resource<GpuRenderPassEncoder>,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_draw_indirect_described(&cb, l2_pass, buffer_rep, offset)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.draw-indexed-indirect",
                |mut caller,
                 (pass, buffer, offset): (
                    Resource<GpuRenderPassEncoder>,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_draw_indexed_indirect_described(
                        &cb, l2_pass, buffer_rep, offset,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.push-debug-group",
                |mut caller, (pass, group_label): (Resource<GpuRenderPassEncoder>, String)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_push_debug_group_described(&cb, l2_pass, group_label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.pop-debug-group",
                |mut caller, (pass,): (Resource<GpuRenderPassEncoder>,)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_pop_debug_group_described(&cb, l2_pass)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.insert-debug-marker",
                |mut caller, (pass, marker_label): (Resource<GpuRenderPassEncoder>, String)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_insert_debug_marker_described(&cb, l2_pass, marker_label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.begin-occlusion-query",
                |mut caller, (pass, query_index): (Resource<GpuRenderPassEncoder>, u32)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_begin_occlusion_query_described(&cb, l2_pass, query_index)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.end-occlusion-query",
                |mut caller, (pass,): (Resource<GpuRenderPassEncoder>,)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_end_occlusion_query_described(&cb, l2_pass)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.label",
                |mut caller, (pass,): (Resource<GpuRenderPassEncoder>,)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let texture_rep = jvm::exp_create_texture(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let view_rep = jvm::exp_texture_create_view_described(
                            &cb, texture_rep, 0, 0, 0, 0, -1, 0, -1,
                        )
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, view_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    let label = jvm::exp_render_pass_encoder_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-label",
                |mut caller, (pass, label): (Resource<GpuRenderPassEncoder>, String)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let texture_rep = jvm::exp_create_texture(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let view_rep = jvm::exp_texture_create_view_described(
                            &cb, texture_rep, 0, 0, 0, 0, -1, 0, -1,
                        )
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, view_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_encoder_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-render-bundle",
                ResourceType::host::<GpuRenderBundle>(),
                |mut store, rep| {
                    let resource = Resource::<GpuRenderBundle>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-render-bundle", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuRenderBundle { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle.label",
                |mut caller, (bundle,): (Resource<GpuRenderBundle>,)| {
                    let bundle_rep = caller.data_mut().table.get(&bundle)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if bundle_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_render_bundle_encoder_described(
                            &cb, device_rep, 0x16, 1,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        jvm::exp_render_bundle_encoder_finish_described(
                            &cb, encoder_rep, String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        bundle_rep
                    };
                    let label = jvm::exp_render_bundle_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle.set-label",
                |mut caller, (bundle, label): (Resource<GpuRenderBundle>, String)| {
                    let bundle_rep = caller.data_mut().table.get(&bundle)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if bundle_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_render_bundle_encoder_described(
                            &cb, device_rep, 0x16, 1,
                        )
                        .map_err(wasmtime::Error::msg)?;
                        jvm::exp_render_bundle_encoder_finish_described(
                            &cb, encoder_rep, String::new(),
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        bundle_rep
                    };
                    jvm::exp_render_bundle_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.execute-bundles",
                |mut caller,
                 (pass, bundles): (
                    Resource<GpuRenderPassEncoder>,
                    Vec<Resource<GpuRenderBundle>>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let mut bundle_reps: Vec<i32> = Vec::with_capacity(bundles.len());
                    for bundle in &bundles {
                        bundle_reps.push(caller.data_mut().table.get(bundle)?.rep as i32);
                    }
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_render_pass_execute_bundles_described(&cb, l2_pass, bundle_reps)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-immediates",
                |mut caller,
                 (pass, range_offset, data, data_offset, data_size): (
                    Resource<GpuRenderPassEncoder>,
                    u32,
                    Vec<u8>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    let _ = data_size;
                    jvm::exp_render_pass_set_immediates_described(
                        &cb,
                        l2_pass,
                        range_offset,
                        data,
                        data_offset.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .resource(
                "gpu-render-bundle-encoder",
                ResourceType::host::<GpuRenderBundleEncoder>(),
                |mut store, rep| {
                    let resource = Resource::<GpuRenderBundleEncoder>::new_own(rep);
                    store.data_mut().table.delete(resource)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-render-bundle-encoder", |mut store, ()| {
                let resource = store
                    .data_mut()
                    .table
                    .push(GpuRenderBundleEncoder { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.label",
                |mut caller, (encoder,): (Resource<GpuRenderBundleEncoder>,)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(&cb, device_rep, 0x16, 1)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    let label = jvm::exp_render_bundle_encoder_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-label",
                |mut caller, (encoder, label): (Resource<GpuRenderBundleEncoder>, String)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(&cb, device_rep, 0x16, 1)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-render-bundle-encoder",
                |mut caller,
                 (device, descriptor): (
                    Resource<GpuDevice>,
                    GpuRenderBundleEncoderDescriptor,
                )| {
                    let device_rep = caller.data_mut().table.get(&device)?.rep;
                    let color_format = descriptor
                        .color_formats
                        .iter()
                        .flatten()
                        .next()
                        .map(|f| f.to_dawn_u32())
                        .unwrap_or_else(|| GpuTextureFormat::Rgba8unorm.to_dawn_u32());
                    let sample_count = descriptor.sample_count.unwrap_or(1);
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_device = if device_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        device_rep
                    };
                    let encoder_rep = jvm::exp_create_render_bundle_encoder_described(
                        &cb,
                        l2_device,
                        color_format,
                        sample_count,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuRenderBundleEncoder { rep: encoder_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.finish",
                |mut caller,
                 (encoder, descriptor): (
                    Resource<GpuRenderBundleEncoder>,
                    Option<GpuRenderBundleDescriptor>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let label = descriptor
                        .as_ref()
                        .and_then(|d| d.label.clone())
                        .unwrap_or_default();
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    let bundle_rep =
                        jvm::exp_render_bundle_encoder_finish_described(&cb, l2_encoder, label)
                            .map_err(wasmtime::Error::msg)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuRenderBundle { rep: bundle_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-pipeline",
                |mut caller,
                 (encoder, pipeline): (
                    Resource<GpuRenderBundleEncoder>,
                    Resource<GpuRenderPipeline>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_set_pipeline_described(
                        &cb,
                        l2_encoder,
                        pipeline_rep,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-bind-group",
                |mut caller,
                 (encoder, index, bind_group, _offsets, _start, _length): (
                    Resource<GpuRenderBundleEncoder>,
                    u32,
                    Option<Resource<GpuBindGroup>>,
                    Option<Vec<u32>>,
                    Option<u64>,
                    Option<u32>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let bind_group_rep = match bind_group.as_ref() {
                        Some(bind_group) => caller.data_mut().table.get(bind_group)?.rep,
                        None => 0,
                    };
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_set_bind_group_described(
                        &cb,
                        l2_encoder,
                        index,
                        bind_group_rep,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((Ok::<(), SetBindGroupError>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.draw",
                |mut caller,
                 (encoder, vertex_count, instance_count, first_vertex, first_instance): (
                    Resource<GpuRenderBundleEncoder>,
                    u32,
                    Option<u32>,
                    Option<u32>,
                    Option<u32>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_draw_described(
                        &cb,
                        l2_encoder,
                        vertex_count,
                        instance_count.unwrap_or(1),
                        first_vertex.unwrap_or(0),
                        first_instance.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-index-buffer",
                |mut caller,
                 (encoder, buffer, format, offset, size): (
                    Resource<GpuRenderBundleEncoder>,
                    Resource<GpuBuffer>,
                    GpuIndexFormat,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_set_index_buffer_described(
                        &cb,
                        l2_encoder,
                        buffer_rep,
                        match format {
                            GpuIndexFormat::Uint16 => 1,
                            GpuIndexFormat::Uint32 => 2,
                        },
                        offset.unwrap_or(0),
                        size.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-vertex-buffer",
                |mut caller,
                 (encoder, slot, buffer, offset, size): (
                    Resource<GpuRenderBundleEncoder>,
                    u32,
                    Option<Resource<GpuBuffer>>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let buffer_rep = match buffer.as_ref() {
                        Some(buffer) => caller.data_mut().table.get(buffer)?.rep,
                        None => 0,
                    };
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_set_vertex_buffer_described(
                        &cb,
                        l2_encoder,
                        slot,
                        buffer_rep,
                        offset.unwrap_or(0),
                        size.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.draw-indexed",
                |mut caller,
                 (
                    encoder,
                    index_count,
                    instance_count,
                    first_index,
                    base_vertex,
                    first_instance,
                ): (
                    Resource<GpuRenderBundleEncoder>,
                    u32,
                    Option<u32>,
                    Option<u32>,
                    Option<i32>,
                    Option<u32>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_draw_indexed_described(
                        &cb,
                        l2_encoder,
                        index_count,
                        instance_count.unwrap_or(1),
                        first_index.unwrap_or(0),
                        base_vertex.unwrap_or(0),
                        first_instance.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.draw-indirect",
                |mut caller,
                 (encoder, buffer, offset): (
                    Resource<GpuRenderBundleEncoder>,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_draw_indirect_described(
                        &cb, l2_encoder, buffer_rep, offset,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.draw-indexed-indirect",
                |mut caller,
                 (encoder, buffer, offset): (
                    Resource<GpuRenderBundleEncoder>,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_draw_indexed_indirect_described(
                        &cb, l2_encoder, buffer_rep, offset,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.push-debug-group",
                |mut caller, (encoder, group_label): (Resource<GpuRenderBundleEncoder>, String)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_push_debug_group_described(
                        &cb,
                        l2_encoder,
                        group_label,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.pop-debug-group",
                |mut caller, (encoder,): (Resource<GpuRenderBundleEncoder>,)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_pop_debug_group_described(&cb, l2_encoder)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.insert-debug-marker",
                |mut caller, (encoder, marker_label): (Resource<GpuRenderBundleEncoder>, String)| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    jvm::exp_render_bundle_encoder_insert_debug_marker_described(
                        &cb,
                        l2_encoder,
                        marker_label,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-immediates",
                |mut caller,
                 (encoder, range_offset, data, data_offset, data_size): (
                    Resource<GpuRenderBundleEncoder>,
                    u32,
                    Vec<u8>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_encoder = if encoder_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_create_render_bundle_encoder_described(
                            &cb,
                            device_rep,
                            GpuTextureFormat::Rgba8unorm.to_dawn_u32(),
                            1,
                        )
                        .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    let _ = data_size;
                    jvm::exp_render_bundle_encoder_set_immediates_described(
                        &cb,
                        l2_encoder,
                        range_offset,
                        data,
                        data_offset.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-compute-pass", |mut store, ()| {
                let resource = store
                    .data_mut()
                    .table
                    .push(GpuComputePassEncoder { rep: 0 })?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.label",
                |mut caller, (pass,): (Resource<GpuComputePassEncoder>,)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    let label = jvm::exp_compute_pass_encoder_label_described(&cb, l2)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((label,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.set-label",
                |mut caller, (pass, label): (Resource<GpuComputePassEncoder>, String)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2 = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_encoder_set_label_described(&cb, l2, label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.end",
                |mut caller, (pass,): (Resource<GpuComputePassEncoder>,)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_end_described(&cb, l2_pass)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.set-pipeline",
                |mut caller,
                 (pass, pipeline): (
                    Resource<GpuComputePassEncoder>,
                    Resource<GpuComputePipeline>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let pipeline_rep = caller.data_mut().table.get(&pipeline)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_set_pipeline_described(&cb, l2_pass, pipeline_rep)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.set-bind-group",
                |mut caller,
                 (pass, index, bind_group, _offsets, _start, _length): (
                    Resource<GpuComputePassEncoder>,
                    u32,
                    Option<Resource<GpuBindGroup>>,
                    Option<Vec<u32>>,
                    Option<u64>,
                    Option<u32>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let bind_group_rep = match bind_group {
                        Some(ref g) => caller.data_mut().table.get(g)?.rep,
                        None => 0,
                    };
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_set_bind_group_described(
                        &cb,
                        l2_pass,
                        index,
                        bind_group_rep,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok((Ok::<(), SetBindGroupError>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.dispatch-workgroups",
                |mut caller,
                 (pass, x, y, z): (
                    Resource<GpuComputePassEncoder>,
                    u32,
                    Option<u32>,
                    Option<u32>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_dispatch_workgroups_described(
                        &cb,
                        l2_pass,
                        x,
                        y.unwrap_or(1),
                        z.unwrap_or(1),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.dispatch-workgroups-indirect",
                |mut caller,
                 (pass, buffer, offset): (
                    Resource<GpuComputePassEncoder>,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let buffer_rep = caller.data_mut().table.get(&buffer)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_dispatch_workgroups_indirect_described(
                        &cb, l2_pass, buffer_rep, offset,
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.set-immediates",
                |mut caller,
                 (pass, range_offset, data, data_offset, data_size): (
                    Resource<GpuComputePassEncoder>,
                    u32,
                    Vec<u8>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    let _ = data_size;
                    jvm::exp_compute_pass_set_immediates_described(
                        &cb,
                        l2_pass,
                        range_offset,
                        data,
                        data_offset.unwrap_or(0),
                    )
                    .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.push-debug-group",
                |mut caller, (pass, group_label): (Resource<GpuComputePassEncoder>, String)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_push_debug_group_described(&cb, l2_pass, group_label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.pop-debug-group",
                |mut caller, (pass,): (Resource<GpuComputePassEncoder>,)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_pop_debug_group_described(&cb, l2_pass)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.insert-debug-marker",
                |mut caller, (pass, marker_label): (Resource<GpuComputePassEncoder>, String)| {
                    let pass_rep = caller.data_mut().table.get(&pass)?.rep;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let l2_pass = if pass_rep == 0 {
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                        let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                            .map_err(wasmtime::Error::msg)?;
                        let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?;
                        jvm::exp_begin_compute_pass(&cb, encoder_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        pass_rep
                    };
                    jvm::exp_compute_pass_insert_debug_marker_described(&cb, l2_pass, marker_label)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent("request-adapter", |accessor, ()| {
                Box::pin(async move {
                    let cb =
                        accessor.with(|mut access| access.data_mut().experimental_host_cb.clone());
                    // Yield so this is true concurrent (not sync wrap / Latch fake-async).
                    let (tx, rx) = oneshot::channel::<()>();
                    std::thread::spawn(move || {
                        let _ = tx.send(());
                    });
                    let _ = rx.await;
                    let Some(cb) = cb else {
                        return Ok((0,));
                    };
                    let rep = jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                    Ok((rep,))
                })
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent("adapter-request-device", |accessor, (adapter,): (u32,)| {
                Box::pin(async move {
                    let cb = accessor.with(|mut access| {
                        access
                            .data_mut()
                            .experimental_host_cb
                            .as_ref()
                            .ok_or_else(|| {
                                wasmtime::Error::msg("experimental host callback not set")
                            })
                            .cloned()
                    })?;
                    let (tx, rx) = oneshot::channel::<()>();
                    std::thread::spawn(move || {
                        let _ = tx.send(());
                    });
                    let _ = rx.await;
                    let rep = jvm::exp_adapter_request_device(&cb, adapter)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((rep,))
                })
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("device-get-queue", |caller, (device,): (u32,)| {
                let cb = caller
                    .data()
                    .experimental_host_cb
                    .as_ref()
                    .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                    .cloned()?;
                let rep = jvm::exp_device_get_queue(&cb, device).map_err(wasmtime::Error::msg)?;
                Ok((rep,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "device-create-command-encoder",
                |caller, (device,): (u32,)| {
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let rep = jvm::exp_create_command_encoder(&cb, device)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((rep,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("command-encoder-finish", |caller, (encoder,): (u32,)| {
                let cb = caller
                    .data()
                    .experimental_host_cb
                    .as_ref()
                    .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                    .cloned()?;
                let rep =
                    jvm::exp_command_encoder_finish(&cb, encoder).map_err(wasmtime::Error::msg)?;
                Ok((rep,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("queue-submit1", |caller, (queue, commands): (u32, u32)| {
                let cb = caller
                    .data()
                    .experimental_host_cb
                    .as_ref()
                    .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                    .cloned()?;
                jvm::exp_queue_submit1(&cb, queue, commands).map_err(wasmtime::Error::msg)?;
                Ok(())
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "command-encoder-begin-render-pass-clear",
                |caller, (encoder, view): (u32, u32)| {
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let rep = jvm::exp_begin_render_pass_clear(&cb, encoder, view)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((rep,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("render-pass-end", |caller, (pass,): (u32,)| {
                let cb = caller
                    .data()
                    .experimental_host_cb
                    .as_ref()
                    .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                    .cloned()?;
                jvm::exp_render_pass_end(&cb, pass).map_err(wasmtime::Error::msg)?;
                Ok(())
            })
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeEngineNew(
    mut env: JNIEnv,
    _class: JClass,
) -> jlong {
    match new_engine() {
        Ok(engine) => to_handle(engine),
        Err(e) => {
            throw(&mut env, e);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeEngineClose(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    unsafe { drop_handle::<Engine>(handle) }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeStoreNew(
    mut env: JNIEnv,
    _class: JClass,
    engine: jlong,
) -> jlong {
    if engine == 0 {
        throw(&mut env, "null engine handle");
        return 0;
    }
    let engine = unsafe { from_handle::<Engine>(engine) };
    to_handle(Store::new(engine, HostState::default()))
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeStoreClose(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    unsafe { drop_handle::<HostStore>(handle) }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeStoreSetHostAdd(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    callback: JObject,
) {
    if store == 0 {
        throw(&mut env, "null store handle");
        return;
    }
    if callback.is_null() {
        throw(&mut env, "null host add callback");
        return;
    }
    let gref = match jvm::global_ref(&mut env, callback) {
        Ok(g) => g,
        Err(e) => {
            throw(&mut env, e);
            return;
        }
    };
    let store = unsafe { from_handle::<HostStore>(store) };
    store.data_mut().add_cb = Some(gref);
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeStoreSetExperimentalHost(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    callback: JObject,
) {
    if store == 0 {
        throw(&mut env, "null store handle");
        return;
    }
    if callback.is_null() {
        throw(&mut env, "null experimental host callback");
        return;
    }
    let gref = match jvm::global_ref(&mut env, callback) {
        Ok(g) => g,
        Err(e) => {
            throw(&mut env, e);
            return;
        }
    };
    let store = unsafe { from_handle::<HostStore>(store) };
    store.data_mut().experimental_host_cb = Some(gref);
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeComponentCompile(
    mut env: JNIEnv,
    _class: JClass,
    engine: jlong,
    bytes: JByteArray,
) -> jlong {
    if engine == 0 {
        throw(&mut env, "null engine handle");
        return 0;
    }
    let engine = unsafe { from_handle::<Engine>(engine) };
    let data = match env.convert_byte_array(&bytes) {
        Ok(d) => d,
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    match Component::new(engine, &data) {
        Ok(c) => to_handle(c),
        Err(e) => {
            throw_compile(&mut env, e);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeComponentClose(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    unsafe { drop_handle::<Component>(handle) }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeLinkerNew(
    mut env: JNIEnv,
    _class: JClass,
    engine: jlong,
) -> jlong {
    if engine == 0 {
        throw(&mut env, "null engine handle");
        return 0;
    }
    let engine = unsafe { from_handle::<Engine>(engine) };
    let mut linker = Linker::<HostState>::new(engine);
    if let Err(e) = define_host(&mut linker) {
        throw_link(&mut env, e);
        return 0;
    }
    to_handle(linker)
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeLinkerClose(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    unsafe { drop_handle::<Linker<HostState>>(handle) }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeInstantiate(
    mut env: JNIEnv,
    _class: JClass,
    linker: jlong,
    store: jlong,
    component: jlong,
) -> jlong {
    if linker == 0 || store == 0 || component == 0 {
        throw(&mut env, "null linker/store/component handle");
        return 0;
    }
    let linker = unsafe { from_handle::<Linker<HostState>>(linker) };
    let store = unsafe { from_handle::<HostStore>(store) };
    let component = unsafe { from_handle::<Component>(component) };
    match pollster::block_on(linker.instantiate_async(&mut *store, component)) {
        Ok(instance) => to_handle(instance),
        Err(e) => {
            throw_link(&mut env, e);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeInstanceClose(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    unsafe { drop_handle::<wasmtime::component::Instance>(handle) }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeCallU32(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    instance: jlong,
    export_name: JString,
    arg: jint,
) -> jint {
    if store == 0 || instance == 0 {
        throw(&mut env, "null store/instance handle");
        return 0;
    }
    let name: String = match env.get_string(&export_name) {
        Ok(s) => s.into(),
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    let store = unsafe { from_handle::<HostStore>(store) };
    let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };
    let func = match instance.get_typed_func::<(u32,), (u32,)>(&mut *store, name.as_str()) {
        Ok(f) => f,
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    match func.call(&mut *store, (arg as u32,)) {
        Ok((result,)) => result as jint,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeCallUnitToU32(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    instance: jlong,
    export_name: JString,
) -> jint {
    if store == 0 || instance == 0 {
        throw(&mut env, "null store/instance handle");
        return 0;
    }
    let name: String = match env.get_string(&export_name) {
        Ok(s) => s.into(),
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    let store = unsafe { from_handle::<HostStore>(store) };
    let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };
    let func = match instance.get_typed_func::<(), (u32,)>(&mut *store, name.as_str()) {
        Ok(f) => f,
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    match func.call(&mut *store, ()) {
        Ok((result,)) => result as jint,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeCallUnitToU64(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    instance: jlong,
    export_name: JString,
) -> jlong {
    if store == 0 || instance == 0 {
        throw(&mut env, "null store/instance handle");
        return 0;
    }
    let name: String = match env.get_string(&export_name) {
        Ok(s) => s.into(),
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    let store = unsafe { from_handle::<HostStore>(store) };
    let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };
    let func = match instance.get_typed_func::<(), (u64,)>(&mut *store, name.as_str()) {
        Ok(f) => f,
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    match func.call(&mut *store, ()) {
        Ok((result,)) => result as jlong,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeCallU32U32(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    instance: jlong,
    export_name: JString,
    a: jint,
    b: jint,
) -> jint {
    if store == 0 || instance == 0 {
        throw(&mut env, "null store/instance handle");
        return 0;
    }
    let name: String = match env.get_string(&export_name) {
        Ok(s) => s.into(),
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    let store = unsafe { from_handle::<HostStore>(store) };
    let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };
    let func = match instance.get_typed_func::<(u32, u32), (u32,)>(&mut *store, name.as_str()) {
        Ok(f) => f,
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    match func.call(&mut *store, (a as u32, b as u32)) {
        Ok((result,)) => result as jint,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}

/// M4: call root export `(u64, u32, u32) -> u32` (e.g. `run-clear`).
#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeCallU64U32U32(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    instance: jlong,
    export_name: JString,
    a: jlong,
    b: jint,
    c: jint,
) -> jint {
    if store == 0 || instance == 0 {
        throw(&mut env, "null store/instance handle");
        return 0;
    }
    let name: String = match env.get_string(&export_name) {
        Ok(s) => s.into(),
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    let store = unsafe { from_handle::<HostStore>(store) };
    let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };
    let func = match instance.get_typed_func::<(u64, u32, u32), (u32,)>(&mut *store, name.as_str())
    {
        Ok(f) => f,
        Err(e) => {
            throw_err(&mut env, e);
            return 0;
        }
    };
    match func.call(&mut *store, (a as u64, b as u32, c as u32)) {
        Ok((result,)) => result as jint,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}

/// ART instrument threads are ~1MiB; W3 extra JNI hops overflow that.
/// Pump Wasmtime on an 8MiB pthread; bounce L2 JNI to the caller (ART aborts
/// AttachCurrentThread on a custom-stack pthread — Java Thread stackSize is ignored).
const CM_PUMP_STACK_BYTES: usize = 8 * 1024 * 1024;

/// M2: call root export `run: func() -> u32` under `run_concurrent` / `call_concurrent`.
#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeCallRunConcurrent(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    instance: jlong,
) -> jint {
    if store == 0 || instance == 0 {
        throw(&mut env, "null store/instance handle");
        return 0;
    }

    // Sync export that sync-lowers an async import: drive with run_concurrent + call_concurrent.
    // (Matches Wasmtime's sync-lower-async-host pattern; pollster pumps the event loop.)
    let result = match jvm::run_on_cm_pump(&mut env, CM_PUMP_STACK_BYTES, move || {
        let store = unsafe { from_handle::<HostStore>(store) };
        let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };
        pollster::block_on(async {
            store
                .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                    let func = accessor.with(|mut access| {
                        instance.get_typed_func::<(), (u32,)>(&mut access, "run")
                    })?;
                    let (value,) = func.call_concurrent(accessor, ()).await?;
                    Ok(value)
                })
                .await?
        })
    }) {
        Ok(inner) => inner,
        Err(e) => {
            throw(&mut env, e);
            return 0;
        }
    };

    match result {
        Ok(v) => v as jint,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}

/// P3-PRIM-3: host `StreamReader` (fixed `P3ST` bytes) → guest export `read`.
/// Packed result: `(nbytes << 4) | status` (status 1 = DROPPED).
#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeCallStreamRead(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    instance: jlong,
    max_len: jint,
) -> jint {
    if store == 0 || instance == 0 {
        throw(&mut env, "null store/instance handle");
        return 0;
    }
    if max_len <= 0 {
        throw(&mut env, "max_len must be positive");
        return 0;
    }
    let store = unsafe { from_handle::<HostStore>(store) };
    let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };

    let result = (|| -> wasmtime::Result<u32> {
        let func =
            instance.get_typed_func::<(StreamReader<u8>, u32), (u32,)>(&mut *store, "read")?;
        let reader = StreamReader::new(&mut *store, b"P3ST".to_vec())?;
        let (packed,) = pollster::block_on(func.call_async(&mut *store, (reader, max_len as u32)))?;
        Ok(packed)
    })();

    match result {
        Ok(v) => v as jint,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}

/// P3-PRIM-5: guest `stream.write` → host `take`/`StreamConsumer`; returns byte count.
#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeCallStreamWrite(
    mut env: JNIEnv,
    _class: JClass,
    store: jlong,
    instance: jlong,
) -> jint {
    if store == 0 || instance == 0 {
        throw(&mut env, "null store/instance handle");
        return 0;
    }
    // Same 8MiB pump as nativeCallRunConcurrent: ART instrument threads are
    // ~1MiB; run_concurrent + StreamConsumer on that stack crashes the
    // instrumentation process (Vivo). Do not AttachCurrentThread on the pump.
    let result = match jvm::run_on_cm_pump(&mut env, CM_PUMP_STACK_BYTES, move || {
        let store = unsafe { from_handle::<HostStore>(store) };
        let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };
        pollster::block_on(async {
            store
                .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                    let func = accessor.with(|mut access| {
                        instance.get_typed_func::<(), (u32,)>(&mut access, "run")
                    })?;
                    let (n,) = func.call_concurrent(accessor, ()).await?;
                    Ok(n)
                })
                .await?
        })
    }) {
        Ok(inner) => inner,
        Err(e) => {
            throw(&mut env, e);
            return 0;
        }
    };

    match result {
        Ok(v) => v as jint,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}
