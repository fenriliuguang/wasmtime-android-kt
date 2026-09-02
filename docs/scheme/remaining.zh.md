# 剩余收口（跟踪）

[English](remaining.md) | **中文**

与英文冲突时以英文为准。自动队列：`BIND`（Dawn C 全绑定）→ `GFX-SIZE`（surface 尺寸/resize）→ `GFX-PIN`（其余 wasi-gfx pin）。手册：[`../agent/remaining.md`](../agent/remaining.md)。

非紧急（永不 `Next:`）：`context.unconfigure`、带时间戳的 `frame-event`、Lost/Outdated `result`、多窗口。
