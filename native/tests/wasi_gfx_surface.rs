//! P010-GFXH: wasi-gfx:surface@0.2.0 `on-frame` CM stream. Guest pulls;
//! host produces one `frame-event` on a helper thread named `GpuThread`.
//! No JS-style callback. Not the product present loop (P010-GFXL).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
    StreamReader,
};
use wasmtime::{Config, Engine, Store};

static WROTE_ON_GPU_THREAD: AtomicBool = AtomicBool::new(false);

struct GfxSurface;

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GfxSurfaceCreateDesc {
    height: Option<u32>,
    width: Option<u32>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GfxFrameEvent {
    nothing: bool,
}

struct TestHost {
    table: ResourceTable,
}

fn gfx_on_frame_event() -> wasmtime::Result<GfxFrameEvent> {
    let (tx, rx) = mpsc::sync_channel(1);
    thread::Builder::new()
        .name("GpuThread".into())
        .spawn(move || {
            WROTE_ON_GPU_THREAD.store(
                thread::current().name() == Some("GpuThread"),
                Ordering::SeqCst,
            );
            let _ = tx.send(GfxFrameEvent { nothing: true });
        })
        .map_err(|e| wasmtime::Error::msg(format!("GpuThread: {e}")))?;
    rx.recv_timeout(Duration::from_secs(1))
        .map_err(|e| wasmtime::Error::msg(format!("GpuThread: {e}")))
}

fn register(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
    let mut surface = linker.instance("wasi-gfx:surface/surface@0.2.0")?;
    surface.resource(
        "surface",
        ResourceType::host::<GfxSurface>(),
        |mut store, rep| {
            let resource = Resource::<GfxSurface>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    surface.func_wrap(
        "[constructor]surface",
        |mut store, (_desc,): (GfxSurfaceCreateDesc,)| {
            let resource = store.data_mut().table.push(GfxSurface)?;
            Ok((resource,))
        },
    )?;
    surface.func_wrap(
        "[method]surface.on-frame",
        |mut store, (this,): (Resource<GfxSurface>,)| {
            store.data_mut().table.get(&this)?;
            let ev = gfx_on_frame_event()?;
            let reader = StreamReader::new(&mut store, vec![ev])?;
            Ok((reader,))
        },
    )?;
    Ok(())
}

#[test]
fn wasi_gfx_on_frame_stream_yields() -> wasmtime::Result<()> {
    WROTE_ON_GPU_THREAD.store(false, Ordering::SeqCst);
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/gfx_on_frame.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker = Linker::new(&engine);
    register(&mut linker)?;
    let mut store = Store::new(
        &engine,
        TestHost {
            table: ResourceTable::new(),
        },
    );
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
    assert_eq!(v, 1, "guest read one on-frame event");
    assert!(
        WROTE_ON_GPU_THREAD.load(Ordering::SeqCst),
        "frame-event must be produced on a thread named GpuThread"
    );
    Ok(())
}
