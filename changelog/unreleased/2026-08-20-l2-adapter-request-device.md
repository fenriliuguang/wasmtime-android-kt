### Code — L2 gpu-adapter request-device guest fields to host (2026-08-20)

- Deepen `[method]gpu-adapter.request-device` described JNI: keep `func_wrap_concurrent` + `result`; forward optional first `required-features` enum (`hasFeature` + ordinal); ignore `required-limits` and string `label`
- Guest fixture still passes descriptor none; `adapter.rep == 0` stub-creates an adapter; export `run` returns harness `1`
- Fixture `webgpu_method_request_device`; native module of the same name
