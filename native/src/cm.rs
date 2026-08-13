//! Component Model JNI (M1 sync + M2 concurrent/async + P3 stream).

use crate::engine::new_engine;
use crate::error::{throw, throw_compile, throw_link, throw_err};
use crate::handles::{drop_handle, from_handle, to_handle};
use crate::host::{HostState, Widget};
use crate::jvm;
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
            // Zero-length readiness probe (component-model#561). Returning
            // Completed here can sync-reenter poll_consume until Android's
            // smaller JNI stack overflows (seen on Vivo/arm64 opt-level=0).
            // Pending + wake yields so guest stream.write can progress.
            cx.waker().wake_by_ref();
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
        .func_wrap(
            "make-widget",
            |mut store, (rep,): (u32,)| {
                let resource = store.data_mut().table.push(Widget { rep })?;
                Ok((resource,))
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .root()
        .func_wrap(
            "echo-widget",
            |mut store, (r,): (Resource<Widget>,)| {
                let w = store.data_mut().table.get(&r)?;
                Ok((w.rep,))
            },
        )
        .map_err(|e| e.to_string())?;

    linker
        .root()
        .func_wrap(
            "add",
            |caller, (a, b): (u32, u32)| {
                let cb = caller
                    .data()
                    .add_cb
                    .as_ref()
                    .ok_or_else(|| wasmtime::Error::msg("host add callback not set"))?
                    .clone();
                let result =
                    jvm::call_u32_u32_to_u32(&cb, a, b).map_err(wasmtime::Error::msg)?;
                Ok((result,))
            },
        )
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
                    getrandom::fill(&mut bytes)
                        .map_err(|e| wasmtime::Error::msg(e.to_string()))?;
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

    // WASI 0.3: wasi:clocks/system-clock@0.3.0#now — transitional u64 unix seconds
    // (official WIT is instant {seconds: s64, nanoseconds: u32}; record deferred).
    linker
        .instance("wasi:clocks/system-clock@0.3.0")
        .map_err(|e| e.to_string())?
        .func_wrap("now", |_store, ()| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            Ok((secs,))
        })
        .map_err(|e| e.to_string())?;

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
            let rep = jvm::exp_adapter_request_device(&cb, adapter).map_err(wasmtime::Error::msg)?;
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
    // yield); W3 first slice `device-get-queue` is sync `func_wrap` (same L2 as
    // experimental). Experimental stays sync. Not final `[method]gpu.*` / option /
    // resource (later W3).
    {
        let mut webgpu = linker
            .instance("wasi:webgpu/webgpu@0.3.0-rc.2")
            .map_err(|e| e.to_string())?;
        webgpu
            .func_wrap_concurrent("request-adapter", |accessor, ()| {
                Box::pin(async move {
                    let cb = accessor.with(|mut access| {
                        access
                            .data_mut()
                            .experimental_host_cb
                            .as_ref()
                            .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
                            .cloned()
                    })?;
                    // Yield so this is true concurrent (not sync wrap / Latch fake-async).
                    let (tx, rx) = oneshot::channel::<()>();
                    std::thread::spawn(move || {
                        let _ = tx.send(());
                    });
                    let _ = rx.await;
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
                            .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set"))
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
    let func = match instance.get_typed_func::<(u64, u32, u32), (u32,)>(&mut *store, name.as_str()) {
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
    let store = unsafe { from_handle::<HostStore>(store) };
    let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };

    // Sync export that sync-lowers an async import: drive with run_concurrent + call_concurrent.
    // (Matches Wasmtime's sync-lower-async-host pattern; pollster pumps the event loop.)
    let result = pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                let func = accessor.with(|mut access| {
                    instance.get_typed_func::<(), (u32,)>(&mut access, "run")
                })?;
                let (value,) = func.call_concurrent(accessor, ()).await?;
                Ok(value)
            })
            .await?
    });

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
        let func = instance
            .get_typed_func::<(StreamReader<u8>, u32), (u32,)>(&mut *store, "read")?;
        let reader = StreamReader::new(&mut *store, b"P3ST".to_vec())?;
        let (packed,) =
            pollster::block_on(func.call_async(&mut *store, (reader, max_len as u32)))?;
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
    let store = unsafe { from_handle::<HostStore>(store) };
    let instance = unsafe { *from_handle::<wasmtime::component::Instance>(instance) };

    // Drive with run_concurrent (same as M2 / wait-for). Plain call_async can
    // sync-reenter StreamConsumer on Android until the JNI stack overflows.
    let result = pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                let func = accessor.with(|mut access| {
                    instance.get_typed_func::<(), (u32,)>(&mut access, "run")
                })?;
                let (n,) = func.call_concurrent(accessor, ()).await?;
                Ok(n)
            })
            .await?
    });

    match result {
        Ok(v) => v as jint,
        Err(e) => {
            throw_err(&mut env, e);
            0
        }
    }
}
