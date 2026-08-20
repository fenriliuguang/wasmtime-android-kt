### Code — L2 gpu-command-buffer label guest fields to host (2026-08-20)

- Deepen `[method]gpu-command-buffer.label` / `set-label` from lift-only stubs to described JNI (handle + guest label → Dawn/Cpu)
- Guest getter still uses rep 0; native wrap stub-creates when needed; export `run` returns harness `1`
- Fixtures `webgpu_method_command_buffer_label` / `webgpu_method_command_buffer_set_label`; native modules of the same names
