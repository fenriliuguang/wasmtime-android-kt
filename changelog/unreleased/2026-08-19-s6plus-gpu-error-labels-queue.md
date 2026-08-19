### Code — S6+ gpu canvas/error/labels/queue getters (2026-08-19)

- Hang product `[method]` names: `gpu.get-preferred-canvas-format` / `gpu.wgsl-language-features` / `gpu-error.kind` / `gpu-error.message` / `gpu-pipeline-layout.label` / `set-label` / `gpu-query-set.label` / `set-label` / `gpu-queue.label` / `on-submitted-work-done` / `set-label`
- Guest lifts WIT types via test constructors; export `run` returns harness `1`; L2 unused (lift-only stubs; `on-submitted-work-done` true CM async void; no new JNI)
- Fixtures `webgpu_method_gpu_*` / `pipeline_layout_*` / `query_set_*` / `queue_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
