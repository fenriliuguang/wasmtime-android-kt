### Code — L2 gpu-device render-pipeline depth-stencil leftovers guest fields to host (2026-08-21)

- JNI `deviceCreateRenderPipelineDescribed` packs stencil-front/back, stencil masks, and depth-bias onto `DepthStencilState`; Dawn `GPUDepthStencilState` copies those slots (absent mask → `0xFFFFFFFF`, absent bias → 0)
- Fixture `webgpu_method_create_render_pipeline` lifts depth24plus-stencil8 plus stencil faces / masks / bias; do not re-cut P4 begin-render-pass depth attachment
- androidx `1.0.0-alpha05` exposes stencil face / mask / bias ctor slots
