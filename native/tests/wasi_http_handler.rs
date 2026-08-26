//! WASI 0.3: wasi:http incoming-handler in-process ABI smoke.

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum HttpErrorCode {
    #[component(name = "unknown")]
    Unknown,
}

struct HttpRequest;

struct HttpResponse {
    status: u16,
}

struct TestHost {
    table: ResourceTable,
}

fn register(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
    let mut types = linker.instance("wasi:http/types@0.3.0")?;
    types.resource(
        "request",
        ResourceType::host::<HttpRequest>(),
        |mut store, rep| {
            let resource = Resource::<HttpRequest>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    types.resource(
        "response",
        ResourceType::host::<HttpResponse>(),
        |mut store, rep| {
            let resource = Resource::<HttpResponse>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    types.func_wrap("[constructor]request", |mut store, ()| {
        let resource = store.data_mut().table.push(HttpRequest)?;
        Ok((resource,))
    })?;
    types.func_wrap("[constructor]response", |mut store, ()| {
        let resource = store.data_mut().table.push(HttpResponse { status: 200 })?;
        Ok((resource,))
    })?;
    types.func_wrap(
        "[method]response.status-code",
        |mut store, (resp,): (Resource<HttpResponse>,)| {
            Ok((store.data_mut().table.get(&resp)?.status,))
        },
    )?;
    Ok(())
}

fn engine() -> wasmtime::Result<Engine> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    Engine::new(&config)
}

fn load_component(engine: &Engine) -> wasmtime::Result<Component> {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/http_handler.wasm"
    ))?;
    Component::new(engine, bytes)
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
fn wasi_http_handler_run_returns_200() -> wasmtime::Result<()> {
    let engine = engine()?;
    let component = load_component(&engine)?;
    let mut linker = Linker::new(&engine);
    register(&mut linker)?;
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
    assert_eq!(v, 200);
    Ok(())
}

#[test]
fn wasi_http_incoming_handler_export() -> wasmtime::Result<()> {
    let engine = engine()?;
    let component = load_component(&engine)?;
    let mut linker = Linker::new(&engine);
    register(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let status = pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u16> {
                let req = accessor.with(|mut access| access.data_mut().table.push(HttpRequest))?;
                let idx = accessor.with(|mut access| {
                    let inst = instance
                        .get_export_index(&mut access, None, "wasi:http/incoming-handler@0.3.0")
                        .ok_or_else(|| {
                            wasmtime::Error::msg("missing wasi:http/incoming-handler@0.3.0")
                        })?;
                    instance
                        .get_export_index(&mut access, Some(&inst), "handle")
                        .ok_or_else(|| wasmtime::Error::msg("missing handle"))
                })?;
                let func = accessor.with(|mut access| {
                    instance.get_typed_func::<
                        (Resource<HttpRequest>,),
                        (Result<Resource<HttpResponse>, HttpErrorCode>,),
                    >(&mut access, idx)
                })?;
                let (result,) = func.call_concurrent(accessor, (req,)).await?;
                let resp = result.map_err(|_| wasmtime::Error::msg("handle err"))?;
                accessor.with(|mut access| Ok(access.data_mut().table.get(&resp)?.status))
            })
            .await?
    })?;
    assert_eq!(status, 200);
    Ok(())
}
