### Code — S6+ remaining device create and destroy WIT (2026-08-19)

- Hang remaining device create + destroy product `[method]` names: `create-render-bundle-encoder` / `create-query-set` / `gpu-device.destroy` / `gpu-buffer.destroy` / `gpu-texture.destroy` / `gpu-query-set.destroy` / `gpu-query-set.type` / `gpu-query-set.count`
- Guest lifts WIT descriptors (empty color-formats / occlusion query-set) and destroy borrows; export `run` returns harness `1`; L2 unused except host-fixed `query-set.type` = occlusion and `count` = 1 (lift-only, no new JNI)
- Fixtures `webgpu_method_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
