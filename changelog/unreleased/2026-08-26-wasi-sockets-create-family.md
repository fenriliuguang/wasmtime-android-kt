### Code — WASI 0.3 wasi:sockets create-tcp-socket family result (2026-08-26)

- Promote `wasi:sockets/tcp-create-socket@0.3.0#create-tcp-socket` to `func(ip-address-family) -> result<tcp-socket, error-code>`. Smoke uses `ipv4` ok. Instance names stay `tcp` / `tcp-create-socket`. No `wasmtime-wasi`
- `ipv6` returns `error-code.unknown` (loopback echo stays IPv4). `connect` still takes no address (P1-SK2)
- Native `wasi_sockets_tcp`; device `WasiSocketsTcpInstrumentedTest` still 4-byte `P3SK`
