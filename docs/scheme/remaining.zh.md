# 剩余收口（跟踪）

[English](remaining.md) | **中文**

与英文冲突时以英文为准。自动队列：`GFX-SIZE`（surface 尺寸/resize）→ `GFX-PIN`（其余 wasi-gfx pin）。`BIND` 已落地。手册：[`../agent/remaining.md`](../agent/remaining.md)。

非紧急（永不 `Next:`）：`context.unconfigure`、带时间戳的 `frame-event`、Lost/Outdated `result`、多窗口。
