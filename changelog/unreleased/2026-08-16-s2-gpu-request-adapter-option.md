### Code — S2 [method]gpu.request-adapter returns option<own<gpu-adapter>> (2026-08-16)

- Replace transitional u32 async getter with WIT-shaped `async (borrow<gpu>, option<gpu-request-adapter-options>) -> option<own<gpu-adapter>>`
- True `func_wrap_concurrent` + oneshot yield (no Latch); host table stores L2 handle in `GpuAdapter.rep`
- Guest passes options=none, drops the own handle; export `run` returns harness `1`
- Fixture `fixtures/w1/webgpu_method_request_adapter`; native `wasi_webgpu_method_request_adapter`; twin instrument `WasiWebGpuMethodRequestAdapterInstrumentedTest`
