### Code — L2 gpu-render-pass-encoder label guest fields to host (2026-08-20)

- Deepen `[method]gpu-render-pass-encoder.label` / `set-label` from lift-only stubs to described JNI (handle + guest label → Dawn/Cpu)
- Guest getter still uses rep 0; native wrap stub-creates when needed; export `run` returns harness `1`
- Fixtures `webgpu_method_render_pass_label` / `webgpu_method_render_pass_set_label`; native modules of the same names
