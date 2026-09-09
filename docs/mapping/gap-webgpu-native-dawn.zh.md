# 差距：`wasi:webgpu` WIT ↔ NativeGpu ↔ Dawn C

[English](gap-webgpu-native-dawn.md) | **中文**

与英文冲突时以英文为准。Maven **0.1.2** 把 `--prebuilt` `libwebgpu_dawn.so` 打进 `host-dawn`。`.so` 在时：boot + cube 热路径、map/readback、bundle 录制、occlusion、adapter `GetInfo`、texture/buffer getter 走 **Dawn**。缺 `.so` 仍是 **Table**（write-buffer 影子 + 映射区间 offset/size）。Record 洞：compilation-hints、color-space、tone-mapping、`required-limits` / `xr-compatible`。仍 Table（[#317](https://github.com/fenriliuguang/wasmtime-android-kt/issues/317)）：limits 全 `1`、compilation-info、pop-error-scope 结果、uncaptured/lost、多数 label、immediates/debug、copy origin、动态 offset。`wasi-gfx` pointer/key 由 `Store.postGfxPointer` / `postGfxKey` 接线，不是 Dawn C。
