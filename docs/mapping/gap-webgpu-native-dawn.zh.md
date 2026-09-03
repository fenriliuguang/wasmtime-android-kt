# 差距：`wasi:webgpu` WIT ↔ NativeGpu ↔ Dawn C

[English](gap-webgpu-native-dawn.md) | **中文**

与英文冲突时以英文为准。Maven **0.1.2** 把 `--prebuilt` `libwebgpu_dawn.so` 打进 `host-dawn`。`.so` 在时：boot + cube 热路径以及 BIND 覆盖的 pin 方法走 **Dawn**。缺 `.so`（Cloud CI 未跑配方）仍是 **Table**。Record 洞：compilation-hints、color-space、tone-mapping；`required-limits` 与 `xr-compatible` 仍无 C 槽。`wasi-gfx` pointer/key 由 `Store.postGfxPointer` / `postGfxKey` 接线，不是 Dawn C。
