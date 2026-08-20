### Code — L2 gpu-compilation-message.message guest fields to host (2026-08-20)

- Deepen `[method]gpu-compilation-message.message` from omit-lane lift-only stub to described JNI with guest shader-module handle
- String getter family uses existing `call_string` / `HostArg::Str` path (no new HostArg variants)
- New host API `compilationMessageMessage` (Cpu stub `cpu-compilation-message`) on `attachCommandCompilationLabel`
