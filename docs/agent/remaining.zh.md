# Agent 手册：剩余收口

[English](remaining.md) | **中文**

与英文冲突时以英文为准。只做 `python3 ./scripts/remaining.py` 打印的 **Next:**。

1. **BIND** — Dawn C 有槽的剩余 pin 方法在 `.so` 加载时调用 `webgpu.h`。
2. **GFX-SIZE** — `height` / `width` / `request-set-size` / `on-resize`。
3. **GFX-PIN** — `on-pointer-*` / `on-key-*`。

非紧急：`unconfigure`、带时间戳的 frame-event、Lost/Outdated `result`、多窗口。
