### Code — L2 gpu-buffer label guest fields to host (2026-08-20)

- Deepen `[method]gpu-buffer.label` / `set-label` from lift-only stubs to described JNI (buffer handle + guest label → Dawn/Cpu)
- Guest `get-buffer` still uses rep 0; native wrap stub-creates a 4-byte MAP_READ|COPY_DST buffer when needed; export `run` returns harness `1`
- Fixtures `webgpu_method_buffer_{label,set_label}`; native modules of the same names
