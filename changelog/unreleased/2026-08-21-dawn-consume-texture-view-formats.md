### Code — L2 dawn consume texture view-formats (2026-08-21)

- Dawn `GPUTextureDescriptor.viewFormats` now takes snapshotted `TextureDescriptor.viewFormats` (empty list → empty array)
- Do not re-cut F3 JNI; fixture `webgpu_method_create_texture` already lifts `view-formats=[rgba8unorm]`
- androidx `1.0.0-alpha05` exposes the `viewFormats` ctor slot
