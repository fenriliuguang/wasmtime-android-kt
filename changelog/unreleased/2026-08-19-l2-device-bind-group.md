### Code — L2 gpu-device create-bind-group guest fields to host (2026-08-19)

- Deepen `[method]gpu-device.create-bind-group` from a host-fixed stub to described JNI (layout handle + optional label)
- Guest passes layout borrow and label=`l2` with empty entries; native wrap uses device/layout `rep` when non-zero and forwards them into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixture `webgpu_method_create_bind_group`; native module `create_bind_group`
