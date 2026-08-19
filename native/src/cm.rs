//! Component Model JNI (M1 sync + M2 concurrent/async + P3 stream).

use crate::engine::new_engine;
use crate::error::{throw, throw_compile, throw_err, throw_link};
use crate::handles::{drop_handle, from_handle, to_handle};
use crate::host::{
    Gpu, GpuAdapter, GpuBindGroup, GpuBindGroupLayout, GpuBuffer, GpuCommandBuffer,
    GpuCommandEncoder, GpuComputePassEncoder, GpuComputePipeline, GpuDevice, GpuPipelineLayout,
    GpuQueue, GpuRenderBundle, GpuRenderBundleEncoder, GpuRenderPassEncoder, GpuRenderPipeline,
    GpuSampler, GpuShaderModule, GpuTexture, GpuTextureView, HostState, Widget,
};
use crate::jvm;
use crate::webgpu_abi::{
    CreatePipelineError, CreatePipelineErrorKind, CreateQuerySetError, GetMappedRangeError,
    GpuAdapterInfo, GpuBindGroupDescriptor, GpuBindGroupLayoutDescriptor, GpuBufferDescriptor,
    GpuBufferMapState, GpuBufferUsage, GpuColor, GpuCommandBufferDescriptor,
    GpuCommandEncoderDescriptor, GpuCompilationInfo, GpuCompilationMessage,
    GpuCompilationMessageType, GpuComputePassDescriptor, GpuComputePipelineDescriptor,
    GpuDeviceDescriptor, GpuExtent3D, GpuIndexFormat, GpuMapMode, GpuPipelineErrorReason,
    GpuPipelineLayoutDescriptor, GpuQuerySet, GpuQuerySetDescriptor, GpuQueryType,
    GpuRenderBundleDescriptor, GpuRenderBundleEncoderDescriptor, GpuRenderPassDescriptor,
    GpuRenderPipelineDescriptor, GpuRequestAdapterOptions, GpuSamplerDescriptor,
    GpuShaderModuleDescriptor, GpuSupportedFeatures, GpuSupportedLimits, GpuDeviceLostInfo,
    GpuDeviceLostReason, GpuError, GpuErrorFilter, GpuErrorKind, GpuUncapturedErrorEvent,
    PopErrorScopeError, GpuTexelCopyBufferInfo,
    GpuTexelCopyBufferLayout, GpuTexelCopyTextureInfo, GpuTextureDescriptor, GpuTextureDimension,
    GpuTextureFormat, GpuTextureUsage, GpuTextureViewDescriptor, GpuTextureViewDimension,
    MapAsyncError,     RecordGpuPipelineConstantValue, RecordOptionGpuSize64,
    RequestDeviceError, RequestDeviceErrorKind, SetBindGroupError, UnmapError, WgslLanguageFeatures,
    WriteBufferError,
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
    // (borrow, option<gpu-command-encoder-descriptor>) -> own<gpu-command-encoder>) and `[method]gpu-device.create-buffer`
    // (S4: sync (borrow, gpu-buffer-descriptor) -> own<gpu-buffer>) and
    // `gpu-buffer` + `get-buffer` + `[method]gpu-buffer.map-async` (S6+: true async
    // result<_, map-async-error>; guest mode/offset/size; L2 still host-fixed MAP_READ buffer)
    // and `[method]gpu-buffer.unmap` (S6+: result<_, unmap-error>; L2 still host-fixed map then unmap)
    // and `[method]gpu-device.create-texture` (S6+: sync (borrow, gpu-texture-descriptor) -> own<gpu-texture>) and
    // `[method]gpu-device.create-sampler` (S8: sync (borrow, option<gpu-sampler-descriptor>) -> own<gpu-sampler>)
    // and S6+ `[method]gpu-device.create-shader-module` (sync (borrow, gpu-shader-module-descriptor) -> own<gpu-shader-module>; L2 still host-fixed WGSL)
    // and `[method]gpu-queue.write-buffer-with-copy` (S6+: borrow buffer + list data → result; L2 still host-fixed 4 bytes)
    // and S5 `[method]gpu-queue.submit` (sync void; list<borrow<gpu-command-buffer>>)
    // and S7 `[method]gpu-command-encoder.finish` (sync (borrow, option<gpu-command-buffer-descriptor>) -> own<gpu-command-buffer>)
    // and `gpu-texture` + `get-texture` + S8 `[method]gpu-texture.create-view` (sync (borrow, option<gpu-texture-view-descriptor>) -> own<gpu-texture-view>)
    // and S6+ `[method]gpu-texture.*` info getters / label / set-label (lift-only stubs).
    // and S6+ `[method]record-gpu-pipeline-constant-value.*` map methods (lift-only stubs).
    // and S6+ `[method]gpu-device.create-bind-group-layout` (sync (borrow, gpu-bind-group-layout-descriptor) -> own<gpu-bind-group-layout>; L2 still host-fixed empty entries)
    // and S6+ `[method]gpu-device.create-pipeline-layout` (sync (borrow, gpu-pipeline-layout-descriptor) -> own<gpu-pipeline-layout>; L2 still host-fixed empty bind-group-layouts)
    // and S6+ `[method]gpu-device.create-bind-group` (sync (borrow, gpu-bind-group-descriptor) -> own<gpu-bind-group>; L2 still host-fixed empty BGL + empty entries)
    // and S6+ `[method]gpu-device.create-render-pipeline` (sync (borrow, gpu-render-pipeline-descriptor) -> own<gpu-render-pipeline>; L2 still host-fixed stub shader + triangle)
    // and S6+ `[method]gpu-device.create-compute-pipeline` (sync (borrow, gpu-compute-pipeline-descriptor) -> own<gpu-compute-pipeline>; L2 still host-fixed stub shader + empty layout)
    // and `[method]gpu-queue.write-texture-with-copy` (S6+: texel copy info + list data; L2 still host-fixed 1×1)
    // and S8 `[method]gpu-command-encoder.begin-compute-pass` (sync (borrow, option<gpu-compute-pass-descriptor>) -> own<gpu-compute-pass-encoder>)
    // and S6+ `[method]gpu-command-encoder.begin-render-pass` (sync (borrow, gpu-render-pass-descriptor) -> own<gpu-render-pass-encoder>; L2 still host-fixed view)
    // and `gpu-compute-pass-encoder` + `get-compute-pass` + `[method]gpu-compute-pass-encoder.end` (sync void)
    // and `[method]gpu-compute-pass-encoder.set-pipeline` (S6+: borrow<gpu-compute-pipeline>; L2 still host-fixed compute pipeline)
    // and `[method]gpu-compute-pass-encoder.set-bind-group` (S6+: index + option bind-group + option offsets → result; L2 still host-fixed empty bind-group)
    // and `[method]gpu-compute-pass-encoder.dispatch-workgroups` (S6+: x + option y/z; L2 still host-fixed 1x1x1)
    // and S6+ remaining compute-pass recording: dispatch-workgroups-indirect / set-immediates /
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
    // and `[method]gpu-render-pass-encoder.set-bind-group` (S6+: index + option bind-group + option offsets → result; L2 still host-fixed empty bind-group)
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
                |accessor, (gpu, _options): (Resource<Gpu>, Option<GpuRequestAdapterOptions>)| {
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
                        let adapter_rep =
                            jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
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
                    Ok((GpuTextureFormat::Rgba8unorm,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu.wgsl-language-features",
                |mut caller, (gpu,): (Resource<Gpu>,)| {
                    let _ = caller.data_mut().table.get(&gpu)?;
                    let resource = caller.data_mut().table.push(WgslLanguageFeatures)?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]wgsl-language-features.has",
                |mut caller, (features, _value): (Resource<WgslLanguageFeatures>, String)| {
                    let _ = caller.data_mut().table.get(&features)?;
                    Ok((false,))
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
                    let _ = caller.data_mut().table.get(&adapter)?;
                    let resource = caller.data_mut().table.push(GpuSupportedFeatures)?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-features.has",
                |mut caller, (features, _value): (Resource<GpuSupportedFeatures>, String)| {
                    let _ = caller.data_mut().table.get(&features)?;
                    Ok((false,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter.limits",
                |mut caller, (adapter,): (Resource<GpuAdapter>,)| {
                    let _ = caller.data_mut().table.get(&adapter)?;
                    let resource = caller.data_mut().table.push(GpuSupportedLimits)?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-supported-limits", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuSupportedLimits)?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-bind-groups",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-bind-groups-plus-vertex-buffers",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-bindings-per-bind-group",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-buffer-size",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u64,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-color-attachment-bytes-per-sample",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-color-attachments",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-invocations-per-workgroup",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-workgroup-size-x",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-workgroup-size-y",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-workgroup-size-z",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-workgroups-per-dimension",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-compute-workgroup-storage-size",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-dynamic-storage-buffers-per-pipeline-layout",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-dynamic-uniform-buffers-per-pipeline-layout",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-immediate-size",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-inter-stage-shader-variables",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-sampled-textures-per-shader-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-samplers-per-shader-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-buffer-binding-size",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u64,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-buffers-in-fragment-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-buffers-in-vertex-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-buffers-per-shader-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-textures-in-fragment-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-textures-in-vertex-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-storage-textures-per-shader-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-texture-array-layers",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-texture-dimension1-d",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-texture-dimension2-d",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-texture-dimension3-d",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-uniform-buffer-binding-size",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u64,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-uniform-buffers-per-shader-stage",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-vertex-attributes",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-vertex-buffer-array-stride",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.max-vertex-buffers",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.min-storage-buffer-offset-alignment",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-supported-limits.min-uniform-buffer-offset-alignment",
                |mut caller, (limits,): (Resource<GpuSupportedLimits>,)| {
                    let _ = caller.data_mut().table.get(&limits)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter.info",
                |mut caller, (adapter,): (Resource<GpuAdapter>,)| {
                    let _ = caller.data_mut().table.get(&adapter)?;
                    let resource = caller.data_mut().table.push(GpuAdapterInfo)?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-adapter-info", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuAdapterInfo)?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.vendor",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    let _ = caller.data_mut().table.get(&info)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.architecture",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    let _ = caller.data_mut().table.get(&info)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.device",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    let _ = caller.data_mut().table.get(&info)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.description",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    let _ = caller.data_mut().table.get(&info)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.subgroup-min-size",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    let _ = caller.data_mut().table.get(&info)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.subgroup-max-size",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    let _ = caller.data_mut().table.get(&info)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-adapter-info.is-fallback-adapter",
                |mut caller, (info,): (Resource<GpuAdapterInfo>,)| {
                    let _ = caller.data_mut().table.get(&info)?;
                    Ok((false,))
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
                 (record, _key, _value): (Resource<RecordOptionGpuSize64>, String, Option<u64>)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.get",
                |mut caller,
                 (record, _key): (Resource<RecordOptionGpuSize64>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok((None::<Option<u64>>,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.has",
                |mut caller,
                 (record, _key): (Resource<RecordOptionGpuSize64>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok((false,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.remove",
                |mut caller,
                 (record, _key): (Resource<RecordOptionGpuSize64>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.keys",
                |mut caller, (record,): (Resource<RecordOptionGpuSize64>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok((Vec::<String>::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.values",
                |mut caller, (record,): (Resource<RecordOptionGpuSize64>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok((Vec::<Option<u64>>::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.entries",
                |mut caller, (record,): (Resource<RecordOptionGpuSize64>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok((Vec::<(String, Option<u64>)>::new(),))
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
                |accessor, (adapter, _descriptor): (
                    Resource<GpuAdapter>,
                    Option<GpuDeviceDescriptor>,
                )| {
                    Box::pin(async move {
                        let (cb, adapter_rep) = accessor.with(|mut access| {
                            let adapter_rep = access.data_mut().table.get(&adapter)?.rep;
                            let cb = access
                                .data_mut()
                                .experimental_host_cb
                                .as_ref()
                                .ok_or_else(|| {
                                    wasmtime::Error::msg("experimental host callback not set")
                                })
                                .cloned()?;
                            Ok::<_, wasmtime::Error>((cb, adapter_rep))
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
                        let device_rep = jvm::exp_adapter_request_device(&cb, l2_adapter)
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
                    let _ = caller.data_mut().table.get(&device)?;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let adapter_rep =
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                    let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let queue_rep =
                        jvm::exp_device_get_queue(&cb, device_rep).map_err(wasmtime::Error::msg)?;
                    let resource = caller.data_mut().table.push(GpuQueue { rep: queue_rep })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.destroy",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let _ = caller.data_mut().table.get(&device)?;
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
                let resource = store.data_mut().table.push(GpuError)?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-error.message",
                |mut caller, (error,): (Resource<GpuError>,)| {
                    let _ = caller.data_mut().table.get(&error)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-error.kind",
                |mut caller, (error,): (Resource<GpuError>,)| {
                    let _ = caller.data_mut().table.get(&error)?;
                    Ok((GpuErrorKind::ValidationError,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.features",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let _ = caller.data_mut().table.get(&device)?;
                    let resource = caller.data_mut().table.push(GpuSupportedFeatures)?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.limits",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let _ = caller.data_mut().table.get(&device)?;
                    let resource = caller.data_mut().table.push(GpuSupportedLimits)?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.adapter-info",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let _ = caller.data_mut().table.get(&device)?;
                    let resource = caller.data_mut().table.push(GpuAdapterInfo)?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.label",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let _ = caller.data_mut().table.get(&device)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.set-label",
                |mut caller, (device, _label): (Resource<GpuDevice>, String)| {
                    let _ = caller.data_mut().table.get(&device)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.lost",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let _ = caller.data_mut().table.get(&device)?;
                    let info = caller.data_mut().table.push(GpuDeviceLostInfo)?;
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
                |mut caller, (device, _filter): (Resource<GpuDevice>, GpuErrorFilter)| {
                    let _ = caller.data_mut().table.get(&device)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu-device.pop-error-scope",
                |accessor, (device,): (Resource<GpuDevice>,)| {
                    Box::pin(async move {
                        accessor
                            .with(|mut access| access.data_mut().table.get(&device).map(|_| ()))?;
                        Ok((Ok::<Option<Resource<GpuError>>, PopErrorScopeError>(None),))
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.on-uncaptured-error",
                |mut caller, (device,): (Resource<GpuDevice>,)| {
                    let _ = caller.data_mut().table.get(&device)?;
                    let reader =
                        StreamReader::<Resource<GpuError>>::new(&mut caller, vec![])?;
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
                let resource = store.data_mut().table.push(GpuUncapturedErrorEvent)?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-uncaptured-error-event.error",
                |mut caller, (event,): (Resource<GpuUncapturedErrorEvent>,)| {
                    let _ = caller.data_mut().table.get(&event)?;
                    let resource = caller.data_mut().table.push(GpuError)?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-device-lost-info", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuDeviceLostInfo)?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device-lost-info.reason",
                |mut caller, (info,): (Resource<GpuDeviceLostInfo>,)| {
                    let _ = caller.data_mut().table.get(&info)?;
                    Ok((GpuDeviceLostReason::Unknown,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device-lost-info.message",
                |mut caller, (info,): (Resource<GpuDeviceLostInfo>,)| {
                    let _ = caller.data_mut().table.get(&info)?;
                    Ok((String::new(),))
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
                 (device, _descriptor): (
                    Resource<GpuDevice>,
                    Option<GpuCommandEncoderDescriptor>,
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
                    let encoder_rep = jvm::exp_create_command_encoder(&cb, l2_device)
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
                    let buffer_rep = jvm::exp_create_buffer_described(
                        &cb,
                        l2_device,
                        descriptor.size,
                        descriptor.usage.to_webgpu_u32(),
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
                    let texture_rep = jvm::exp_create_texture_described(
                        &cb,
                        l2_device,
                        width,
                        height,
                        depth,
                        descriptor.format.to_dawn_u32(),
                        descriptor.usage.to_webgpu_u32(),
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
                    let (dimension, aspect) = match &descriptor {
                        None => (0, 0),
                        Some(d) => (
                            d.dimension.map(|m| m.to_dawn_u32()).unwrap_or(0),
                            d.aspect.map(|m| m.to_dawn_u32()).unwrap_or(0),
                        ),
                    };
                    let view_rep = jvm::exp_texture_create_view_described(
                        &cb,
                        l2_texture,
                        dimension,
                        aspect,
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
                    let _ = caller.data_mut().table.get(&view)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture-view.set-label",
                |mut caller, (view, _label): (Resource<GpuTextureView>, String)| {
                    let _ = caller.data_mut().table.get(&view)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.destroy",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let _ = caller.data_mut().table.get(&texture)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.width",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let _ = caller.data_mut().table.get(&texture)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.height",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let _ = caller.data_mut().table.get(&texture)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.depth-or-array-layers",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let _ = caller.data_mut().table.get(&texture)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.mip-level-count",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let _ = caller.data_mut().table.get(&texture)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.sample-count",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let _ = caller.data_mut().table.get(&texture)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.dimension",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let _ = caller.data_mut().table.get(&texture)?;
                    Ok((GpuTextureDimension::D2,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.format",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let _ = caller.data_mut().table.get(&texture)?;
                    Ok((GpuTextureFormat::Rgba8unorm,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.usage",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let _ = caller.data_mut().table.get(&texture)?;
                    Ok((GpuTextureUsage::empty(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.texture-binding-view-dimension",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let _ = caller.data_mut().table.get(&texture)?;
                    Ok((None::<GpuTextureViewDimension>,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.label",
                |mut caller, (texture,): (Resource<GpuTexture>,)| {
                    let _ = caller.data_mut().table.get(&texture)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-texture.set-label",
                |mut caller, (texture, _label): (Resource<GpuTexture>, String)| {
                    let _ = caller.data_mut().table.get(&texture)?;
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
                    let _ = caller.data_mut().table.get(&buffer)?;
                    Ok((0u64,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.usage",
                |mut caller, (buffer,): (Resource<GpuBuffer>,)| {
                    let _ = caller.data_mut().table.get(&buffer)?;
                    Ok((GpuBufferUsage::empty(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.map-state",
                |mut caller, (buffer,): (Resource<GpuBuffer>,)| {
                    let _ = caller.data_mut().table.get(&buffer)?;
                    Ok((GpuBufferMapState::Unmapped,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.label",
                |mut caller, (buffer,): (Resource<GpuBuffer>,)| {
                    let _ = caller.data_mut().table.get(&buffer)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.set-label",
                |mut caller, (buffer, _label): (Resource<GpuBuffer>, String)| {
                    let _ = caller.data_mut().table.get(&buffer)?;
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
                    let _ = caller.data_mut().table.get(&buffer)?;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let adapter_rep =
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                    let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let buffer_rep =
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?;
                    jvm::exp_buffer_unmap(&cb, buffer_rep).map_err(wasmtime::Error::msg)?;
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
                    let _ = caller.data_mut().table.get(&buffer)?;
                    let _ = (offset, size);
                    Ok((Ok::<Vec<u8>, GetMappedRangeError>(Vec::new()),))
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
                    let _ = caller.data_mut().table.get(&buffer)?;
                    let _ = (data, offset, size);
                    Ok((Ok::<(), GetMappedRangeError>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-buffer.destroy",
                |mut caller, (buffer,): (Resource<GpuBuffer>,)| {
                    let _ = caller.data_mut().table.get(&buffer)?;
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
                    let (mag_filter, min_filter, address_mode_u) = match &descriptor {
                        None => (0, 0, 0),
                        Some(d) => (
                            d.mag_filter.map(|m| m.to_dawn_u32()).unwrap_or(0),
                            d.min_filter.map(|m| m.to_dawn_u32()).unwrap_or(0),
                            d.address_mode_u.map(|m| m.to_dawn_u32()).unwrap_or(0),
                        ),
                    };
                    let sampler_rep = jvm::exp_create_sampler_described(
                        &cb,
                        l2_device,
                        mag_filter,
                        min_filter,
                        address_mode_u,
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
                    let _ = caller.data_mut().table.get(&sampler)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-sampler.set-label",
                |mut caller, (sampler, _label): (Resource<GpuSampler>, String)| {
                    let _ = caller.data_mut().table.get(&sampler)?;
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
                |mut caller, (device, _descriptor): (
                    Resource<GpuDevice>,
                    GpuShaderModuleDescriptor,
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
                    let shader_rep = jvm::exp_create_shader_module(&cb, l2_device)
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
                        accessor.with(|mut access| {
                            access.data_mut().table.get(&shader).map(|_| ())
                        })?;
                        let resource = accessor.with(|mut access| {
                            access.data_mut().table.push(GpuCompilationInfo)
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
                    let _ = caller.data_mut().table.get(&shader)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-shader-module.set-label",
                |mut caller, (shader, _label): (Resource<GpuShaderModule>, String)| {
                    let _ = caller.data_mut().table.get(&shader)?;
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
            .func_wrap("[constructor]record-gpu-pipeline-constant-value", |mut store, ()| {
                let resource = store
                    .data_mut()
                    .table
                    .push(RecordGpuPipelineConstantValue)?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.add",
                |mut caller,
                 (record, _key, _value): (
                    Resource<RecordGpuPipelineConstantValue>,
                    String,
                    f64,
                )| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.get",
                |mut caller,
                 (record, _key): (Resource<RecordGpuPipelineConstantValue>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok((None::<f64>,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.has",
                |mut caller,
                 (record, _key): (Resource<RecordGpuPipelineConstantValue>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok((false,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.remove",
                |mut caller,
                 (record, _key): (Resource<RecordGpuPipelineConstantValue>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.keys",
                |mut caller, (record,): (Resource<RecordGpuPipelineConstantValue>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok((Vec::<String>::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.values",
                |mut caller, (record,): (Resource<RecordGpuPipelineConstantValue>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok((Vec::<f64>::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.entries",
                |mut caller, (record,): (Resource<RecordGpuPipelineConstantValue>,)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok((Vec::<(String, f64)>::new(),))
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
                    let _ = caller.data_mut().table.get(&layout)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-bind-group-layout.set-label",
                |mut caller, (layout, _label): (Resource<GpuBindGroupLayout>, String)| {
                    let _ = caller.data_mut().table.get(&layout)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-bind-group-layout",
                |mut caller, (device, _descriptor): (
                    Resource<GpuDevice>,
                    GpuBindGroupLayoutDescriptor,
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
                    let layout_rep = jvm::exp_create_bind_group_layout(&cb, l2_device)
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
                |mut caller, (device, _descriptor): (
                    Resource<GpuDevice>,
                    GpuPipelineLayoutDescriptor,
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
                    let layout_rep = jvm::exp_create_pipeline_layout(&cb, l2_device)
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
                    let _ = caller.data_mut().table.get(&layout)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-pipeline-layout.set-label",
                |mut caller, (layout, _label): (Resource<GpuPipelineLayout>, String)| {
                    let _ = caller.data_mut().table.get(&layout)?;
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
                |mut caller, (device, _descriptor): (
                    Resource<GpuDevice>,
                    GpuBindGroupDescriptor,
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
                    let bg_rep = jvm::exp_create_bind_group(&cb, l2_device)
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
                    let _ = caller.data_mut().table.get(&bind_group)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-bind-group.set-label",
                |mut caller, (bind_group, _label): (Resource<GpuBindGroup>, String)| {
                    let _ = caller.data_mut().table.get(&bind_group)?;
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
                    let _ = caller.data_mut().table.get(&pipeline)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pipeline.set-label",
                |mut caller, (pipeline, _label): (Resource<GpuRenderPipeline>, String)| {
                    let _ = caller.data_mut().table.get(&pipeline)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pipeline.get-bind-group-layout",
                |mut caller, (pipeline, _index): (Resource<GpuRenderPipeline>, u32)| {
                    let _ = caller.data_mut().table.get(&pipeline)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuBindGroupLayout { rep: 0 })?;
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
                    let _ = caller.data_mut().table.get(&pipeline)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pipeline.set-label",
                |mut caller, (pipeline, _label): (Resource<GpuComputePipeline>, String)| {
                    let _ = caller.data_mut().table.get(&pipeline)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pipeline.get-bind-group-layout",
                |mut caller, (pipeline, _index): (Resource<GpuComputePipeline>, u32)| {
                    let _ = caller.data_mut().table.get(&pipeline)?;
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuBindGroupLayout { rep: 0 })?;
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
                    let _ = caller.data_mut().table.get(&descriptor.vertex.module)?;
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
                    let pipeline_rep = jvm::exp_create_render_pipeline(&cb, l2_device)
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
                    let _ = caller.data_mut().table.get(&descriptor.compute.module)?;
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
                    let pipeline_rep = jvm::exp_create_compute_pipeline(&cb, l2_device)
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
                        let (cb, device_rep) =
                            accessor.with(|mut access| -> wasmtime::Result<_> {
                                let _ = access.data_mut().table.get(&descriptor.vertex.module)?;
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
                        let pipeline_rep = jvm::exp_create_render_pipeline(&cb, l2_device)
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
                        let (cb, device_rep) =
                            accessor.with(|mut access| -> wasmtime::Result<_> {
                                let _ = access.data_mut().table.get(&descriptor.compute.module)?;
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
                        let pipeline_rep = jvm::exp_create_compute_pipeline(&cb, l2_device)
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
                    let _ = caller.data_mut().table.get(&encoder)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.set-label",
                |mut caller, (encoder, _label): (Resource<GpuCommandEncoder>, String)| {
                    let _ = caller.data_mut().table.get(&encoder)?;
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
                let resource = store.data_mut().table.push(GpuQuerySet)?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-device.create-query-set",
                |mut caller,
                 (device, _descriptor): (Resource<GpuDevice>, GpuQuerySetDescriptor)| {
                    let _ = caller.data_mut().table.get(&device)?;
                    let resource = caller.data_mut().table.push(GpuQuerySet)?;
                    Ok((Ok::<_, CreateQuerySetError>(resource),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-query-set.destroy",
                |mut caller, (query_set,): (Resource<GpuQuerySet>,)| {
                    let _ = caller.data_mut().table.get(&query_set)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-query-set.type",
                |mut caller, (query_set,): (Resource<GpuQuerySet>,)| {
                    let _ = caller.data_mut().table.get(&query_set)?;
                    Ok((GpuQueryType::Occlusion,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-query-set.count",
                |mut caller, (query_set,): (Resource<GpuQuerySet>,)| {
                    let _ = caller.data_mut().table.get(&query_set)?;
                    Ok((1u32,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-query-set.label",
                |mut caller, (query_set,): (Resource<GpuQuerySet>,)| {
                    let _ = caller.data_mut().table.get(&query_set)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-query-set.set-label",
                |mut caller, (query_set, _label): (Resource<GpuQuerySet>, String)| {
                    let _ = caller.data_mut().table.get(&query_set)?;
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
                    let _ = descriptor.color_attachments.len();
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
                    // L2 still host-fixed offscreen view (JNI substitutes a real view).
                    let pass_rep =
                        jvm::exp_begin_render_pass_clear(&cb, l2_encoder, 23)
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
                 (encoder, _descriptor): (
                    Resource<GpuCommandEncoder>,
                    Option<GpuComputePassDescriptor>,
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
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    let pass_rep = jvm::exp_begin_compute_pass(&cb, l2_encoder)
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
                    _first_query,
                    _query_count,
                    destination,
                    _destination_offset,
                ): (
                    Resource<GpuCommandEncoder>,
                    Resource<GpuQuerySet>,
                    u32,
                    u32,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let _ = caller.data_mut().table.get(&encoder)?;
                    let _ = caller.data_mut().table.get(&query_set)?;
                    let _ = caller.data_mut().table.get(&destination)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.push-debug-group",
                |mut caller, (encoder, _group_label): (Resource<GpuCommandEncoder>, String)| {
                    let _ = caller.data_mut().table.get(&encoder)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.pop-debug-group",
                |mut caller, (encoder,): (Resource<GpuCommandEncoder>,)| {
                    let _ = caller.data_mut().table.get(&encoder)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-encoder.insert-debug-marker",
                |mut caller, (encoder, _marker_label): (Resource<GpuCommandEncoder>, String)| {
                    let _ = caller.data_mut().table.get(&encoder)?;
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
                 (encoder, _descriptor): (
                    Resource<GpuCommandEncoder>,
                    Option<GpuCommandBufferDescriptor>,
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
                        jvm::exp_create_command_encoder(&cb, device_rep)
                            .map_err(wasmtime::Error::msg)?
                    } else {
                        encoder_rep
                    };
                    let buffer_rep = jvm::exp_command_encoder_finish(&cb, l2_encoder)
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
                    let _ = caller.data_mut().table.get(&buffer)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-command-buffer.set-label",
                |mut caller, (buffer, _label): (Resource<GpuCommandBuffer>, String)| {
                    let _ = caller.data_mut().table.get(&buffer)?;
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
                let resource = store.data_mut().table.push(GpuCompilationInfo)?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap("get-compilation-message", |mut store, ()| {
                let resource = store.data_mut().table.push(GpuCompilationMessage)?;
                Ok((resource,))
            })
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-info.messages",
                |mut caller, (info,): (Resource<GpuCompilationInfo>,)| {
                    let _ = caller.data_mut().table.get(&info)?;
                    Ok((Vec::<Resource<GpuCompilationMessage>>::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.message",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let _ = caller.data_mut().table.get(&msg)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.type",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let _ = caller.data_mut().table.get(&msg)?;
                    Ok((GpuCompilationMessageType::Error,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.line-num",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let _ = caller.data_mut().table.get(&msg)?;
                    Ok((0u64,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.line-pos",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let _ = caller.data_mut().table.get(&msg)?;
                    Ok((0u64,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.offset",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let _ = caller.data_mut().table.get(&msg)?;
                    Ok((0u64,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compilation-message.length",
                |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
                    let _ = caller.data_mut().table.get(&msg)?;
                    Ok((0u64,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-queue.label",
                |mut caller, (queue,): (Resource<GpuQueue>,)| {
                    let _ = caller.data_mut().table.get(&queue)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-queue.set-label",
                |mut caller, (queue, _label): (Resource<GpuQueue>, String)| {
                    let _ = caller.data_mut().table.get(&queue)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent(
                "[method]gpu-queue.on-submitted-work-done",
                |accessor, (queue,): (Resource<GpuQueue>,)| {
                    Box::pin(async move {
                        accessor
                            .with(|mut access| access.data_mut().table.get(&queue).map(|_| ()))?;
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
                    let _ = caller.data_mut().table.get(&queue)?;
                    for command in &commands {
                        let _ = caller.data_mut().table.get(command)?;
                    }
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let adapter_rep =
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                    let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let queue_rep = jvm::exp_device_get_queue(&cb, device_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let commands_rep = jvm::exp_command_encoder_finish(&cb, encoder_rep)
                        .map_err(wasmtime::Error::msg)?;
                    jvm::exp_queue_submit1(&cb, queue_rep, commands_rep)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-queue.write-buffer-with-copy",
                |mut caller,
                 (queue, _buffer, _offset, _data, _data_offset, _size): (
                    Resource<GpuQueue>,
                    Resource<GpuBuffer>,
                    u64,
                    Vec<u8>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let _ = caller.data_mut().table.get(&queue)?;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let adapter_rep =
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                    let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let queue_rep =
                        jvm::exp_device_get_queue(&cb, device_rep).map_err(wasmtime::Error::msg)?;
                    let buffer_rep =
                        jvm::exp_create_buffer(&cb, device_rep).map_err(wasmtime::Error::msg)?;
                    jvm::exp_queue_write_buffer(&cb, queue_rep, buffer_rep)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((Ok::<(), WriteBufferError>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-queue.write-texture-with-copy",
                |mut caller,
                 (queue, destination, _data, _layout, _size): (
                    Resource<GpuQueue>,
                    GpuTexelCopyTextureInfo,
                    Vec<u8>,
                    GpuTexelCopyBufferLayout,
                    GpuExtent3D,
                )| {
                    let _ = caller.data_mut().table.get(&queue)?;
                    let _ = caller.data_mut().table.get(&destination.texture)?;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let adapter_rep =
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                    let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let queue_rep =
                        jvm::exp_device_get_queue(&cb, device_rep).map_err(wasmtime::Error::msg)?;
                    let texture_rep =
                        jvm::exp_create_texture(&cb, device_rep).map_err(wasmtime::Error::msg)?;
                    jvm::exp_queue_write_texture(&cb, queue_rep, texture_rep)
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
                 (pass, _index, _bind_group, _offsets, _start, _length): (
                    Resource<GpuRenderPassEncoder>,
                    u32,
                    Option<Resource<GpuBindGroup>>,
                    Option<Vec<u32>>,
                    Option<u64>,
                    Option<u32>,
                )| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let adapter_rep =
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                    let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let pass_rep = jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                        .map_err(wasmtime::Error::msg)?;
                    jvm::exp_render_pass_set_bind_group(&cb, pass_rep)
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
                 (pass, _x, _y, _width, _height, _min_depth, _max_depth): (
                    Resource<GpuRenderPassEncoder>,
                    f32,
                    f32,
                    f32,
                    f32,
                    f32,
                    f32,
                )| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-scissor-rect",
                |mut caller,
                 (pass, _x, _y, _width, _height): (
                    Resource<GpuRenderPassEncoder>,
                    u32,
                    u32,
                    u32,
                    u32,
                )| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-blend-constant",
                |mut caller, (pass, _color): (Resource<GpuRenderPassEncoder>, GpuColor)| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-stencil-reference",
                |mut caller, (pass, _reference): (Resource<GpuRenderPassEncoder>, u32)| {
                    let _ = caller.data_mut().table.get(&pass)?;
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
                 (pass, buffer, _offset): (
                    Resource<GpuRenderPassEncoder>,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    let _ = caller.data_mut().table.get(&buffer)?;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let adapter_rep =
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                    let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let pass_rep = jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                        .map_err(wasmtime::Error::msg)?;
                    jvm::exp_render_pass_draw(&cb, pass_rep).map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.draw-indexed-indirect",
                |mut caller,
                 (pass, buffer, _offset): (
                    Resource<GpuRenderPassEncoder>,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    let _ = caller.data_mut().table.get(&buffer)?;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let adapter_rep =
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                    let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let pass_rep = jvm::exp_begin_render_pass_clear(&cb, encoder_rep, 23)
                        .map_err(wasmtime::Error::msg)?;
                    jvm::exp_render_pass_draw(&cb, pass_rep).map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.push-debug-group",
                |mut caller, (pass, _group_label): (Resource<GpuRenderPassEncoder>, String)| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.pop-debug-group",
                |mut caller, (pass,): (Resource<GpuRenderPassEncoder>,)| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.insert-debug-marker",
                |mut caller, (pass, _marker_label): (Resource<GpuRenderPassEncoder>, String)| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.begin-occlusion-query",
                |mut caller, (pass, _query_index): (Resource<GpuRenderPassEncoder>, u32)| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.end-occlusion-query",
                |mut caller, (pass,): (Resource<GpuRenderPassEncoder>,)| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.label",
                |mut caller, (pass,): (Resource<GpuRenderPassEncoder>,)| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-label",
                |mut caller, (pass, _label): (Resource<GpuRenderPassEncoder>, String)| {
                    let _ = caller.data_mut().table.get(&pass)?;
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
                    let _ = caller.data_mut().table.get(&bundle)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle.set-label",
                |mut caller, (bundle, _label): (Resource<GpuRenderBundle>, String)| {
                    let _ = caller.data_mut().table.get(&bundle)?;
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
                    let _ = caller.data_mut().table.get(&pass)?;
                    for bundle in &bundles {
                        let _ = caller.data_mut().table.get(bundle)?;
                    }
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-pass-encoder.set-immediates",
                |mut caller,
                 (pass, _range_offset, _data, _data_offset, _data_size): (
                    Resource<GpuRenderPassEncoder>,
                    u32,
                    Vec<u8>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let _ = caller.data_mut().table.get(&pass)?;
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
                    let _ = caller.data_mut().table.get(&encoder)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-label",
                |mut caller, (encoder, _label): (Resource<GpuRenderBundleEncoder>, String)| {
                    let _ = caller.data_mut().table.get(&encoder)?;
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
                    let _ = caller.data_mut().table.get(&device)?;
                    let _ = descriptor.color_formats.len();
                    let resource = caller
                        .data_mut()
                        .table
                        .push(GpuRenderBundleEncoder { rep: 0 })?;
                    Ok((resource,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.finish",
                |mut caller,
                 (encoder, _descriptor): (
                    Resource<GpuRenderBundleEncoder>,
                    Option<GpuRenderBundleDescriptor>,
                )| {
                    let _ = caller.data_mut().table.get(&encoder)?;
                    let resource = caller.data_mut().table.push(GpuRenderBundle { rep: 0 })?;
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
                    let _ = caller.data_mut().table.get(&encoder)?;
                    let _ = caller.data_mut().table.get(&pipeline)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-bind-group",
                |mut caller,
                 (encoder, _index, bind_group, _offsets, _start, _length): (
                    Resource<GpuRenderBundleEncoder>,
                    u32,
                    Option<Resource<GpuBindGroup>>,
                    Option<Vec<u32>>,
                    Option<u64>,
                    Option<u32>,
                )| {
                    let _ = caller.data_mut().table.get(&encoder)?;
                    if let Some(bind_group) = bind_group.as_ref() {
                        let _ = caller.data_mut().table.get(bind_group)?;
                    }
                    Ok((Ok::<(), SetBindGroupError>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.draw",
                |mut caller,
                 (encoder, _vertex_count, _instance_count, _first_vertex, _first_instance): (
                    Resource<GpuRenderBundleEncoder>,
                    u32,
                    Option<u32>,
                    Option<u32>,
                    Option<u32>,
                )| {
                    let _ = caller.data_mut().table.get(&encoder)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-index-buffer",
                |mut caller,
                 (encoder, buffer, _format, _offset, _size): (
                    Resource<GpuRenderBundleEncoder>,
                    Resource<GpuBuffer>,
                    GpuIndexFormat,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let _ = caller.data_mut().table.get(&encoder)?;
                    let _ = caller.data_mut().table.get(&buffer)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-vertex-buffer",
                |mut caller,
                 (encoder, _slot, buffer, _offset, _size): (
                    Resource<GpuRenderBundleEncoder>,
                    u32,
                    Option<Resource<GpuBuffer>>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let _ = caller.data_mut().table.get(&encoder)?;
                    if let Some(buffer) = buffer.as_ref() {
                        let _ = caller.data_mut().table.get(buffer)?;
                    }
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
                    _index_count,
                    _instance_count,
                    _first_index,
                    _base_vertex,
                    _first_instance,
                ): (
                    Resource<GpuRenderBundleEncoder>,
                    u32,
                    Option<u32>,
                    Option<u32>,
                    Option<i32>,
                    Option<u32>,
                )| {
                    let _ = caller.data_mut().table.get(&encoder)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.draw-indirect",
                |mut caller,
                 (encoder, buffer, _offset): (
                    Resource<GpuRenderBundleEncoder>,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let _ = caller.data_mut().table.get(&encoder)?;
                    let _ = caller.data_mut().table.get(&buffer)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.draw-indexed-indirect",
                |mut caller,
                 (encoder, buffer, _offset): (
                    Resource<GpuRenderBundleEncoder>,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let _ = caller.data_mut().table.get(&encoder)?;
                    let _ = caller.data_mut().table.get(&buffer)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.push-debug-group",
                |mut caller, (encoder, _group_label): (Resource<GpuRenderBundleEncoder>, String)| {
                    let _ = caller.data_mut().table.get(&encoder)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.pop-debug-group",
                |mut caller, (encoder,): (Resource<GpuRenderBundleEncoder>,)| {
                    let _ = caller.data_mut().table.get(&encoder)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.insert-debug-marker",
                |mut caller, (encoder, _marker_label): (Resource<GpuRenderBundleEncoder>, String)| {
                    let _ = caller.data_mut().table.get(&encoder)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-render-bundle-encoder.set-immediates",
                |mut caller,
                 (encoder, _range_offset, _data, _data_offset, _data_size): (
                    Resource<GpuRenderBundleEncoder>,
                    u32,
                    Vec<u8>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let _ = caller.data_mut().table.get(&encoder)?;
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
                    let _ = caller.data_mut().table.get(&pass)?;
                    Ok((String::new(),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.set-label",
                |mut caller, (pass, _label): (Resource<GpuComputePassEncoder>, String)| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.end",
                |mut caller, (pass,): (Resource<GpuComputePassEncoder>,)| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let adapter_rep =
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                    let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let pass_rep = jvm::exp_begin_compute_pass(&cb, encoder_rep)
                        .map_err(wasmtime::Error::msg)?;
                    jvm::exp_compute_pass_end(&cb, pass_rep).map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.set-pipeline",
                |mut caller,
                 (pass, _pipeline): (
                    Resource<GpuComputePassEncoder>,
                    Resource<GpuComputePipeline>,
                )| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let adapter_rep =
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                    let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let pass_rep = jvm::exp_begin_compute_pass(&cb, encoder_rep)
                        .map_err(wasmtime::Error::msg)?;
                    jvm::exp_compute_pass_set_pipeline(&cb, pass_rep)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.set-bind-group",
                |mut caller,
                 (pass, _index, _bind_group, _offsets, _start, _length): (
                    Resource<GpuComputePassEncoder>,
                    u32,
                    Option<Resource<GpuBindGroup>>,
                    Option<Vec<u32>>,
                    Option<u64>,
                    Option<u32>,
                )| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let adapter_rep =
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                    let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let pass_rep = jvm::exp_begin_compute_pass(&cb, encoder_rep)
                        .map_err(wasmtime::Error::msg)?;
                    jvm::exp_compute_pass_set_bind_group(&cb, pass_rep)
                        .map_err(wasmtime::Error::msg)?;
                    Ok((Ok::<(), SetBindGroupError>(()),))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.dispatch-workgroups",
                |mut caller,
                 (pass, _x, _y, _z): (
                    Resource<GpuComputePassEncoder>,
                    u32,
                    Option<u32>,
                    Option<u32>,
                )| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let adapter_rep =
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                    let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let pass_rep = jvm::exp_begin_compute_pass(&cb, encoder_rep)
                        .map_err(wasmtime::Error::msg)?;
                    jvm::exp_compute_pass_dispatch_workgroups(&cb, pass_rep)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.dispatch-workgroups-indirect",
                |mut caller,
                 (pass, buffer, _offset): (
                    Resource<GpuComputePassEncoder>,
                    Resource<GpuBuffer>,
                    u64,
                )| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    let _ = caller.data_mut().table.get(&buffer)?;
                    let cb = caller
                        .data()
                        .experimental_host_cb
                        .as_ref()
                        .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                        .cloned()?;
                    let adapter_rep =
                        jvm::exp_request_adapter(&cb).map_err(wasmtime::Error::msg)?;
                    let device_rep = jvm::exp_adapter_request_device(&cb, adapter_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let encoder_rep = jvm::exp_create_command_encoder(&cb, device_rep)
                        .map_err(wasmtime::Error::msg)?;
                    let pass_rep = jvm::exp_begin_compute_pass(&cb, encoder_rep)
                        .map_err(wasmtime::Error::msg)?;
                    jvm::exp_compute_pass_dispatch_workgroups(&cb, pass_rep)
                        .map_err(wasmtime::Error::msg)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.set-immediates",
                |mut caller,
                 (pass, _range_offset, _data, _data_offset, _data_size): (
                    Resource<GpuComputePassEncoder>,
                    u32,
                    Vec<u8>,
                    Option<u64>,
                    Option<u64>,
                )| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.push-debug-group",
                |mut caller, (pass, _group_label): (Resource<GpuComputePassEncoder>, String)| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.pop-debug-group",
                |mut caller, (pass,): (Resource<GpuComputePassEncoder>,)| {
                    let _ = caller.data_mut().table.get(&pass)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]gpu-compute-pass-encoder.insert-debug-marker",
                |mut caller, (pass, _marker_label): (Resource<GpuComputePassEncoder>, String)| {
                    let _ = caller.data_mut().table.get(&pass)?;
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
