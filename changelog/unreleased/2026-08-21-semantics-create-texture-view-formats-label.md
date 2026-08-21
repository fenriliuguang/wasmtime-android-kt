### Code — leftover descriptor semantics create-texture view-formats/label (2026-08-21)

- JNI `deviceCreateTextureDescribed` now packs guest `view-formats` (empty → none) and `label` (empty → none) into existing `TextureDescriptor`
- Fixture `webgpu_method_create_texture` passes `view-formats=[rgba8unorm]` and `label=l2`; native smoke asserts both lifted fields
- Dawn `GPUTextureDescriptor` already takes `label`; `viewFormats` stay on the Kotlin record if androidx omits the ctor slot
