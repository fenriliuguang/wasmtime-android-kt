### Code — L2 gpu-device-lost-info reason message guest fields to host (2026-08-20)

- Deepen `[method]gpu-device-lost-info.reason` / `message` from omit-lane lift-only stubs to described JNI with guest device handle
- `GpuDeviceLostInfo` stores owning device rep; `get-device-lost-info` still pushes `device: 0`
- New host APIs `deviceLostInfoReason` / `deviceLostInfoMessage` (Cpu stub unknown + `cpu-device-lost`)
