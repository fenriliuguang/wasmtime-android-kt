### Code — L2 gpu request-adapter guest fields to host (2026-08-20)

- Deepen `[method]gpu.request-adapter` described JNI: keep `func_wrap_concurrent` + `option`; forward `power-preference` (`0` none/`undefined`, `1` low-power, `2` high-performance) and `force-fallback-adapter` (`0` none/false, `1` true); skip string `feature-level` and `xr-compatible`
- Guest fixture still passes options none; unwired host still yields guest `none` (not a trap); export `run` returns harness `1`
- Fixture `webgpu_method_request_adapter`; native module of the same name
