### Code — L2 gpu-device create-bind-group-layout guest fields to host (2026-08-19)

- Deepen `[method]gpu-device.create-bind-group-layout` from a host-fixed stub to described JNI (first entry: binding / visibility / buffer type)
- Guest passes one uniform buffer entry at binding 0 with compute visibility; native wrap uses device `rep` when non-zero and forwards the flattened entry into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixture `webgpu_method_create_bind_group_layout`; native module `create_bind_group_layout`
