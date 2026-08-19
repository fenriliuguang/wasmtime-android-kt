### Code — S6+ gpu-supported-features.has + limits uniform/vertex/min getters (2026-08-19)

- Hang product `[method]` names: `gpu-supported-features.has` / `gpu-supported-limits.max-uniform-buffer-binding-size` / `max-uniform-buffers-per-shader-stage` / `max-vertex-attributes` / `max-vertex-buffer-array-stride` / `max-vertex-buffers` / `min-storage-buffer-offset-alignment` / `min-uniform-buffer-offset-alignment`
- Guest lifts WIT types via adapter features or `get-supported-limits`; export `run` returns harness `1`; L2 unused (lift-only stub `false` / `1` / `1u64`; no new JNI)
- Fixtures `webgpu_method_supported_features_has` + `webgpu_method_supported_limits_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
