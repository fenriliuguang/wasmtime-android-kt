### Code — L2 gpu-canvas-context get-configuration guest fields to host (2026-08-20)

- Deepen `[method]gpu-canvas-context.get-configuration` from a lift-only `none` to described JNI (`has` / stored `device` / Dawn `format` / WebGPU `usage` → `option<gpu-canvas-configuration-owned>`)
- Guest fixture still constructs via `get-canvas-context` (`rep` 0) so host returns `none`; configured contexts reuse the A2 store; view-formats / color-space / tone-mapping / alpha-mode stay none
- Fixture `webgpu_method_canvas_context_get_configuration`; native module of the same name
