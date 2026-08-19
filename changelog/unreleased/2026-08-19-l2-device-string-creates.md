### Code — L2 gpu-device string creates guest fields to host (2026-08-19)

- Deepen `[method]gpu-device.create-command-encoder` and `create-shader-module` from host-fixed stubs to described JNI (`HostArg` string: encoder label / WGSL `code`)
- Guest passes label=`l2` and compute WGSL `fn l2`; native wrap uses device `rep` when non-zero and forwards the strings into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixtures `webgpu_method_create_command_encoder` / `webgpu_method_create_shader_module`; native modules `create_command_encoder` / `create_shader_module`
