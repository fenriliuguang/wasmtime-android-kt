### Code — L2 gpu-texture meta getters guest fields to host (2026-08-19)

- Deepen `[method]gpu-texture.sample-count` / `dimension` / `format` / `usage` from lift-only stubs to described JNI (texture handle → Dawn/Cpu sample-count, dimension, format, usage)
- Guest `get-texture` still uses rep 0; native wrap stub-creates a 1×1 texture when needed and maps Dawn ints back to WIT; export `run` returns harness `1`
- Fixtures `webgpu_method_texture_{sample_count,dimension,format,usage}`; native modules `texture_sample_count`, `texture_dimension`, `texture_format_get`, `texture_usage`
