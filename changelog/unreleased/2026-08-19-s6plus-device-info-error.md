### Code — S6+ gpu-device info/error WIT (2026-08-19)

- Hang product `[method]` names: `gpu-device.adapter-info` / `features` / `limits` / `label` / `set-label` / `lost` / `push-error-scope` / `pop-error-scope` / `on-uncaptured-error` and `gpu-device-lost-info.reason` / `message`
- Guest lifts WIT records, futures, streams, and error scopes; export `run` returns harness `1`; L2 unused (empty stubs; no new JNI)
- Fixtures `webgpu_method_device_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
