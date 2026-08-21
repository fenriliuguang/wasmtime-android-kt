### Code — L2 create-render-pipeline vertex.buffers + format (2026-08-21)

- `[method]gpu-device.create-render-pipeline` forwards guest `vertex.buffers` (stride / step / attributes) and the first fragment **target format** through described JNI (stop hardcoding `format = 0`)
- Fixture `webgpu_method_create_render_pipeline` asserts one `float32x3` buffer (stride 12) plus fragment `rgba8unorm`; blend / multisample / constants stay dropped
- Async create-render-pipeline uses the same described JNI arrays; empty vertex.buffers stays valid
