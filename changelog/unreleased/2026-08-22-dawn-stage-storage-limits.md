# Dawn consume stage-only required-limits storage keys

- `DawnWasiWebGpuHost.dawnRequiredLimits` maps guest `max-storage-buffers/textures-in-vertex-stage` and `…-in-fragment-stage` onto `GPULimits.compatibilityModeLimits` (androidx `GPUCompatibilityModeLimits` setters).
- Do not re-cut F9 `record-option-gpu-size64`; snapshot already forwards kebab-case keys.
- Fixture `webgpu_method_request_device_stage_storage_limits`: guest adds those four keys (`=4`) then `request-device`.
