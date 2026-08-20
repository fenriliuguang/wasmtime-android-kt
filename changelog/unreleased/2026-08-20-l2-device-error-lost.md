### Code — L2 gpu-device error-scope and lost guest fields to host (2026-08-20)

- Deepen `[method]gpu-device.push-error-scope` / `pop-error-scope` / `lost` / `on-uncaptured-error` from lift-only stubs to described JNI (device handle + filter ordinal → Dawn/Cpu; pop returns 0 = none)
- Guest `get-device` still uses rep 0; the wrap stub-requests adapter→device when needed; the lost future and uncaptured-error stream stay local lifts
- New host APIs `devicePushErrorScope` / `devicePopErrorScope`; Cpu tracks an error-scope depth
