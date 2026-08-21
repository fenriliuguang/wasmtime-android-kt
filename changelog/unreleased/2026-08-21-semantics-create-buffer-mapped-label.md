### Code — leftover descriptor semantics create-buffer mapped/label (2026-08-21)

- JNI `deviceCreateBufferDescribed` now packs guest `mapped-at-creation` (`-1` none / `0` false / `1` true) and `label` (empty → none) into existing `BufferDescriptor`
- Fixture `webgpu_method_create_buffer` passes `mapped-at-creation=true` and `label=l2`; native smoke asserts both lifted fields
- Dawn `GPUBufferDescriptor` already consumes `mappedAtCreation` and `label`
