//! WASI 0.3: wasi:http incoming-handler + body stream + outbound send.
//! P010-HCTOR: product-shaped linker omits `[constructor]request` /
//! `[constructor]response`; test wrap keeps them. Host supplies `request`
//! when calling `handle`.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::thread;
use std::time::Duration;

use futures::channel::oneshot;
use wasmtime::component::{
    Component, ComponentType, FutureReader, Lift, Linker, Lower, Resource, ResourceTable,
    ResourceType, Source, StreamConsumer, StreamReader, StreamResult,
};
use wasmtime::{Config, Engine, Store, StoreContextMut};

const PAYLOAD: &[u8] = b"HBOD";

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct DnsErrorPayload {
    rcode: Option<String>,
    #[component(name = "info-code")]
    info_code: Option<u16>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct TlsAlertReceivedPayload {
    #[component(name = "alert-id")]
    alert_id: Option<u8>,
    #[component(name = "alert-message")]
    alert_message: Option<String>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct FieldSizePayload {
    #[component(name = "field-name")]
    field_name: Option<String>,
    #[component(name = "field-size")]
    field_size: Option<u32>,
}

/// WASI 0.3.0 `wasi:http` `error-code` (official variant; last case `internal-error`).
#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum HttpErrorCode {
    #[component(name = "DNS-timeout")]
    DnsTimeout,
    #[component(name = "DNS-error")]
    DnsError(DnsErrorPayload),
    #[component(name = "destination-not-found")]
    DestinationNotFound,
    #[component(name = "destination-unavailable")]
    DestinationUnavailable,
    #[component(name = "destination-IP-prohibited")]
    DestinationIpProhibited,
    #[component(name = "destination-IP-unroutable")]
    DestinationIpUnroutable,
    #[component(name = "connection-refused")]
    ConnectionRefused,
    #[component(name = "connection-terminated")]
    ConnectionTerminated,
    #[component(name = "connection-timeout")]
    ConnectionTimeout,
    #[component(name = "connection-read-timeout")]
    ConnectionReadTimeout,
    #[component(name = "connection-write-timeout")]
    ConnectionWriteTimeout,
    #[component(name = "connection-limit-reached")]
    ConnectionLimitReached,
    #[component(name = "TLS-protocol-error")]
    TlsProtocolError,
    #[component(name = "TLS-certificate-error")]
    TlsCertificateError,
    #[component(name = "TLS-alert-received")]
    TlsAlertReceived(TlsAlertReceivedPayload),
    #[component(name = "HTTP-request-denied")]
    HttpRequestDenied,
    #[component(name = "HTTP-request-length-required")]
    HttpRequestLengthRequired,
    #[component(name = "HTTP-request-body-size")]
    HttpRequestBodySize(Option<u64>),
    #[component(name = "HTTP-request-method-invalid")]
    HttpRequestMethodInvalid,
    #[component(name = "HTTP-request-URI-invalid")]
    HttpRequestUriInvalid,
    #[component(name = "HTTP-request-URI-too-long")]
    HttpRequestUriTooLong,
    #[component(name = "HTTP-request-header-section-size")]
    HttpRequestHeaderSectionSize(Option<u32>),
    #[component(name = "HTTP-request-header-size")]
    HttpRequestHeaderSize(Option<FieldSizePayload>),
    #[component(name = "HTTP-request-trailer-section-size")]
    HttpRequestTrailerSectionSize(Option<u32>),
    #[component(name = "HTTP-request-trailer-size")]
    HttpRequestTrailerSize(FieldSizePayload),
    #[component(name = "HTTP-response-incomplete")]
    HttpResponseIncomplete,
    #[component(name = "HTTP-response-header-section-size")]
    HttpResponseHeaderSectionSize(Option<u32>),
    #[component(name = "HTTP-response-header-size")]
    HttpResponseHeaderSize(FieldSizePayload),
    #[component(name = "HTTP-response-body-size")]
    HttpResponseBodySize(Option<u64>),
    #[component(name = "HTTP-response-trailer-section-size")]
    HttpResponseTrailerSectionSize(Option<u32>),
    #[component(name = "HTTP-response-trailer-size")]
    HttpResponseTrailerSize(FieldSizePayload),
    #[component(name = "HTTP-response-transfer-coding")]
    HttpResponseTransferCoding(Option<String>),
    #[component(name = "HTTP-response-content-coding")]
    HttpResponseContentCoding(Option<String>),
    #[component(name = "HTTP-response-timeout")]
    HttpResponseTimeout,
    #[component(name = "HTTP-upgrade-failed")]
    HttpUpgradeFailed,
    #[component(name = "HTTP-protocol-error")]
    HttpProtocolError,
    #[component(name = "loop-detected")]
    LoopDetected,
    #[component(name = "configuration-error")]
    ConfigurationError,
    #[component(name = "internal-error")]
    InternalError(Option<String>),
}

fn http_authority_reject(authority: &str) -> Option<HttpErrorCode> {
    let rest = if authority.len() >= 6 && authority[..6].eq_ignore_ascii_case("https:") {
        authority[6..].trim_start_matches('/')
    } else if authority.len() >= 5 && authority[..5].eq_ignore_ascii_case("http:") {
        authority[5..].trim_start_matches('/')
    } else {
        authority
    };
    if rest.is_empty() || rest.contains('/') {
        Some(HttpErrorCode::HttpRequestUriInvalid)
    } else {
        None
    }
}

fn http_error_from_io(err: &std::io::Error) -> HttpErrorCode {
    use std::io::ErrorKind::*;
    let msg = err.to_string();
    if msg.starts_with("tls-cert:") {
        return HttpErrorCode::TlsCertificateError;
    }
    if msg.starts_with("tls:") {
        return HttpErrorCode::TlsProtocolError;
    }
    match err.kind() {
        InvalidInput => HttpErrorCode::HttpRequestUriInvalid,
        ConnectionRefused => HttpErrorCode::ConnectionRefused,
        TimedOut => HttpErrorCode::ConnectionTimeout,
        ConnectionReset | ConnectionAborted => HttpErrorCode::ConnectionTerminated,
        _ => HttpErrorCode::InternalError(None),
    }
}

struct HttpRequest {
    body: Vec<u8>,
    authority: String,
    method: Method,
    path_with_query: Option<String>,
    scheme: Option<Scheme>,
}

impl HttpRequest {
    fn incoming(authority: &str) -> Self {
        Self {
            body: PAYLOAD.to_vec(),
            authority: authority.to_string(),
            method: Method::Get,
            path_with_query: None,
            scheme: None,
        }
    }
}

/// WASI 0.3.0 `wasi:http` `method`.
#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum Method {
    #[component(name = "get")]
    Get,
    #[component(name = "head")]
    Head,
    #[component(name = "post")]
    Post,
    #[component(name = "put")]
    Put,
    #[component(name = "delete")]
    Delete,
    #[component(name = "connect")]
    Connect,
    #[component(name = "options")]
    Options,
    #[component(name = "trace")]
    Trace,
    #[component(name = "patch")]
    Patch,
    #[component(name = "other")]
    Other(String),
}

/// WASI 0.3.0 `wasi:http` `scheme`.
#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum Scheme {
    #[component(name = "HTTP")]
    Http,
    #[component(name = "HTTPS")]
    Https,
    #[component(name = "other")]
    Other(String),
}

struct HttpResponse {
    status: u16,
    body: Arc<Mutex<Vec<u8>>>,
}

struct HttpFields {}

struct TestHost {
    table: ResourceTable,
}

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

impl StreamConsumer<TestHost> for CollectConsumer {
    type Item = u8;

    fn poll_consume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        store: StoreContextMut<TestHost>,
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
            let _ = cx;
            return Poll::Pending;
        }
        let n = chunk.len();
        this.buf.lock().unwrap().extend_from_slice(chunk);
        src.mark_read(n);
        Poll::Ready(Ok(StreamResult::Completed))
    }
}

fn register(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
    register_http(linker, true)
}

fn register_product(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
    register_http(linker, false)
}

fn register_http(linker: &mut Linker<TestHost>, fixture_ctors: bool) -> wasmtime::Result<()> {
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
    types.resource(
        "fields",
        ResourceType::host::<HttpFields>(),
        |mut store, rep| {
            let resource = Resource::<HttpFields>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    if fixture_ctors {
        types.func_wrap("[constructor]request", |mut store, ()| {
            let resource = store.data_mut().table.push(HttpRequest::incoming(""))?;
            Ok((resource,))
        })?;
        types.func_wrap("[constructor]response", |mut store, ()| {
            let resource = store.data_mut().table.push(HttpResponse {
                status: 200,
                body: Arc::new(Mutex::new(Vec::new())),
            })?;
            Ok((resource,))
        })?;
    }
    types.func_wrap(
        "[method]request.get-method",
        |mut store, (req,): (Resource<HttpRequest>,)| {
            Ok((store.data_mut().table.get(&req)?.method.clone(),))
        },
    )?;
    types.func_wrap(
        "[method]request.get-path-with-query",
        |mut store, (req,): (Resource<HttpRequest>,)| {
            Ok((store.data_mut().table.get(&req)?.path_with_query.clone(),))
        },
    )?;
    types.func_wrap(
        "[method]request.get-scheme",
        |mut store, (req,): (Resource<HttpRequest>,)| {
            Ok((store.data_mut().table.get(&req)?.scheme.clone(),))
        },
    )?;
    types.func_wrap(
        "[method]request.get-authority",
        |mut store, (req,): (Resource<HttpRequest>,)| {
            let auth = &store.data_mut().table.get(&req)?.authority;
            let out = if auth.is_empty() {
                None
            } else {
                Some(auth.clone())
            };
            Ok((out,))
        },
    )?;
    types.func_wrap(
        "[method]response.status-code",
        |mut store, (resp,): (Resource<HttpResponse>,)| {
            Ok((store.data_mut().table.get(&resp)?.status,))
        },
    )?;
    types.func_wrap(
        "[method]response.get-status-code",
        |mut store, (resp,): (Resource<HttpResponse>,)| {
            Ok((store.data_mut().table.get(&resp)?.status,))
        },
    )?;
    types.func_wrap(
        "[method]response.set-status-code",
        |mut store, (resp, status): (Resource<HttpResponse>, u16)| {
            if !(100..=599).contains(&status) {
                return Ok((Err::<(), ()>(()),));
            }
            store.data_mut().table.get_mut(&resp)?.status = status;
            Ok((Ok::<(), ()>(()),))
        },
    )?;
    types.func_wrap(
        "[static]request.consume-body",
        |mut store, (this,): (Resource<HttpRequest>,)| {
            let req = store.data_mut().table.delete(this)?;
            let reader = StreamReader::new(&mut store, req.body)?;
            let fut = FutureReader::new(&mut store, async move {
                Ok::<_, wasmtime::Error>(Ok::<Option<Resource<HttpFields>>, HttpErrorCode>(None))
            })?;
            Ok(((reader, fut),))
        },
    )?;
    types.func_wrap(
        "[static]response.new",
        |mut store, (reader,): (StreamReader<u8>,)| {
            let buf = Arc::new(Mutex::new(Vec::new()));
            let (tx, rx) = oneshot::channel::<u32>();
            reader.pipe(
                &mut store,
                CollectConsumer {
                    buf: buf.clone(),
                    done: Some(tx),
                },
            )?;
            let resource = store.data_mut().table.push(HttpResponse {
                status: 200,
                body: buf,
            })?;
            let fut = FutureReader::new(&mut store, async move {
                let _n = rx.await.unwrap_or(0);
                Ok::<_, wasmtime::Error>(Ok::<(), HttpErrorCode>(()))
            })?;
            Ok(((resource, fut),))
        },
    )?;
    types.func_wrap(
        "[static]response.consume-body",
        |mut store, (this,): (Resource<HttpResponse>,)| {
            let resp = store.data_mut().table.delete(this)?;
            let bytes = resp.body.lock().map(|b| b.clone()).unwrap_or_default();
            let reader = StreamReader::new(&mut store, bytes)?;
            let fut = FutureReader::new(&mut store, async move {
                Ok::<_, wasmtime::Error>(Ok::<Option<Resource<HttpFields>>, HttpErrorCode>(None))
            })?;
            Ok(((reader, fut),))
        },
    )?;
    types.func_wrap(
        "[method]request.set-authority",
        |mut store, (req, authority): (Resource<HttpRequest>, String)| {
            if let Some(code) = http_authority_reject(&authority) {
                return Ok((Err(code),));
            }
            store.data_mut().table.get_mut(&req)?.authority = authority;
            Ok((Ok::<(), HttpErrorCode>(()),))
        },
    )?;
    drop(types);
    let mut client = linker.instance("wasi:http/client@0.3.0")?;
    client.resource(
        "request",
        ResourceType::host::<HttpRequest>(),
        |mut store, rep| {
            let resource = Resource::<HttpRequest>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    client.resource(
        "response",
        ResourceType::host::<HttpResponse>(),
        |mut store, rep| {
            let resource = Resource::<HttpResponse>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    client.func_wrap("send", |mut store, (req,): (Resource<HttpRequest>,)| {
        let authority = store.data_mut().table.delete(req)?.authority;
        if let Some(code) = http_authority_reject(&authority) {
            return Ok((Err(code),));
        }
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        thread::spawn(move || {
            let _ = done_tx.send(http_send_get(&authority));
        });
        let outcome = done_rx
            .recv()
            .map_err(|_| wasmtime::Error::msg("send canceled"))?;
        match outcome {
            Ok((status, body)) => {
                let resource = store.data_mut().table.push(HttpResponse {
                    status,
                    body: Arc::new(Mutex::new(body)),
                })?;
                Ok((Ok::<Resource<HttpResponse>, HttpErrorCode>(resource),))
            }
            Err(e) => Ok((Err(http_error_from_io(&e)),)),
        }
    })?;
    Ok(())
}

fn engine() -> wasmtime::Result<Engine> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    Engine::new(&config)
}

fn load_component(engine: &Engine, file: &str) -> wasmtime::Result<Component> {
    let path = format!("{}/../fixtures/wasi/{file}", env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(path)?;
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

fn call_run(engine: &Engine, file: &str) -> wasmtime::Result<u32> {
    let component = load_component(engine, file)?;
    let mut linker = Linker::new(engine);
    register(&mut linker)?;
    let mut store = new_store(engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                let func = accessor
                    .with(|mut access| instance.get_typed_func::<(), (u32,)>(&mut access, "run"))?;
                let (value,) = func.call_concurrent(accessor, ()).await?;
                Ok(value)
            })
            .await?
    })
}

#[test]
fn wasi_http_handler_run_returns_200() -> wasmtime::Result<()> {
    let engine = engine()?;
    assert_eq!(call_run(&engine, "http_handler.wasm")?, 200);
    Ok(())
}

#[test]
fn wasi_http_incoming_handler_export() -> wasmtime::Result<()> {
    let engine = engine()?;
    let component = load_component(&engine, "http_handler.wasm")?;
    let mut linker = Linker::new(&engine);
    register(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let status = pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u16> {
                let req = accessor
                    .with(|mut access| access.data_mut().table.push(HttpRequest::incoming("")))?;
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

#[test]
fn product_linker_rejects_http_ctors() -> wasmtime::Result<()> {
    let engine = engine()?;
    let component = load_component(&engine, "http_handler.wasm")?;
    let mut linker = Linker::new(&engine);
    register_product(&mut linker)?;
    let mut store = new_store(&engine);
    let err = pollster::block_on(linker.instantiate_async(&mut store, &component))
        .expect_err("constructor guest must not instantiate on the product-shaped linker");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("[constructor]request")
            || msg.contains("[constructor]response")
            || msg.contains("unknown import")
            || msg.contains("import"),
        "link error should mention missing HTTP constructor, got: {msg}"
    );
    Ok(())
}

#[test]
fn product_linker_run_returns_200() -> wasmtime::Result<()> {
    let engine = engine()?;
    let component = load_component(&engine, "http_handle.wasm")?;
    let mut linker = Linker::new(&engine);
    register_product(&mut linker)?;
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
    assert_eq!(v, 200, "product linker + http_handle run returns 200");
    Ok(())
}

#[test]
fn product_linker_handle_host_supplies_request() -> wasmtime::Result<()> {
    let engine = engine()?;
    let component = load_component(&engine, "http_handle.wasm")?;
    let mut linker = Linker::new(&engine);
    register_product(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let status = pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u16> {
                let req = accessor
                    .with(|mut access| access.data_mut().table.push(HttpRequest::incoming("")))?;
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
    assert_eq!(
        status, 200,
        "host-supplied request; product handle returns 200"
    );
    Ok(())
}

fn svc_req(
    method: Method,
    path: Option<&str>,
    scheme: Option<Scheme>,
    authority: &str,
) -> HttpRequest {
    HttpRequest {
        body: PAYLOAD.to_vec(),
        authority: authority.to_string(),
        method,
        path_with_query: path.map(str::to_string),
        scheme,
    }
}

fn call_handle_status(engine: &Engine, req: HttpRequest) -> wasmtime::Result<u16> {
    let component = load_component(engine, "http_svc.wasm")?;
    let mut linker = Linker::new(engine);
    register_product(&mut linker)?;
    let mut store = new_store(engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u16> {
                let req = accessor.with(|mut access| access.data_mut().table.push(req))?;
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
    })
}

#[test]
fn wasi_http_svc_run_returns_200() -> wasmtime::Result<()> {
    let engine = engine()?;
    assert_eq!(call_run_product(&engine, "http_svc.wasm")?, 200);
    Ok(())
}

fn call_run_product(engine: &Engine, file: &str) -> wasmtime::Result<u32> {
    let component = load_component(engine, file)?;
    let mut linker = Linker::new(engine);
    register_product(&mut linker)?;
    let mut store = new_store(engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                let func = accessor
                    .with(|mut access| instance.get_typed_func::<(), (u32,)>(&mut access, "run"))?;
                let (value,) = func.call_concurrent(accessor, ()).await?;
                Ok(value)
            })
            .await?
    })
}

#[test]
fn wasi_http_svc_handle_get_svc_returns_201() -> wasmtime::Result<()> {
    let engine = engine()?;
    let status = call_handle_status(
        &engine,
        svc_req(
            Method::Get,
            Some("/svc"),
            Some(Scheme::Http),
            "example.test",
        ),
    )?;
    assert_eq!(status, 201, "GET /svc with authority and HTTP scheme");
    Ok(())
}

#[test]
fn wasi_http_svc_handle_post_returns_405() -> wasmtime::Result<()> {
    let engine = engine()?;
    let status = call_handle_status(
        &engine,
        svc_req(
            Method::Post,
            Some("/svc"),
            Some(Scheme::Http),
            "example.test",
        ),
    )?;
    assert_eq!(status, 405, "POST is not GET");
    Ok(())
}

#[test]
fn wasi_http_svc_handle_wrong_path_returns_404() -> wasmtime::Result<()> {
    let engine = engine()?;
    let status = call_handle_status(
        &engine,
        svc_req(
            Method::Get,
            Some("/nope"),
            Some(Scheme::Http),
            "example.test",
        ),
    )?;
    assert_eq!(status, 404, "path other than /svc");
    Ok(())
}

#[test]
fn wasi_http_svc_handle_missing_authority_returns_400() -> wasmtime::Result<()> {
    let engine = engine()?;
    let status = call_handle_status(
        &engine,
        svc_req(Method::Get, Some("/svc"), Some(Scheme::Http), ""),
    )?;
    assert_eq!(status, 400, "empty authority is none");
    Ok(())
}

#[test]
fn wasi_http_svc_handle_missing_scheme_returns_400() -> wasmtime::Result<()> {
    let engine = engine()?;
    let status = call_handle_status(
        &engine,
        svc_req(Method::Get, Some("/svc"), None, "example.test"),
    )?;
    assert_eq!(status, 400, "scheme none");
    Ok(())
}

#[test]
fn wasi_http_body_stream_echo() -> wasmtime::Result<()> {
    let engine = engine()?;
    assert_eq!(call_run(&engine, "http_body.wasm")?, PAYLOAD.len() as u32);
    Ok(())
}

const HOUT: &[u8] = b"HOUT";
const P3HA: &[u8; 4] = b"P3HA";

fn http_send_get(authority: &str) -> std::io::Result<(u16, Vec<u8>)> {
    let (https, hostport) = if authority.len() >= 6 && authority[..6].eq_ignore_ascii_case("https:")
    {
        (true, authority[6..].trim_start_matches('/'))
    } else if authority.len() >= 5 && authority[..5].eq_ignore_ascii_case("http:") {
        (false, authority[5..].trim_start_matches('/'))
    } else {
        (false, authority)
    };
    if hostport.is_empty() || hostport.contains('/') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "authority",
        ));
    }
    let (host, port) = hostport
        .rsplit_once(':')
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "host:port"))?;
    let ip: Ipv4Addr = host
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{e}")))?;
    let port: u16 = port
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{e}")))?;
    let mut stream =
        TcpStream::connect_timeout(&SocketAddr::from((ip, port)), Duration::from_secs(2))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let req = format!("GET / HTTP/1.1\r\nHost: {hostport}\r\nConnection: close\r\n\r\n");
    if https {
        test_tls_http_get(stream, ip, req.as_bytes())
    } else {
        stream.write_all(req.as_bytes())?;
        stream.shutdown(std::net::Shutdown::Write)?;
        http_read_response(&mut stream)
    }
}

fn http_read_response(stream: &mut dyn Read) -> std::io::Result<(u16, Vec<u8>)> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 1024];
    loop {
        match stream.read(&mut tmp) {
            Ok(0) => break,
            Ok(n) => buf.extend_from_slice(&tmp[..n]),
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
    }
    let split = buf
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "no header end"))?;
    let headers = std::str::from_utf8(&buf[..split])
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{e}")))?;
    let status = headers
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "status"))?;
    Ok((status, buf[split + 4..].to_vec()))
}

#[derive(Debug)]
struct SkipServerVerify;

impl rustls::client::danger::ServerCertVerifier for SkipServerVerify {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

fn test_tls_http_get(
    stream: TcpStream,
    ip: Ipv4Addr,
    req: &[u8],
) -> std::io::Result<(u16, Vec<u8>)> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mut config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(std::sync::Arc::new(SkipServerVerify))
        .with_no_client_auth();
    config.enable_sni = false;
    let name = rustls::pki_types::ServerName::try_from(ip.to_string())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{e}")))?
        .to_owned();
    let conn = rustls::ClientConnection::new(std::sync::Arc::new(config), name)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("tls:{e}")))?;
    let mut tls = rustls::StreamOwned::new(conn, stream);
    tls.write_all(req)?;
    tls.flush()?;
    http_read_response(&mut tls)
}

fn first_non_loopback_ipv4() -> Ipv4Addr {
    let sock = std::net::UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).expect("udp bind");
    sock.connect((Ipv4Addr::new(1, 1, 1, 1), 80))
        .expect("udp connect for local ip");
    match sock.local_addr().expect("local_addr") {
        SocketAddr::V4(a) if !a.ip().is_loopback() && !a.ip().is_unspecified() => *a.ip(),
        other => panic!("need non-loopback ipv4, got {other}"),
    }
}

fn spawn_http_hout(
    ip: Ipv4Addr,
) -> std::io::Result<(
    u16,
    Arc<Mutex<Vec<u8>>>,
    thread::JoinHandle<std::io::Result<()>>,
)> {
    let listener = TcpListener::bind((ip, 0))?;
    listener.set_nonblocking(true)?;
    let port = listener.local_addr()?.port();
    let received = Arc::new(Mutex::new(Vec::new()));
    let received_thread = received.clone();
    let server = thread::spawn(move || -> std::io::Result<()> {
        let start = std::time::Instant::now();
        let (mut sock, _) = loop {
            match listener.accept() {
                Ok(pair) => break pair,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if start.elapsed() > Duration::from_secs(2) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "http accept timeout",
                        ));
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(e),
            }
        };
        sock.set_nonblocking(false)?;
        sock.set_read_timeout(Some(Duration::from_secs(2)))?;
        sock.set_write_timeout(Some(Duration::from_secs(2)))?;
        let mut buf = Vec::new();
        sock.read_to_end(&mut buf)?;
        *received_thread.lock().unwrap() = buf.clone();
        let resp = b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\nHOUT";
        sock.write_all(resp)?;
        Ok(())
    });
    Ok((port, received, server))
}

fn spawn_https_hout() -> std::io::Result<(
    u16,
    Arc<Mutex<Vec<u8>>>,
    thread::JoinHandle<std::io::Result<()>>,
)> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let key_pair = rcgen::KeyPair::generate().expect("tls key");
    let cert = rcgen::CertificateParams::new(vec!["127.0.0.1".into()])
        .expect("cert params")
        .self_signed(&key_pair)
        .expect("self signed");
    let cert_der = rustls::pki_types::CertificateDer::from(cert.der().to_vec());
    let key_der = rustls::pki_types::PrivateKeyDer::Pkcs8(
        rustls::pki_types::PrivatePkcs8KeyDer::from(key_pair.serialize_der()),
    );
    let cfg = std::sync::Arc::new(
        rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert_der], key_der)
            .expect("server config"),
    );
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    listener.set_nonblocking(true)?;
    let port = listener.local_addr()?.port();
    let received = Arc::new(Mutex::new(Vec::new()));
    let received_thread = received.clone();
    let server = thread::spawn(move || -> std::io::Result<()> {
        let start = std::time::Instant::now();
        let (sock, _) = loop {
            match listener.accept() {
                Ok(pair) => break pair,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if start.elapsed() > Duration::from_secs(2) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "https accept timeout",
                        ));
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(e),
            }
        };
        sock.set_nonblocking(false)?;
        sock.set_read_timeout(Some(Duration::from_secs(2)))?;
        sock.set_write_timeout(Some(Duration::from_secs(2)))?;
        let conn = rustls::ServerConnection::new(cfg)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("tls:{e}")))?;
        let mut tls = rustls::StreamOwned::new(conn, sock);
        let mut buf = vec![0u8; 2048];
        let n = tls.read(&mut buf)?;
        *received_thread.lock().unwrap() = buf[..n].to_vec();
        let resp = b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\nHOUT";
        tls.write_all(resp)?;
        tls.flush()?;
        tls.conn.send_close_notify();
        let _ = tls.flush();
        Ok(())
    });
    Ok((port, received, server))
}

fn patch_p3ha(wasm: &mut [u8], authority: &str) {
    let idx = wasm
        .windows(4)
        .position(|w| w == P3HA)
        .expect("P3HA marker in http_out.wasm");
    let bytes = authority.as_bytes();
    assert!(bytes.len() <= 21, "authority too long");
    assert!(idx + 1 + 21 < wasm.len(), "truncated P3HA record");
    wasm[idx + 4] = bytes.len() as u8;
    wasm[idx + 5..idx + 5 + bytes.len()].copy_from_slice(bytes);
}

#[test]
fn wasi_http_outbound_send_hits_bound_peer() -> wasmtime::Result<()> {
    let ip = first_non_loopback_ipv4();
    assert!(!ip.is_loopback());
    let (port, received, server) = spawn_http_hout(ip).expect("http bind");
    let authority = format!("{ip}:{port}");

    let engine = engine()?;
    let mut bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/http_out.wasm"
    ))?;
    patch_p3ha(&mut bytes, &authority);
    let component = Component::new(&engine, bytes)?;
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
    server.join().unwrap().unwrap();
    assert_eq!(v, HOUT.len() as u32);
    let seen = received.lock().unwrap();
    assert!(
        seen.starts_with(b"GET / HTTP/1.1"),
        "host must wire-send (peer saw {:?})",
        String::from_utf8_lossy(&seen)
    );
    Ok(())
}

#[test]
fn http_error_from_io_refused_is_connection_refused() {
    let err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
    assert!(matches!(
        http_error_from_io(&err),
        HttpErrorCode::ConnectionRefused
    ));
}

#[test]
fn wasi_http_empty_authority_is_uri_invalid() -> wasmtime::Result<()> {
    let engine = engine()?;
    assert_eq!(
        call_run(&engine, "http_empty_authority.wasm")?,
        19,
        "guest must see HTTP-request-URI-invalid (disc 19)"
    );
    Ok(())
}

#[test]
fn https_helper_roundtrip() {
    let (port, received, server) = spawn_https_hout().expect("https bind");
    let (status, body) = http_send_get(&format!("https:127.0.0.1:{port}")).expect("https get");
    server.join().unwrap().unwrap();
    assert_eq!(status, 200);
    assert_eq!(body, HOUT);
    assert!(received
        .lock()
        .unwrap()
        .windows(14)
        .any(|w| w == b"GET / HTTP/1.1"));
}

#[test]
fn wasi_http_https_send_hits_local_rustls() -> wasmtime::Result<()> {
    let (port, received, server) = spawn_https_hout().expect("https bind");
    let authority = format!("https:127.0.0.1:{port}");
    assert!(authority.len() <= 21, "authority {authority}");

    let engine = engine()?;
    let mut bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/http_https_tls.wasm"
    ))?;
    patch_p3ha(&mut bytes, &authority);
    let component = Component::new(&engine, bytes)?;
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
    server.join().unwrap().unwrap();
    assert_eq!(v, HOUT.len() as u32);
    let seen = received.lock().unwrap();
    assert!(
        seen.windows(b"GET / HTTP/1.1".len())
            .any(|w| w == b"GET / HTTP/1.1"),
        "tls host must wire-send (peer saw {:?})",
        String::from_utf8_lossy(&seen)
    );
    Ok(())
}
