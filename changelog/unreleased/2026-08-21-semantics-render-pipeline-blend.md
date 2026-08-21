### Code — leftover descriptor semantics render-pipeline blend/MSAA/cull (2026-08-21)

- JNI `deviceCreateRenderPipelineDescribed` now packs primitive (topology / strip-index / front-face / cull) + optional multisample + per-target blend 7-tuples into existing `RenderPipelineDescriptor`
- Fixture `webgpu_method_create_render_pipeline` passes `cull-mode=back`; native smoke asserts the lifted primitive
- Dawn `GPUPrimitiveState` stays topology-only this cut (androidx extra ctor params not assumed); blend/MSAA/cull stay on the Kotlin record
