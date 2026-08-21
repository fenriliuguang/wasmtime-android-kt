### Code — leftover descriptor semantics create-shader-module hints/label (2026-08-21)

- JNI `deviceCreateShaderModuleDescribed` now packs guest `label` (empty → none) and `compilation-hints` (empty → none; layouts `-1` none / `0` auto / `>0` specific) into existing `ShaderModuleDescriptor`
- Fixture `webgpu_method_create_shader_module` passes one hint `entry-point=l2` (layout none) and `label=l2`; native smoke asserts both lifted fields
- Dawn `GPUShaderModuleDescriptor` already takes `label`; compilation-hints stay on the Kotlin record if androidx omits them
