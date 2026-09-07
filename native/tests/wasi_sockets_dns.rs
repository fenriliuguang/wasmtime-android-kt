//! WASI 0.3: wasi:sockets ip-name-lookup (helper thread).

use std::net::ToSocketAddrs;

use wasmtime::component::{Component, ComponentType, Lift, Linker, Lower};
use wasmtime::{Config, Engine, Store};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ComponentType, Lift, Lower)]
#[component(variant)]
enum IpAddress {
    #[component(name = "ipv4")]
    Ipv4((u8, u8, u8, u8)),
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum DnsErrorCode {
    #[component(name = "unknown")]
    Unknown,
    #[component(name = "access-denied")]
    AccessDenied,
    #[component(name = "invalid-argument")]
    InvalidArgument,
    #[component(name = "name-unresolvable")]
    NameUnresolvable,
    #[component(name = "temporary-resolver-failure")]
    TemporaryResolverFailure,
    #[component(name = "permanent-resolver-failure")]
    PermanentResolverFailure,
}

fn dns_error_from_io(err: &std::io::Error) -> DnsErrorCode {
    use std::io::ErrorKind::*;
    match err.kind() {
        PermissionDenied => DnsErrorCode::AccessDenied,
        InvalidInput => DnsErrorCode::InvalidArgument,
        TimedOut | Interrupted => DnsErrorCode::TemporaryResolverFailure,
        _ => DnsErrorCode::NameUnresolvable,
    }
}

fn resolve_name_guest(name: &str) -> Result<Vec<IpAddress>, DnsErrorCode> {
    if name.is_empty() || name.contains('\0') {
        return Err(DnsErrorCode::InvalidArgument);
    }
    if let Ok(ip) = name.parse::<std::net::Ipv4Addr>() {
        let o = ip.octets();
        return Ok(vec![IpAddress::Ipv4((o[0], o[1], o[2], o[3]))]);
    }
    let addrs = (name, 0u16)
        .to_socket_addrs()
        .map_err(|e| dns_error_from_io(&e))?;
    let mut out = Vec::new();
    for addr in addrs {
        if let std::net::SocketAddr::V4(v) = addr {
            let o = v.ip().octets();
            let item = IpAddress::Ipv4((o[0], o[1], o[2], o[3]));
            if !out.contains(&item) {
                out.push(item);
            }
        }
    }
    if out.is_empty() {
        Err(DnsErrorCode::NameUnresolvable)
    } else {
        Ok(out)
    }
}

struct TestHost {}

fn register(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
    let mut dns = linker.instance("wasi:sockets/ip-name-lookup@0.3.0")?;
    dns.func_wrap("resolve-addresses", |_store, (name,): (String,)| {
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = done_tx.send(resolve_name_guest(&name));
        });
        match done_rx
            .recv()
            .map_err(|_| wasmtime::Error::msg("dns lookup canceled"))?
        {
            Ok(addrs) => Ok((Ok(addrs),)),
            Err(e) => Ok((Err(e),)),
        }
    })?;
    Ok(())
}

#[test]
fn empty_name_is_invalid_argument() {
    assert!(matches!(
        resolve_name_guest(""),
        Err(DnsErrorCode::InvalidArgument)
    ));
}

#[test]
fn literal_loopback_is_ipv4() {
    let addrs = resolve_name_guest("127.0.0.1").expect("literal");
    assert_eq!(addrs, vec![IpAddress::Ipv4((127, 0, 0, 1))]);
}

#[test]
fn localhost_has_loopback_ipv4() {
    let addrs = resolve_name_guest("localhost").expect("localhost");
    assert!(addrs.contains(&IpAddress::Ipv4((127, 0, 0, 1))));
}

#[test]
fn wasi_sockets_dns_localhost_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/sockets_dns.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker = Linker::new(&engine);
    register(&mut linker)?;
    let mut store = Store::new(&engine, TestHost {});
    let instance = linker.instantiate(&mut store, &component)?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (n,) = func.call(&mut store, ())?;
    assert_eq!(n, 1);
    Ok(())
}
