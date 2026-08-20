### Code — L2 gpu-query-set info guest fields to host (2026-08-19)

- Deepen `[method]gpu-query-set.type` / `count` / `destroy` from lift-only stubs to described JNI (query-set handle → Dawn/Cpu type, count, destroy)
- Guest `get-query-set` still uses rep 0; native wrap stub-creates an occlusion query-set of count 1 when needed; export `run` returns harness `1`
- Fixtures `webgpu_method_query_set_{type,count,destroy}`; native modules of the same names
