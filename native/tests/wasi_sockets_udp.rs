//! WASI 0.3: wasi:sockets UDP create / send / receive (loopback sandbox).

use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

const PAYLOAD: &[u8] = b"P3UD";

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
#[allow(dead_code)]
struct Ipv4SocketAddress {
    port: u16,
    address: (u8, u8, u8, u8),
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum IpSocketAddress {
    #[component(name = "ipv4")]
    Ipv4(Ipv4SocketAddress),
}

struct UdpSocketHost {
    sock: Option<UdpSocket>,
}

struct TestHost {
    table: ResourceTable,
}

fn udp_bind_guest(addr: IpSocketAddress) -> std::io::Result<UdpSocket> {
    match addr {
        IpSocketAddress::Ipv4(a) => {
            let ip = Ipv4Addr::new(a.address.0, a.address.1, a.address.2, a.address.3);
            if !ip.is_loopback() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "udp sandbox is loopback only",
                ));
            }
            UdpSocket::bind((ip, a.port))
        }
    }
}

fn udp_send_guest(
    sock: &UdpSocket,
    data: &[u8],
    remote: Option<IpSocketAddress>,
) -> std::io::Result<()> {
    let addr = match remote {
        Some(IpSocketAddress::Ipv4(a)) => {
            let ip = Ipv4Addr::new(a.address.0, a.address.1, a.address.2, a.address.3);
            if !ip.is_loopback() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "udp sandbox is loopback only",
                ));
            }
            if a.port == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "udp send remote port must be nonzero",
                ));
            }
            SocketAddr::from((ip, a.port))
        }
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "udp send needs a remote address",
            ));
        }
    };
    sock.send_to(data, addr).map(|_| ())
}

fn udp_recv_guest(sock: &UdpSocket) -> std::io::Result<(Vec<u8>, IpSocketAddress)> {
    sock.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut buf = vec![0u8; 2048];
    let (n, from) = sock.recv_from(&mut buf)?;
    buf.truncate(n);
    match from {
        SocketAddr::V4(v) => {
            let o = v.ip().octets();
            Ok((
                buf,
                IpSocketAddress::Ipv4(Ipv4SocketAddress {
                    port: v.port(),
                    address: (o[0], o[1], o[2], o[3]),
                }),
            ))
        }
        SocketAddr::V6(_) => Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "ipv6")),
    }
}

fn register(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
    {
        let mut udp = linker.instance("wasi:sockets/udp@0.3.0")?;
        udp.resource(
            "udp-socket",
            ResourceType::host::<UdpSocketHost>(),
            |mut store, rep| {
                let resource = Resource::<UdpSocketHost>::new_own(rep);
                store.data_mut().table.delete(resource)?;
                Ok(())
            },
        )?;
        udp.func_wrap(
            "[method]udp-socket.bind",
            |mut store, (sock, addr): (Resource<UdpSocketHost>, IpSocketAddress)| {
                if store.data_mut().table.get(&sock)?.sock.is_some() {
                    return Ok((Err(SockErrorCode::InvalidState),));
                }
                let (done_tx, done_rx) = std::sync::mpsc::channel();
                thread::spawn(move || {
                    let _ = done_tx.send(udp_bind_guest(addr));
                });
                let bound = match done_rx
                    .recv()
                    .map_err(|_| wasmtime::Error::msg("udp bind canceled"))?
                {
                    Ok(s) => s,
                    Err(e) => return Ok((Err(sock_error_from_io(&e)),)),
                };
                store.data_mut().table.get_mut(&sock)?.sock = Some(bound);
                Ok((Ok::<(), SockErrorCode>(()),))
            },
        )?;
        udp.func_wrap(
            "[method]udp-socket.send",
            |mut store, (sock, data, remote): (Resource<UdpSocketHost>, Vec<u8>, Option<IpSocketAddress>)| {
                if data.len() > 65507 {
                    return Ok((Err(SockErrorCode::DatagramTooLarge),));
                }
                if store.data_mut().table.get(&sock)?.sock.is_none() {
                    let (done_tx, done_rx) = std::sync::mpsc::channel();
                    thread::spawn(move || {
                        let _ = done_tx.send(udp_bind_guest(IpSocketAddress::Ipv4(
                            Ipv4SocketAddress {
                                port: 0,
                                address: (127, 0, 0, 1),
                            },
                        )));
                    });
                    let bound = match done_rx
                        .recv()
                        .map_err(|_| wasmtime::Error::msg("udp implicit bind canceled"))?
                    {
                        Ok(s) => s,
                        Err(e) => return Ok((Err(sock_error_from_io(&e)),)),
                    };
                    store.data_mut().table.get_mut(&sock)?.sock = Some(bound);
                }
                let cloned = store
                    .data_mut()
                    .table
                    .get(&sock)?
                    .sock
                    .as_ref()
                    .ok_or_else(|| wasmtime::Error::msg("udp-socket missing"))?
                    .try_clone()?;
                let (done_tx, done_rx) = std::sync::mpsc::channel();
                thread::spawn(move || {
                    let _ = done_tx.send(udp_send_guest(&cloned, &data, remote));
                });
                match done_rx
                    .recv()
                    .map_err(|_| wasmtime::Error::msg("udp send canceled"))?
                {
                    Ok(()) => Ok((Ok::<(), SockErrorCode>(()),)),
                    Err(e) => Ok((Err(sock_error_from_io(&e)),)),
                }
            },
        )?;
        udp.func_wrap(
            "[method]udp-socket.receive",
            |mut store, (sock,): (Resource<UdpSocketHost>,)| {
                let cloned = match store.data_mut().table.get(&sock)?.sock.as_ref() {
                    Some(s) => s.try_clone()?,
                    None => return Ok((Err(SockErrorCode::InvalidState),)),
                };
                let (done_tx, done_rx) = std::sync::mpsc::channel();
                thread::spawn(move || {
                    let _ = done_tx.send(udp_recv_guest(&cloned));
                });
                match done_rx
                    .recv()
                    .map_err(|_| wasmtime::Error::msg("udp receive canceled"))?
                {
                    Ok((bytes, from)) => Ok((Ok((bytes, from)),)),
                    Err(e) => Ok((Err(sock_error_from_io(&e)),)),
                }
            },
        )?;
    }
    {
        let mut create = linker.instance("wasi:sockets/udp-create-socket@0.3.0")?;
        create.resource(
            "udp-socket",
            ResourceType::host::<UdpSocketHost>(),
            |mut store, rep| {
                let resource = Resource::<UdpSocketHost>::new_own(rep);
                store.data_mut().table.delete(resource)?;
                Ok(())
            },
        )?;
        create.func_wrap(
            "create-udp-socket",
            |mut store, (family,): (IpAddressFamily,)| match family {
                IpAddressFamily::Ipv4 => {
                    let resource = store.data_mut().table.push(UdpSocketHost { sock: None })?;
                    Ok((Ok(resource),))
                }
                IpAddressFamily::Ipv6 => Ok((Err(SockErrorCode::NotSupported),)),
            },
        )?;
    }
    Ok(())
}

const P3UD: &[u8; 4] = b"P3UD";

fn patch_echo_port(wasm: &mut [u8], port: u16) {
    let idx = wasm
        .windows(4)
        .position(|w| w == P3UD)
        .expect("P3UD marker in sockets_udp.wasm");
    assert!(idx + 6 <= wasm.len(), "truncated P3UD record");
    wasm[idx + 4] = (port & 0xff) as u8;
    wasm[idx + 5] = (port >> 8) as u8;
}

fn spawn_loopback_echo(port_hint: u16) -> (u16, thread::JoinHandle<()>) {
    let sock = UdpSocket::bind((Ipv4Addr::LOCALHOST, port_hint))
        .or_else(|_| UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)))
        .expect("udp echo bind");
    let port = sock.local_addr().expect("local addr").port();
    sock.set_read_timeout(Some(Duration::from_secs(2)))
        .expect("echo timeout");
    let handle = thread::spawn(move || {
        let mut buf = [0u8; 64];
        if let Ok((n, from)) = sock.recv_from(&mut buf) {
            let _ = sock.send_to(&buf[..n], from);
        }
    });
    (port, handle)
}

#[test]
fn bind_non_loopback_is_access_denied() {
    let err = udp_bind_guest(IpSocketAddress::Ipv4(Ipv4SocketAddress {
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
fn send_non_loopback_is_access_denied() {
    let sock = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let err = udp_send_guest(
        &sock,
        PAYLOAD,
        Some(IpSocketAddress::Ipv4(Ipv4SocketAddress {
            port: 53,
            address: (8, 8, 8, 8),
        })),
    )
    .unwrap_err();
    assert!(matches!(
        sock_error_from_io(&err),
        SockErrorCode::AccessDenied
    ));
}

#[test]
fn wasi_sockets_udp_send_receive_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let mut bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/sockets_udp.wasm"
    ))?;
    let hint = 22000u16.wrapping_add((std::process::id() % 2000) as u16);
    let (port, echo) = spawn_loopback_echo(hint);
    patch_echo_port(&mut bytes, port);
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
    let _ = echo.join();
    assert_eq!(n, PAYLOAD.len() as u32);
    Ok(())
}
