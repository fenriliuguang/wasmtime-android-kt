### Code — S6+ texture descriptor / render-pass descriptor / map-async result (2026-08-17)

- Cut three remaining required-descriptor methods in one slice: `create-texture`, `begin-render-pass`, `map-async`
- Guest passes a real `gpu-texture-descriptor` (1×1×1 rgba8unorm render-attachment), empty `gpu-render-pass-descriptor` color-attachments, and `map-async` READ with offset/size none; drops owns; export `run` returns harness `1`; L2 stays host-fixed where needed
- Fixtures `webgpu_method_create_texture` / `webgpu_method_begin_render_pass` / `webgpu_method_buffer_map_async`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
