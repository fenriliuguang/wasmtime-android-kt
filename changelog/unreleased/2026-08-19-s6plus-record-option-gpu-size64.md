### Code — S6+ record-option-gpu-size64 map take WIT types (2026-08-19)

- Hang product `[method]` names: `record-option-gpu-size64.add` / `get` / `has` / `remove` / `keys` / `values` / `entries` (plus `[constructor]record-option-gpu-size64`)
- Guest lifts WIT map types via constructor; export `run` returns harness `1`; L2 unused (lift-only stubs; no new JNI)
- Fixtures `webgpu_method_record_option_gpu_size64_*`; native modules under `wasi_webgpu_method/`; twin instruments via `attachRecordOptionGpuSize64` assert harness `1`
