# `0.1.0` 宣称表（不是 CTS）

[English](claim-010.md) | **中文**

与英文冲突时以英文为准。0.1.x 产品子集；当前 GAV **`0.1.3-SNAPSHOT`** 把 `--prebuilt` `libwebgpu_dawn.so` 打进 Maven `host-dawn`。默认 NativeGpu；`.so` 在时 pin 方法走 Dawn C（map/readback 与 bundle 录制见 #317 修复）。仍 Table：compilation-info、pop-error-scope 结果、uncaptured/lost、多数 label。自动刀：NativeGpu Remaining Table [`nativegpu-remaining.md`](nativegpu-remaining.md)（`python3 ./scripts/nativegpu-remaining.py`）。WASI leftover 已空。gfx 非紧急：`unconfigure`、带时间戳的 frame-event、Lost/Outdated `result`、多窗口。设备：Vivo V2458A / 2026-09-02，全量 instruments 绿，仓外 cube >3 min、vsync ~8.33 ms。
