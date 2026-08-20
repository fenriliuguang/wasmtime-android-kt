### Code — L2 queue work-done and shader compilation-info guest fields to host (2026-08-20)

- Deepen `[method]gpu-queue.on-submitted-work-done` / `[method]gpu-shader-module.get-compilation-info` from lift-only stubs to described JNI: the guest queue / shader-module handle is validated by the host; the completion future and compilation-info stay local lifts
- Guest constructors still use rep 0; the wraps stub-create queue / shader-module when needed
- New host APIs `queueValidate` / `shaderModuleValidate`
