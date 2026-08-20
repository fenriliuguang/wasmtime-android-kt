### Code — L2 gpu-device info getters and destroy guest fields to host (2026-08-20)

- Deepen `[method]gpu-device.features` / `limits` / `adapter-info` / `destroy` from lift-only stubs to described JNI: the guest device handle is validated (or destroyed) by the host before the local resource lift
- Guest `get-device` still uses rep 0; the wrap stub-requests adapter→device when needed; returned features/limits/adapter-info resources stay local lifts (their getters are omit-lane)
- New host APIs `deviceValidate` / `deviceDestroy`; fixtures unchanged (comment rows only)
