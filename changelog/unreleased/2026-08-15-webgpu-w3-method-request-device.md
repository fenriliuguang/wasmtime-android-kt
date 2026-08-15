### Code — wasi:webgpu W3 [method]gpu-adapter.request-device (2026-08-15)

- Register WIT `gpu-adapter` resource + `get-adapter` + `[method]gpu-adapter.request-device` (true CM async; L2 `request-adapter` then `adapter-request-device`, same u32 as flat `adapter-request-device`)
- Fixture `fixtures/w1/webgpu_method_request_device`; native `wasi_webgpu_method_request_device`; twin instrument `WasiWebGpuMethodRequestDeviceInstrumentedTest`
- Transitional: method still returns u32 (not `result<gpu-device, request-device-error>`); no descriptor. Flat `adapter-request-device` kept. Not compliance
