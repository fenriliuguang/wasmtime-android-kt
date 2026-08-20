### Code — L2 gpu-compilation-message.length guest fields to host (2026-08-20)

- Deepen `[method]gpu-compilation-message.length` from omit-lane lift-only stub to described JNI with guest shader-module handle
- Completes the scalar getter family alongside type / line-num / line-pos / offset
- New host API `compilationMessageLength` (Cpu stub 256) on `attachCommandCompilationLabel`
