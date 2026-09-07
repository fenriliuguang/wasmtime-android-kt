# RFC：Guest 并发 /「线程」

[English](rfc-threads.md) | **中文**

**状态：Draft**（2026-09-05）。只讨论设计，本分支不写实现。与英文冲突时以英文为准。

产品政策仍以 [`rfc.md`](rfc.md) 为准，本页未接受前不是政策。

- 仓库里已有宿主线程（GpuThread、每 Store 一个 `run_concurrent`、sockets/HTTP helper、8MiB CM pump）。不要重切。
- Guest 还不能在 `BLOCKED` 的 CM `stream.read` 上挂起；`on-frame` 是同步 `func`；未开 Wasmtime stackful CM async。这才是「支持 thread」的缺口。
- **不是** `wasi:threads` / wasm shared-memory / pthread guest（选项 D，拒绝）。
- 选项：**A** 维持现状并写清契约；**B**（草案倾向）在 Android 上打开 stackful CM async，须先回答 ART attach、GpuThread 独占、hitch、体积；**C** 多 Store 已合法，给不了单 guest 多任务。
- 接受本 RFC 后才改正文 [`rfc.md`](rfc.md) / [`threading-android.md`](../mapping/threading-android.md)。B 另开 named 实现队列，不并进 leftover `L-*`。
- 不做：WASI leftover、`wasmtime-wasi`、基准测试、外仓 example。
