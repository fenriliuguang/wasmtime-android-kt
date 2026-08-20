### Code — L2 gpu-compilation-info.messages guest fields to host (2026-08-20)

- Deepen `[method]gpu-compilation-info.messages` from omit-lane empty list to described JNI with guest shader-module handle
- `GpuCompilationInfo` stores `shader_module` rep; host returns message count and cm pushes `GpuCompilationMessage` owns
- New host API `compilationInfoMessagesCount` (Cpu stub 1) on `attachCommandCompilationLabel`
