### Code — S6+ gpu-texture info WIT (2026-08-19)

- Hang product `[method]` names: `gpu-texture.width` / `height` / `depth-or-array-layers` / `mip-level-count` / `sample-count` / `dimension` / `format` / `usage` / `texture-binding-view-dimension` / `label` / `set-label`
- Guest lifts WIT numerics, enums, flags, option, and strings via `get-texture`; export `run` returns harness `1`; L2 unused (lift-only stubs; no new JNI)
- Fixtures `webgpu_method_texture_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
