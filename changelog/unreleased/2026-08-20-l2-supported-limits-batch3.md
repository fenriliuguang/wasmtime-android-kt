### Code — L2 gpu-supported-limits batch3 guest fields to host (2026-08-20)

- Deepen `[method]gpu-supported-limits.max-compute-workgroup-size-y` / `max-compute-workgroup-size-z` / `max-compute-workgroups-per-dimension` / `max-compute-workgroup-storage-size` from lift-only stubs to described JNI
- Reuse `GpuSupportedLimits { adapter, device }` handles through `l2_supported_limits_handles` into host `supportedLimitsMaxComputeWorkgroupSizeY` family
- Cpu stubs stay 1 / 1L; Dawn reads adapter or device `limits.maxComputeWorkgroupSizeY` (and siblings)
