### Code — S6+ gpu-supported-limits max getters WIT batch 2 (2026-08-19)

- Hang product `[method]` names: `gpu-supported-limits.max-dynamic-storage-buffers-per-pipeline-layout` / `max-dynamic-uniform-buffers-per-pipeline-layout` / `max-immediate-size` / `max-inter-stage-shader-variables` / `max-sampled-textures-per-shader-stage` / `max-samplers-per-shader-stage` / `max-storage-buffer-binding-size` / `max-storage-buffers-in-fragment-stage` / `max-storage-buffers-in-vertex-stage` / `max-storage-buffers-per-shader-stage` / `max-storage-textures-in-fragment-stage` / `max-storage-textures-in-vertex-stage` / `max-storage-textures-per-shader-stage`
- Guest lifts WIT numerics via `get-supported-limits`; export `run` returns harness `1`; L2 unused (lift-only stub `1` / `1u64`; no new JNI)
- Fixtures `webgpu_method_supported_limits_max_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
