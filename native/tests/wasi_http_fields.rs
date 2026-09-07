//! WASI 0.3: wasi:http fields / headers on request and response.

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
enum HeaderError {
    #[component(name = "invalid-syntax")]
    InvalidSyntax,
    #[component(name = "forbidden")]
    Forbidden,
    #[component(name = "immutable")]
    Immutable,
}

struct HttpRequest {
    headers: Vec<(String, Vec<u8>)>,
}

struct HttpResponse {
    headers: Vec<(String, Vec<u8>)>,
}

struct HttpFields {
    entries: Vec<(String, Vec<u8>)>,
    immutable: bool,
}

struct TestHost {
    table: ResourceTable,
}

fn field_name_ok(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii() && b != 0 && b != b' ' && b != b':')
}

fn fields_get(entries: &[(String, Vec<u8>)], name: &str) -> Vec<Vec<u8>> {
    entries
        .iter()
        .filter(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.clone())
        .collect()
}

fn fields_has(entries: &[(String, Vec<u8>)], name: &str) -> bool {
    entries.iter().any(|(n, _)| n.eq_ignore_ascii_case(name))
}

fn register(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
    let mut types = linker.instance("wasi:http/types@0.3.0")?;
    types.resource(
        "fields",
        ResourceType::host::<HttpFields>(),
        |mut store, rep| {
            let resource = Resource::<HttpFields>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
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
    types.func_wrap("[constructor]fields", |mut store, ()| {
        let resource = store.data_mut().table.push(HttpFields {
            entries: Vec::new(),
            immutable: false,
        })?;
        Ok((resource,))
    })?;
    types.func_wrap("[constructor]request", |mut store, ()| {
        let resource = store.data_mut().table.push(HttpRequest {
            headers: Vec::new(),
        })?;
        Ok((resource,))
    })?;
    types.func_wrap("[constructor]response", |mut store, ()| {
        let resource = store.data_mut().table.push(HttpResponse {
            headers: Vec::new(),
        })?;
        Ok((resource,))
    })?;
    types.func_wrap(
        "[method]fields.get",
        |mut store, (fields, name): (Resource<HttpFields>, String)| {
            let values = fields_get(&store.data_mut().table.get(&fields)?.entries, &name);
            Ok((values,))
        },
    )?;
    types.func_wrap(
        "[method]fields.has",
        |mut store, (fields, name): (Resource<HttpFields>, String)| {
            let has = fields_has(&store.data_mut().table.get(&fields)?.entries, &name);
            Ok((has,))
        },
    )?;
    types.func_wrap(
        "[method]fields.append",
        |mut store, (fields, name, value): (Resource<HttpFields>, String, Vec<u8>)| {
            let f = store.data_mut().table.get_mut(&fields)?;
            if f.immutable {
                return Ok((Err(HeaderError::Immutable),));
            }
            if !field_name_ok(&name) {
                return Ok((Err(HeaderError::InvalidSyntax),));
            }
            f.entries.push((name, value));
            Ok((Ok::<(), HeaderError>(()),))
        },
    )?;
    types.func_wrap(
        "[method]request.get-headers",
        |mut store, (req,): (Resource<HttpRequest>,)| {
            let headers = store.data_mut().table.get(&req)?.headers.clone();
            let resource = store.data_mut().table.push(HttpFields {
                entries: headers,
                immutable: true,
            })?;
            Ok((resource,))
        },
    )?;
    types.func_wrap(
        "[method]response.get-headers",
        |mut store, (resp,): (Resource<HttpResponse>,)| {
            let headers = store.data_mut().table.get(&resp)?.headers.clone();
            let resource = store.data_mut().table.push(HttpFields {
                entries: headers,
                immutable: true,
            })?;
            Ok((resource,))
        },
    )?;
    Ok(())
}

#[test]
fn wasi_http_fields_headers_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/http_fields.wasm"
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
    let instance = linker.instantiate(&mut store, &component)?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (n,) = func.call(&mut store, ())?;
    assert_eq!(n, 1);
    Ok(())
}
