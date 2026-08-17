### Code — S6+ layout / shader create cluster (2026-08-17)

- Cut four remaining `gpu-device.create-*` methods that still returned transitional `u32`: `create-shader-module`, `create-bind-group-layout`, `create-pipeline-layout`, `create-bind-group`
- Guest passes real WIT descriptors (empty shader code / empty BGL entries / empty pipeline layouts / layout borrow + empty bind-group entries); drops owns; export `run` returns harness `1`; L2 stays host-fixed stub WGSL / empty layouts
- Fixtures `webgpu_method_create_shader_module` / `webgpu_method_create_bind_group_layout` / `webgpu_method_create_pipeline_layout` / `webgpu_method_create_bind_group`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
