### Code — wasi:webgpu W3 [method]gpu.request-adapter (2026-08-14)

- Register WIT `gpu` resource + `get-gpu` + `[method]gpu.request-adapter` (true CM async; same L2 u32 as flat `request-adapter`)
- Fixture `fixtures/w1/webgpu_method_request_adapter`; native `wasi_webgpu_method_request_adapter`; twin instrument `WasiWebGpuMethodRequestAdapterInstrumentedTest`
- Transitional: method still returns u32 (not `option<gpu-adapter>`); no options record. Flat `request-adapter` kept. Not compliance
