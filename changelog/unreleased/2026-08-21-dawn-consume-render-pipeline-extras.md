### Code — L2 dawn consume render-pipeline extras (2026-08-21)

- Dawn `GPUPrimitiveState` takes snapshotted cull-mode / front-face / strip-index-format; `GPUColorTargetState.blend` and `GPURenderPipelineDescriptor.multisample` copy the Kotlin record
- Do not re-cut F1 JNI; fixture `webgpu_method_create_render_pipeline` already lifts `cull-mode=back`
- androidx `1.0.0-alpha05` exposes these ctor slots (absent blend / multisample → none)
