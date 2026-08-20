### Code — L2 gpu-supported-limits batch6 guest fields to host (2026-08-20)

- Deepen `[method]gpu-supported-limits.max-storage-buffers-in-vertex-stage` / `max-storage-buffers-per-shader-stage` / `max-storage-textures-in-fragment-stage` / `max-storage-textures-in-vertex-stage` from lift-only stubs to described JNI
- Reuse `GpuSupportedLimits { adapter, device }` handles through `l2_supported_limits_handles` into host `supportedLimitsMaxStorageBuffersInVertexStage` family
- Cpu stubs stay 1 / 1L; Dawn reads adapter or device `limits.maxStorageBuffersInVertexStage` (and siblings)
