### Code — L2 gpu-supported-limits batch8 guest fields to host (2026-08-20)

- Deepen `[method]gpu-supported-limits.max-texture-dimension3-d` / `max-uniform-buffer-binding-size` / `max-uniform-buffers-per-shader-stage` / `max-vertex-attributes` from lift-only stubs to described JNI
- Reuse `GpuSupportedLimits { adapter, device }` handles through `l2_supported_limits_handles` into host `supportedLimitsMaxTextureDimension3D` family
- Cpu stubs stay 1 / 1L; Dawn reads adapter or device `limits.maxTextureDimension3D` (and siblings)
