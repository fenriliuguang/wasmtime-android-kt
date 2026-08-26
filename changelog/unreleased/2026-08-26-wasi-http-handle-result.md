### Code — WASI 0.3 wasi:http incoming-handler handle result (2026-08-26)

- Promote guest export `wasi:http/incoming-handler@0.3.0#handle` to `async func(own<request>) -> result<own<response>, error-code>` (ok path). Root `run` still returns 200 for `callRunConcurrent`. No `wasmtime-wasi`
- Constructors stay Fixture (G-http-ctor deferred). No body / fields / `wasi:http/service` world (G-http-body)
- Native `wasi_http_handler`; device `WasiHttpHandlerInstrumentedTest` still status 200
