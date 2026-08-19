### Code — L2 gpu-buffer info getters guest fields to host (2026-08-19)

- Deepen `[method]gpu-buffer.size` / `usage` / `map-state` from lift-only stubs to described JNI (buffer handle → Dawn/Cpu size, usage bits, map-state ordinal)
- Guest `get-buffer` still uses rep 0; native wrap stub-creates a 4-byte MAP_READ|COPY_DST buffer when needed; export `run` returns harness `1`
- Fixtures `webgpu_method_buffer_{size,usage,map_state}`; native modules of the same names
