### Code — WASI 0.3 wasi:sockets connect ip-socket-address (2026-08-26)

- Promote `[method]tcp-socket.connect` to `async func(ip-socket-address) -> result<_, error-code>`. Guest smoke passes ipv4 loopback. Host may ignore port and keep the echo pair. No `wasmtime-wasi`
- Instance names stay `tcp` / `tcp-create-socket`. Data plane stays stream-plus-future. Still INTERNET + helper-thread
- Native `wasi_sockets_tcp`; device `WasiSocketsTcpInstrumentedTest` still 4-byte `P3SK`
