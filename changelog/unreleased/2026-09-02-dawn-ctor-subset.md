### Code — Dawn C ctor subset (2026-09-02)

- NativeGpu `create-render-pipeline` fills Dawn C blend / depth-stencil / MSAA / write-mask and pipeline constants. `create-compute-pipeline` constants reach `WGPUConstantEntry`.
- `request-adapter` passes feature-level / power-preference / force-fallback on `WGPURequestAdapterOptions` (still prefers Vulkan). `request-device` passes required-features and labels. `required-limits` and `xr-compatible` stay Record (no C slot).
- Cloud / missing `.so` stays table-backed.
