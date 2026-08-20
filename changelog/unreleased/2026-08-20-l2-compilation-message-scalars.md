### Code — L2 gpu-compilation-message scalar getters guest fields to host (2026-08-20)

- Deepen `[method]gpu-compilation-message.type` / `line-num` / `line-pos` / `offset` from omit-lane lift-only stubs to described JNI with guest shader-module handle
- `GpuCompilationMessage` stores the owning shader-module rep; `get-compilation-message` still pushes `shader_module: 0` and getters stub-request a shader module when needed
- New host APIs `compilationMessageType` / `LineNum` / `LinePos` / `Offset` on `attachCommandCompilationLabel`
