### Code — S6+ record-gpu-pipeline-constant-value map WIT (2026-08-19)

- Hang product `[constructor]record-gpu-pipeline-constant-value` and `[method]` names: `record-gpu-pipeline-constant-value.add` / `get` / `has` / `remove` / `keys` / `values` / `entries`
- Guest lifts WIT map ops via resource constructor; export `run` returns harness `1`; L2 unused (lift-only stubs; no new JNI)
- Fixtures `webgpu_method_record_gpu_pipeline_constant_value_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
