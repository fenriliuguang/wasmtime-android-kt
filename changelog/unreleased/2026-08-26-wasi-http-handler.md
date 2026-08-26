### Code — WASI 0.3 wasi:http incoming-handler subset (2026-08-26)

- Official names: `wasi:http/types@0.3.0` `request` / `response` / `[constructor]` / `[method]response.status-code`; guest export `wasi:http/incoming-handler@0.3.0#handle`. **No `wasmtime-wasi`**: that crate would add WASI-http + extra `.so` size and extra host threads on Android; this cut is a thin in-process handler ABI (status 200), not a proxy server
- Subset: `handle` is `async func(own<request>) -> own<response>` (not `result` / outparam / body / fields). Not a listening socket; loopback HTTP on the wire is out of scope. INTERNET not required for this ABI smoke
- Native `wasi_http_handler`; device `WasiHttpHandlerInstrumentedTest`
