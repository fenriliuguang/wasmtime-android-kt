### Code — L2 gpu-adapter-info string getters guest fields to host (2026-08-20)

- Deepen `[method]gpu-adapter-info.vendor` / `architecture` / `device` / `description` from omit-lane lift-only stubs to described JNI with guest adapter handle
- New `call_string` JNI helper and host APIs reading `GPUAdapter.info` (Cpu stubs `cpu-vendor` / `cpu-arch` / `cpu-device` / `cpu-desc`)
- Completes omit-lane `gpu-adapter-info` resource (7/7 getters described)
