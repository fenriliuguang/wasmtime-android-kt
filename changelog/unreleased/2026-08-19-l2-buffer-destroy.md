### Code — L2 gpu-buffer destroy guest fields to host (2026-08-19)

- Deepen `[method]gpu-buffer.destroy` from a lift-only stub to described JNI (buffer handle → Dawn/Cpu destroy)
- Guest `get-buffer` still uses rep 0; native wrap stub-creates a 4-byte buffer when needed; export `run` returns harness `1`
- Fixture `webgpu_method_buffer_destroy`; native module of the same name
