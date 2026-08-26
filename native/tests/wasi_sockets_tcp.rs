//! WASI 0.3: wasi:sockets TCP loopback echo smoke (Android subset).

use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
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
    server: Option<std::thread::JoinHandle<std::io::Result<()>>>,
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

fn tcp_loopback_pair() -> std::io::Result<(TcpStream, std::thread::JoinHandle<std::io::Result<()>>)>
{
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
    let addr = listener.local_addr()?;
    let server = std::thread::spawn(move || -> std::io::Result<()> {
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
            |accessor, (sock, _addr): (Resource<TcpSocket>, IpSocketAddress)| {
                Box::pin(async move {
                    accessor.with(|mut access| -> wasmtime::Result<()> {
                        access.data_mut().table.get(&sock)?;
                        Ok(())
                    })?;
                    let (done_tx, done_rx) = oneshot::channel::<
                        std::io::Result<(TcpStream, std::thread::JoinHandle<std::io::Result<()>>)>,
                    >();
                    std::thread::spawn(move || {
                        let _ = done_tx.send(tcp_loopback_pair());
                    });
                    let (client, server) = done_rx
                        .await
                        .map_err(|_| wasmtime::Error::msg("connect canceled"))?
                        .map_err(|e| wasmtime::Error::msg(format!("loopback connect: {e}")))?;
                    accessor.with(|mut access| -> wasmtime::Result<()> {
                        let entry = access.data_mut().table.get_mut(&sock)?;
                        entry.client = Some(client);
                        entry.server = Some(server);
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
