### Code — L2 gpu-texture extent getters guest fields to host (2026-08-19)

- Deepen `[method]gpu-texture.width` / `height` / `depth-or-array-layers` / `mip-level-count` from lift-only stubs to described JNI (texture handle → Dawn/Cpu extent)
- Guest `get-texture` still uses rep 0; native wrap stub-creates a 1×1 texture when needed and returns the host value; export `run` returns harness `1`
- Fixtures `webgpu_method_texture_{width,height,depth_or_array_layers,mip_level_count}`; native modules of the same names
