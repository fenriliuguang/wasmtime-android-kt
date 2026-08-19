### Code — L2 gpu-texture destroy and binding-view-dimension guest fields to host (2026-08-19)

- Deepen `[method]gpu-texture.destroy` and `texture-binding-view-dimension` from lift-only stubs to described JNI (texture handle → Dawn/Cpu destroy; view-dimension 0 = none)
- Guest `get-texture` still uses rep 0; native wrap stub-creates a 1×1 texture when needed; export `run` returns harness `1`
- Fixtures `webgpu_method_texture_{destroy,texture_binding_view_dimension}`; native modules of the same names
