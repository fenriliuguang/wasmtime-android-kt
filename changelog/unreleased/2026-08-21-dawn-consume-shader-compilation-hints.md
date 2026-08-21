### Code — L2 dawn consume shader compilation-hints (2026-08-21)

- Kotlin `ShaderModuleDescriptor.compilationHints` stays populated from F5 JNI; Dawn `GPUShaderModuleDescriptor` still has no matching ctor argument
- Do not re-cut F5 JNI; fixture `webgpu_method_create_shader_module` already lifts hints + label
- androidx `1.0.0-alpha05` hole: descriptor is label + SPIR-V/WGSL only — remaining sentinel changed so G3 does not auto-repeat
