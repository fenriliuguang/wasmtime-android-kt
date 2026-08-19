### Code — L2 gpu-texture create-view guest fields to host (2026-08-19)

- Deepen `[method]gpu-texture.create-view` from host-fixed default view to described JNI (`textureCreateViewDescribed`)
- Guest passes `some(gpu-texture-view-descriptor)` with dimension=d2 and aspect=all; native wrap forwards Dawn ints into `WasiWebGpuHost`; drops own; export `run` returns harness `1`
- Fixture `webgpu_method_texture_create_view`; native module `texture_create_view`; twin instrument `WasiWebGpuMethodTextureCreateViewInstrumentedTest`
