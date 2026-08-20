### Code — L2 gpu-query-set label guest fields to host (2026-08-20)

- Deepen `[method]gpu-query-set.label` / `set-label` from lift-only stubs to described JNI (handle + guest label → Dawn/Cpu)
- Guest getter still uses rep 0; native wrap stub-creates when needed; export `run` returns harness `1`
- Fixtures `webgpu_method_query_set_label` / `webgpu_method_query_set_set_label`; native modules of the same names
