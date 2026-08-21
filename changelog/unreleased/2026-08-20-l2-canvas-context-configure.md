### Code — L2 gpu-canvas-context configure guest fields to host (2026-08-20)

- Deepen `[method]gpu-canvas-context.configure` from a lift-only stub to described JNI (`device` / Dawn `format` / WebGPU `usage`; `context == 0` allocates a canvas-context handle)
- Guest still passes format=rgba8unorm and options none; native wrap stub-creates a device when `device.rep == 0` and writes the host handle back; not a product `surface-*`
- Fixture `webgpu_method_canvas_context_configure`; native module of the same name; instrument uses described attach
