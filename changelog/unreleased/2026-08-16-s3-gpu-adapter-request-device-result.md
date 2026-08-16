### Code — S3 [method]gpu-adapter.request-device returns result<own<gpu-device>, …> (2026-08-16)

- Replace transitional u32 async getter with WIT-shaped `async (borrow<gpu-adapter>, option<gpu-device-descriptor>) -> result<own<gpu-device>, request-device-error>`
- True `func_wrap_concurrent` + oneshot yield (no Latch); host table stores L2 handle in `GpuDevice.rep`
- Guest passes descriptor=none, drops the own handle on ok; export `run` returns harness `1`
- Fixture `fixtures/w1/webgpu_method_request_device`; native `wasi_webgpu_method_request_device`; twin instrument `WasiWebGpuMethodRequestDeviceInstrumentedTest`
