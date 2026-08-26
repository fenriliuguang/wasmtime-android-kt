### Code — WASI 0.3 wasi:sockets TCP loopback subset (2026-08-26)

- Official names: `wasi:sockets/tcp-create-socket@0.3.0#create-tcp-socket`, `wasi:sockets/tcp@0.3.0` `resource tcp-socket`, `[method]tcp-socket.connect` / `write-via-stream` / `read-via-stream`. No `wasmtime-wasi`
- Subset: create takes no `ip-address-family`; `connect` is `async func()` (always `127.0.0.1`); write/read use cli stream shapes. No UDP / listen / name-lookup. **INTERNET** is required on Android even for loopback; blocking accept/connect/read/write run on a helper thread (`func_wrap_concurrent` + oneshot), not the ART main thread
- Native `wasi_sockets_tcp`; device `WasiSocketsTcpInstrumentedTest`
