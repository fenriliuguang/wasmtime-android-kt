//! WASI 0.3: wasi:sockets TCP bind / listen / accept (loopback sandbox).

use std::io::{ErrorKind, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum SockErrorCode {
    #[component(name = "access-denied")]
    AccessDenied,
    #[component(name = "not-supported")]
    NotSupported,
    #[component(name = "invalid-argument")]
    InvalidArgument,
    #[component(name = "out-of-memory")]
    OutOfMemory,
    #[component(name = "timeout")]
    Timeout,
    #[component(name = "invalid-state")]
    InvalidState,
    #[component(name = "address-not-bindable")]
    AddressNotBindable,
    #[component(name = "address-in-use")]
    AddressInUse,
    #[component(name = "remote-unreachable")]
    RemoteUnreachable,
    #[component(name = "connection-refused")]
    ConnectionRefused,
    #[component(name = "connection-broken")]
    ConnectionBroken,
    #[component(name = "connection-reset")]
    ConnectionReset,
    #[component(name = "connection-aborted")]
    ConnectionAborted,
    #[component(name = "datagram-too-large")]
    DatagramTooLarge,
    #[component(name = "other")]
    Other(Option<String>),
}

fn sock_error_from_io(err: &std::io::Error) -> SockErrorCode {
    use std::io::ErrorKind::*;
    match err.kind() {
        PermissionDenied => SockErrorCode::AccessDenied,
        InvalidInput => SockErrorCode::InvalidArgument,
        OutOfMemory => SockErrorCode::OutOfMemory,
        TimedOut => SockErrorCode::Timeout,
        AddrNotAvailable => SockErrorCode::AddressNotBindable,
        AddrInUse => SockErrorCode::AddressInUse,
        HostUnreachable | NetworkUnreachable | NetworkDown => SockErrorCode::RemoteUnreachable,
        ConnectionRefused => SockErrorCode::ConnectionRefused,
        BrokenPipe => SockErrorCode::ConnectionBroken,
        ConnectionReset => SockErrorCode::ConnectionReset,
        ConnectionAborted => SockErrorCode::ConnectionAborted,
        Unsupported => SockErrorCode::NotSupported,
        _ => SockErrorCode::Other(None),
    }
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum IpAddressFamily {
    #[component(name = "ipv4")]
    Ipv4,
    #[component(name = "ipv6")]
    Ipv6,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct Ipv4SocketAddress {
    port: u16,
    address: (u8, u8, u8, u8),
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
enum IpSocketAddress {
    #[component(name = "ipv4")]
    Ipv4(Ipv4SocketAddress),
}

struct TcpSocket {
    #[allow(dead_code)]
    client: Option<TcpStream>,
    listener: Option<std::net::TcpListener>,
}

struct TestHost {
    table: ResourceTable,
}

fn tcp_bind_guest(addr: IpSocketAddress) -> std::io::Result<std::net::TcpListener> {
    match addr {
        IpSocketAddress::Ipv4(a) => {
            let ip = Ipv4Addr::new(a.address.0, a.address.1, a.address.2, a.address.3);
            if !ip.is_loopback() {
                return Err(std::io::Error::new(
                    ErrorKind::PermissionDenied,
                    "listen sandbox is loopback only",
                ));
            }
            std::net::TcpListener::bind((ip, a.port))
        }
    }
}

fn tcp_addr_from_std(addr: SocketAddr) -> IpSocketAddress {
    match addr {
        SocketAddr::V4(v) => {
            let o = v.ip().octets();
            IpSocketAddress::Ipv4(Ipv4SocketAddress {
                port: v.port(),
                address: (o[0], o[1], o[2], o[3]),
            })
        }
        SocketAddr::V6(_) => IpSocketAddress::Ipv4(Ipv4SocketAddress {
            port: 0,
            address: (127, 0, 0, 1),
        }),
    }
}

fn register(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
    {
        let mut tcp = linker.instance("wasi:sockets/tcp@0.3.0")?;
        tcp.resource(
            "tcp-socket",
            ResourceType::host::<TcpSocket>(),
            |mut store, rep| {
                let resource = Resource::<TcpSocket>::new_own(rep);
                store.data_mut().table.delete(resource)?;
                Ok(())
            },
        )?;
        tcp.func_wrap(
            "[method]tcp-socket.bind",
            |mut store, (sock, addr): (Resource<TcpSocket>, IpSocketAddress)| {
                store.data_mut().table.get(&sock)?;
                let (done_tx, done_rx) = std::sync::mpsc::channel();
                thread::spawn(move || {
                    let _ = done_tx.send(tcp_bind_guest(addr));
                });
                let listener = match done_rx
                    .recv()
                    .map_err(|_| wasmtime::Error::msg("bind canceled"))?
                {
                    Ok(l) => l,
                    Err(e) => return Ok((Err(sock_error_from_io(&e)),)),
                };
                store.data_mut().table.get_mut(&sock)?.listener = Some(listener);
                Ok((Ok::<(), SockErrorCode>(()),))
            },
        )?;
        tcp.func_wrap(
            "[method]tcp-socket.listen",
            |mut store, (sock,): (Resource<TcpSocket>,)| {
                if store.data_mut().table.get(&sock)?.listener.is_some() {
                    return Ok((Ok::<(), SockErrorCode>(()),));
                }
                let (done_tx, done_rx) = std::sync::mpsc::channel();
                thread::spawn(move || {
                    let _ =
                        done_tx.send(tcp_bind_guest(IpSocketAddress::Ipv4(Ipv4SocketAddress {
                            port: 0,
                            address: (127, 0, 0, 1),
                        })));
                });
                let listener = match done_rx
                    .recv()
                    .map_err(|_| wasmtime::Error::msg("listen canceled"))?
                {
                    Ok(l) => l,
                    Err(e) => return Ok((Err(sock_error_from_io(&e)),)),
                };
                store.data_mut().table.get_mut(&sock)?.listener = Some(listener);
                Ok((Ok::<(), SockErrorCode>(()),))
            },
        )?;
        tcp.func_wrap(
            "[method]tcp-socket.accept",
            |mut store, (sock,): (Resource<TcpSocket>,)| {
                let listener = match store.data_mut().table.get(&sock)?.listener.as_ref() {
                    Some(l) => l.try_clone()?,
                    None => return Ok((Err(SockErrorCode::InvalidState),)),
                };
                let (done_tx, done_rx) = std::sync::mpsc::channel();
                thread::spawn(move || {
                    let _ = done_tx.send(listener.accept());
                });
                let (stream, peer) = match done_rx
                    .recv()
                    .map_err(|_| wasmtime::Error::msg("accept canceled"))?
                {
                    Ok(v) => v,
                    Err(e) => return Ok((Err(sock_error_from_io(&e)),)),
                };
                let child = store.data_mut().table.push(TcpSocket {
                    client: Some(stream),
                    listener: None,
                })?;
                Ok((Ok((child, tcp_addr_from_std(peer))),))
            },
        )?;
    }
    {
        let mut create = linker.instance("wasi:sockets/tcp-create-socket@0.3.0")?;
        create.resource(
            "tcp-socket",
            ResourceType::host::<TcpSocket>(),
            |mut store, rep| {
                let resource = Resource::<TcpSocket>::new_own(rep);
                store.data_mut().table.delete(resource)?;
                Ok(())
            },
        )?;
        create.func_wrap(
            "create-tcp-socket",
            |mut store, (family,): (IpAddressFamily,)| match family {
                IpAddressFamily::Ipv4 => {
                    let resource = store.data_mut().table.push(TcpSocket {
                        client: None,
                        listener: None,
                    })?;
                    Ok((Ok(resource),))
                }
                IpAddressFamily::Ipv6 => Ok((Err(SockErrorCode::NotSupported),)),
            },
        )?;
    }
    Ok(())
}

const P3BN: &[u8; 4] = b"P3BN";

fn patch_listen_port(wasm: &mut [u8], port: u16) {
    let idx = wasm
        .windows(4)
        .position(|w| w == P3BN)
        .expect("P3BN marker in sockets_tcp_listen.wasm");
    assert!(idx + 6 <= wasm.len(), "truncated P3BN record");
    wasm[idx + 4] = (port & 0xff) as u8;
    wasm[idx + 5] = (port >> 8) as u8;
}

fn spawn_loopback_connector(port: u16) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if let Ok(mut s) = TcpStream::connect_timeout(&addr, Duration::from_millis(100)) {
                let _ = s.write_all(b"hi");
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
    })
}

#[test]
fn bind_non_loopback_is_access_denied() {
    let err = tcp_bind_guest(IpSocketAddress::Ipv4(Ipv4SocketAddress {
        port: 80,
        address: (8, 8, 8, 8),
    }))
    .unwrap_err();
    assert!(matches!(
        sock_error_from_io(&err),
        SockErrorCode::AccessDenied
    ));
}

#[test]
fn wasi_sockets_tcp_listen_accept_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let mut bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/sockets_tcp_listen.wasm"
    ))?;
    let port = 21000u16.wrapping_add((std::process::id() % 2000) as u16);
    patch_listen_port(&mut bytes, port);
    let connector = spawn_loopback_connector(port);
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
    let _ = connector.join();
    assert_eq!(n, 1);
    Ok(())
}
