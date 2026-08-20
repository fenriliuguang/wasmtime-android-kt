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
    GpuAdapterInfo, GpuBindGroupDescriptor, GpuBindGroupLayoutDescriptor, GpuBufferBindingType,
    GpuBufferDescriptor, GpuBufferMapState, GpuBufferUsage, GpuColor, GpuCommandBufferDescriptor,
    GpuCommandEncoderDescriptor, GpuCompilationInfo, GpuCompilationMessage,
    GpuCompilationMessageType, GpuComputePassDescriptor, GpuComputePipelineDescriptor,
    GpuDeviceDescriptor, GpuDeviceLostInfo, GpuDeviceLostReason, GpuError, GpuErrorFilter,
    GpuErrorKind, GpuExtent3D, GpuIndexFormat, GpuLayoutMode, GpuLoadOp, GpuMapMode,
    GpuPipelineErrorReason, GpuPipelineLayoutDescriptor, GpuQuerySetDescriptor, GpuQueryType,
    GpuRenderBundleDescriptor, GpuRenderBundleEncoderDescriptor, GpuRenderPassDescriptor,
    GpuRenderPipelineDescriptor, GpuRequestAdapterOptions, GpuSamplerDescriptor,
    GpuShaderModuleDescriptor, GpuShaderStage, GpuStoreOp, GpuSupportedFeatures,
    GpuSupportedLimits, GpuTexelCopyBufferInfo, GpuTexelCopyBufferLayout, GpuTexelCopyTextureInfo,
    GpuTextureDescriptor, GpuTextureDimension, GpuTextureFormat, GpuTextureUsage,
    GpuTextureViewDescriptor, GpuTextureViewDimension, GpuUncapturedErrorEvent, MapAsyncError,
    PopErrorScopeError, RecordGpuPipelineConstantValue, RecordOptionGpuSize64, RequestDeviceError,
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
    // and `[method]gpu-device.create-texture` (S6+: sync (borrow, gpu-texture-descriptor) -> own<gpu-texture>) and
    // `[method]gpu-device.create-sampler` (S8: sync (borrow, option<gpu-sampler-descriptor>) -> own<gpu-sampler>)
    // and S6+ `[method]gpu-device.create-shader-module` (sync (borrow, gpu-shader-module-descriptor) -> own<gpu-shader-module>; L2 described WGSL code)
    // and `[method]gpu-queue.write-buffer-with-copy` (S6+: borrow buffer + list data → result; L2 described bytes + offset)
    // and S5 `[method]gpu-queue.submit` (sync void; list<borrow<gpu-command-buffer>>; L2 described handles)
    // and S7 `[method]gpu-command-encoder.finish` (sync (borrow, option<gpu-command-buffer-descriptor>) -> own<gpu-command-buffer>; L2 described label)
    // and `gpu-texture` + `get-texture` + S8 `[method]gpu-texture.create-view` (sync (borrow, option<gpu-texture-view-descriptor>) -> own<gpu-texture-view>)
    // and S6+ `[method]gpu-texture.*` info getters / label / set-label (L2 described extent: width/height/depth/mip; remaining still lift-only).
    // and S6+ `[method]record-gpu-pipeline-constant-value.*` map methods (lift-only stubs).
    // and S6+ `[method]gpu-device.create-bind-group-layout` (sync (borrow, gpu-bind-group-layout-descriptor) -> own<gpu-bind-group-layout>; L2 described first entry)
    // and S6+ `[method]gpu-device.create-pipeline-layout` (sync (borrow, gpu-pipeline-layout-descriptor) -> own<gpu-pipeline-layout>; L2 described BGL handles + label)
    // and S6+ `[method]gpu-device.create-bind-group` (sync (borrow, gpu-bind-group-descriptor) -> own<gpu-bind-group>; L2 described layout + label)
    // and S6+ `[method]gpu-device.create-render-pipeline` (sync (borrow, gpu-render-pipeline-descriptor) -> own<gpu-render-pipeline>; L2 described vertex shader/entry + layout + label)
    // and S6+ `[method]gpu-device.create-compute-pipeline` (sync (borrow, gpu-compute-pipeline-descriptor) -> own<gpu-compute-pipeline>; L2 described shader/entry/layout/label)
    // and `[method]gpu-queue.write-texture-with-copy` (S6+: texel copy info + list data; L2 described bytes + size)
    // and S8 `[method]gpu-command-encoder.begin-compute-pass` (sync (borrow, option<gpu-compute-pass-descriptor>) -> own<gpu-compute-pass-encoder>; L2 described timestamp-write indices)
    // and S6+ `[method]gpu-command-encoder.begin-render-pass` (sync (borrow, gpu-render-pass-descriptor) -> own<gpu-render-pass-encoder>; L2 described first color-attachment view/load/store)
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
                 (record, _key, _value): (Resource<RecordOptionGpuSize64>, String, Option<u64>)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.get",
                |mut caller, (record, _key): (Resource<RecordOptionGpuSize64>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok((None::<Option<u64>>,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.has",
                |mut caller, (record, _key): (Resource<RecordOptionGpuSize64>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok((false,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-option-gpu-size64.remove",
                |mut caller, (record, _key): (Resource<RecordOptionGpuSize64>, String)| {
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
                    let view_rep =
                        jvm::exp_texture_create_view_described(&cb, l2_texture, dimension, aspect)
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
                        jvm::exp_texture_create_view_described(&cb, texture_rep, 0, 0)
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
                        jvm::exp_texture_create_view_described(&cb, texture_rep, 0, 0)
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
                        jvm::exp_create_sampler_described(&cb, device_rep, 0, 0, 0)
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
                        jvm::exp_create_sampler_described(&cb, device_rep, 0, 0, 0)
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
                    let shader_rep = jvm::exp_create_shader_module_described(&cb, l2_device, code)
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
                |mut caller, (record, _key): (Resource<RecordGpuPipelineConstantValue>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok((None::<f64>,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.has",
                |mut caller, (record, _key): (Resource<RecordGpuPipelineConstantValue>, String)| {
                    let _ = caller.data_mut().table.get(&record)?;
                    Ok((false,))
                },
            )
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap(
                "[method]record-gpu-pipeline-constant-value.remove",
                |mut caller, (record, _key): (Resource<RecordGpuPipelineConstantValue>, String)| {
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
                    let (binding, visibility, buffer_type) = match descriptor.entries.first() {
                        Some(entry) => {
                            let buffer_type = match entry.buffer.as_ref().and_then(|b| b.ty) {
                                Some(GpuBufferBindingType::Uniform) => 0,
                                Some(GpuBufferBindingType::Storage) => 1,
                                Some(GpuBufferBindingType::ReadOnlyStorage) => 2,
                                None => -1,
                            };
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
                            (entry.binding as i32, visibility, buffer_type)
                        }
                        None => (0, 0, -1),
                    };
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
                        binding,
                        visibility,
                        buffer_type,
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
                    let bg_rep = jvm::exp_create_bind_group_described(
                        &cb, l2_device, l2_layout, label,
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
                    let label = descriptor.label.clone().unwrap_or_default();
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
                        0,
                        layout_rep,
                        label,
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
                            layout_rep,
                            label,
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
                            let label = descriptor.label.clone().unwrap_or_default();
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
                                layout_rep,
                                label,
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
                            0,
                            layout_rep,
                            label,
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
                        let (cb, device_rep, shader_rep, entry_point, layout_rep, label) =
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
                                let device_rep = access.data_mut().table.get(&device)?.rep;
                                let cb = access
                                    .data_mut()
                                    .experimental_host_cb
                                    .as_ref()
                                    .ok_or_else(|| {
                                        wasmtime::Error::msg("experimental host callback not set")
                                    })
                                    .cloned()?;
                                Ok((cb, device_rep, shader_rep, entry_point, layout_rep, label))
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
                    let encoder_rep = caller.data_mut().table.get(&encoder)?.rep;
                    let (view_rep, load_op, store_op) = match descriptor
                        .color_attachments
                        .iter()
                        .find_map(|att| att.as_ref())
                    {
                        Some(att) => {
                            let load_op = att.load_op.to_dawn_u32();
                            let store_op = att.store_op.to_dawn_u32();
                            let view_rep = caller.data_mut().table.get(&att.view)?.rep;
                            (view_rep, load_op, store_op)
                        }
                        None => (0, GpuLoadOp::Clear.to_dawn_u32(), GpuStoreOp::Store.to_dawn_u32()),
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
                        view_rep,
                        load_op,
                        store_op,
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
