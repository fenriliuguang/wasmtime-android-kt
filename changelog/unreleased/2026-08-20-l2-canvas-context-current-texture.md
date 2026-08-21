### Code — L2 gpu-canvas-context current-texture guest fields to host (2026-08-20)

- Deepen `[method]gpu-canvas-context.get-current-texture` / `unconfigure` from lift-only stubs to described JNI (guest context handle → 1×1 texture / unconfigure store)
- Guest fixtures still construct via `get-canvas-context` (`rep` 0); host treats 0 as no-op unconfigure and a 1×1 RGBA8 texture; configured contexts use stored format/usage; not a product `surface-*`
- Fixtures `webgpu_method_canvas_context_get_current_texture` / `webgpu_method_canvas_context_unconfigure`; native modules of the same names
