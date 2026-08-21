### Code — L2 gpu-device color-target write-mask guest fields to host (2026-08-21)

- JNI `deviceCreateRenderPipelineDescribed` packs per-target WIT `write-mask` (`-1` = absent) onto `ColorTargetState.writeMask`; Dawn `GPUColorTargetState.writeMask` maps RGB+A 1:1 and WIT `all` (bit 4) to `ColorWriteMask.All`
- Fixture `webgpu_method_create_render_pipeline` now lifts explicit `write-mask=all`; do not re-cut F1 blend / primitive JNI
- androidx `1.0.0-alpha05` exposes the `writeMask` ctor slot (absent → Dawn All)
