### Code — S6+ compute pass/pipeline label WIT (2026-08-19)

- Hang product `[method]` names: `gpu-compute-pass-encoder.label` / `set-label` and `gpu-compute-pipeline.label` / `set-label` / `get-bind-group-layout`
- Guest lifts WIT string getters/setters or own bind-group-layout (drop); export `run` returns harness `1`; L2 unused (empty labels; no new JNI)
- Fixtures `webgpu_method_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
