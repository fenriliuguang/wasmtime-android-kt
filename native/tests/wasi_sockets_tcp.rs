//! WASI 0.3: wasi:sockets TCP loopback echo + P010 outbound (non-loopback dial).

use std::io::{Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
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

const PAYLOAD: &[u8] = b"P3SK";

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum SockErrorCode {
    #[component(name = "unknown")]
    Unknown,
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

struct TcpSocket {
    client: Option<TcpStream>,
    server: Option<thread::JoinHandle<std::io::Result<()>>>,
}

struct TcpConnected {
    client: TcpStream,
    server: Option<thread::JoinHandle<std::io::Result<()>>>,
}

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

fn tcp_loopback_pair() -> std::io::Result<(TcpStream, thread::JoinHandle<std::io::Result<()>>)> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    let addr = listener.local_addr()?;
    let server = thread::spawn(move || -> std::io::Result<()> {
        let (mut sock, _) = listener.accept()?;
        sock.set_read_timeout(Some(Duration::from_secs(2)))?;
        sock.set_write_timeout(Some(Duration::from_secs(2)))?;
        let mut buf = Vec::new();
        sock.read_to_end(&mut buf)?;
        sock.write_all(&buf)?;
        Ok(())
    });
    let client = TcpStream::connect_timeout(&addr, Duration::from_secs(2))?;
    client.set_read_timeout(Some(Duration::from_secs(2)))?;
    client.set_write_timeout(Some(Duration::from_secs(2)))?;
    Ok((client, server))
}

/// Match product `cm.rs`: loopback echo pair; non-loopback dials guest address.
fn tcp_connect_guest(addr: IpSocketAddress) -> std::io::Result<TcpConnected> {
    match addr {
        IpSocketAddress::Ipv4(a) => {
            let ip = Ipv4Addr::new(a.address.0, a.address.1, a.address.2, a.address.3);
            if ip.is_loopback() {
                let (client, server) = tcp_loopback_pair()?;
                return Ok(TcpConnected {
                    client,
                    server: Some(server),
                });
            }
            let sock_addr = SocketAddr::from((ip, a.port));
            let client = TcpStream::connect_timeout(&sock_addr, Duration::from_secs(2))?;
            client.set_read_timeout(Some(Duration::from_secs(2)))?;
            client.set_write_timeout(Some(Duration::from_secs(2)))?;
            Ok(TcpConnected {
                client,
                server: None,
            })
        }
    }
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

fn spawn_echo_on(
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
                            "echo accept timeout",
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
        sock.write_all(&buf)?;
        Ok(())
    });
    Ok((port, received, server))
}

const P3IP: &[u8; 4] = b"P3IP";

fn patch_outbound_peer(wasm: &mut [u8], ip: Ipv4Addr, port: u16) {
    let idx = wasm
        .windows(4)
        .position(|w| w == P3IP)
        .expect("P3IP marker in sockets_tcp_out.wasm");
    assert!(idx + 10 <= wasm.len(), "truncated P3IP record");
    let oct = ip.octets();
    wasm[idx + 4] = (port & 0xff) as u8;
    wasm[idx + 5] = (port >> 8) as u8;
    wasm[idx + 6] = oct[0];
    wasm[idx + 7] = oct[1];
    wasm[idx + 8] = oct[2];
    wasm[idx + 9] = oct[3];
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
        tcp.func_wrap_concurrent(
            "[method]tcp-socket.connect",
            |accessor, (sock, addr): (Resource<TcpSocket>, IpSocketAddress)| {
                Box::pin(async move {
                    accessor.with(|mut access| -> wasmtime::Result<()> {
                        access.data_mut().table.get(&sock)?;
                        Ok(())
                    })?;
                    let (done_tx, done_rx) = oneshot::channel::<std::io::Result<TcpConnected>>();
                    thread::spawn(move || {
                        let _ = done_tx.send(tcp_connect_guest(addr));
                    });
                    let connected = match done_rx
                        .await
                        .map_err(|_| wasmtime::Error::msg("connect canceled"))?
                    {
                        Ok(c) => c,
                        Err(_) => return Ok((Err(SockErrorCode::Unknown),)),
                    };
                    accessor.with(|mut access| -> wasmtime::Result<()> {
                        let entry = access.data_mut().table.get_mut(&sock)?;
                        entry.client = Some(connected.client);
                        entry.server = connected.server;
                        Ok(())
                    })?;
                    Ok((Ok::<(), SockErrorCode>(()),))
                })
            },
        )?;
        tcp.func_wrap(
            "[method]tcp-socket.write-via-stream",
            |mut store, (sock, reader): (Resource<TcpSocket>, StreamReader<u8>)| {
                let client = store
                    .data_mut()
                    .table
                    .get(&sock)?
                    .client
                    .as_ref()
                    .ok_or_else(|| wasmtime::Error::msg("tcp-socket not connected"))?
                    .try_clone()?;
                let (tx, rx) = oneshot::channel::<u32>();
                let buf = Arc::new(Mutex::new(Vec::new()));
                reader.pipe(
                    &mut store,
                    CollectConsumer {
                        buf: buf.clone(),
                        done: Some(tx),
                    },
                )?;
                let fut = FutureReader::new(&mut store, async move {
                    let _n = rx.await.unwrap_or(0);
                    let bytes = buf.lock().map(|b| b.clone()).unwrap_or_default();
                    let mut client = client;
                    match client
                        .write_all(&bytes)
                        .and_then(|_| client.shutdown(Shutdown::Write))
                    {
                        Ok(()) => Ok::<_, wasmtime::Error>(Ok::<(), SockErrorCode>(())),
                        Err(_) => Ok(Err(SockErrorCode::Unknown)),
                    }
                })?;
                Ok((fut,))
            },
        )?;
        tcp.func_wrap(
            "[method]tcp-socket.read-via-stream",
            |mut store, (sock,): (Resource<TcpSocket>,)| {
                let entry = store.data_mut().table.get_mut(&sock)?;
                let mut client = entry
                    .client
                    .as_ref()
                    .ok_or_else(|| wasmtime::Error::msg("tcp-socket not connected"))?
                    .try_clone()?;
                let mut incoming = Vec::new();
                client.read_to_end(&mut incoming)?;
                if let Some(h) = entry.server.take() {
                    let _ = h.join();
                }
                let reader = StreamReader::new(&mut store, incoming)?;
                let fut = FutureReader::new(&mut store, async move {
                    Ok::<_, wasmtime::Error>(Ok::<(), SockErrorCode>(()))
                })?;
                Ok(((reader, fut),))
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
                        server: None,
                    })?;
                    Ok((Ok(resource),))
                }
                IpAddressFamily::Ipv6 => Ok((Err(SockErrorCode::Unknown),)),
            },
        )?;
    }
    Ok(())
}

#[test]
fn tcp_loopback_pair_echo() {
    let (mut client, server) = tcp_loopback_pair().expect("pair");
    client.write_all(PAYLOAD).unwrap();
    client.shutdown(Shutdown::Write).unwrap();
    let mut incoming = Vec::new();
    client.read_to_end(&mut incoming).unwrap();
    server.join().unwrap().unwrap();
    assert_eq!(incoming, PAYLOAD);
}

#[test]
fn wasi_sockets_tcp_loopback_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/sockets_tcp.wasm"
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
    assert_eq!(v, PAYLOAD.len() as u32);
    Ok(())
}

#[test]
fn tcp_dial_non_loopback_hits_bound_peer() {
    let ip = first_non_loopback_ipv4();
    assert!(!ip.is_loopback());
    let (port, received, server) = spawn_echo_on(ip).expect("echo bind");
    let mut client =
        TcpStream::connect_timeout(&SocketAddr::from((ip, port)), Duration::from_secs(2))
            .expect("dial");
    client.write_all(PAYLOAD).unwrap();
    client.shutdown(Shutdown::Write).unwrap();
    let mut incoming = Vec::new();
    client.read_to_end(&mut incoming).unwrap();
    server.join().unwrap().unwrap();
    assert_eq!(incoming, PAYLOAD);
    assert_eq!(*received.lock().unwrap(), PAYLOAD);
}

#[test]
fn wasi_sockets_tcp_outbound_smoke() -> wasmtime::Result<()> {
    let ip = first_non_loopback_ipv4();
    assert!(!ip.is_loopback());
    let (port, received, server) = spawn_echo_on(ip).expect("echo bind");

    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let mut bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/sockets_tcp_out.wasm"
    ))?;
    patch_outbound_peer(&mut bytes, ip, port);
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
    server.join().unwrap().unwrap();
    assert_eq!(v, PAYLOAD.len() as u32);
    assert_eq!(
        *received.lock().unwrap(),
        PAYLOAD,
        "host must dial guest address (echo server saw no payload if ignore-port pair)"
    );
    Ok(())
}
