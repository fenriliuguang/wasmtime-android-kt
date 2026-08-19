### Code — S6+ gpu-supported-limits max texture getters WIT batch 3 (2026-08-19)

- Hang product `[method]` names: `gpu-supported-limits.max-texture-array-layers` / `max-texture-dimension1-d` / `max-texture-dimension2-d` / `max-texture-dimension3-d`
- Guest lifts WIT numerics via `get-supported-limits`; export `run` returns harness `1`; L2 unused (lift-only stub `1`; no new JNI)
- Fixtures `webgpu_method_supported_limits_max_texture_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
