### Code — L2 gpu-device create-query-set and bundle-encoder guest fields to host (2026-08-20)

- Deepen `[method]gpu-device.create-query-set` / `create-render-bundle-encoder` from lift-only stubs to described JNI (query type ordinal + count; first color format Dawn int + sample count → host create; returned reps stored on the guest resources)
- Guest `get-device` still uses rep 0; the wrap stub-requests adapter→device when needed; export `run` returns harness `1`
- New host API `deviceCreateRenderBundleEncoder`; Cpu gains a RenderBundleEncoder handle kind
