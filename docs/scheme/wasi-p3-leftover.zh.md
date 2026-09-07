# WASI 0.3 leftover 队列

[English](wasi-p3-leftover.md) | **中文**

与英文冲突时以英文为准。`0.1.2` 之后的 **自动刀**：在 in-tree thin host 上补齐 [`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md) 的 named leftover。长分支 **`cursor/wasi-p3-leftover-b677`**：一刀一 commit；`python3 ./scripts/wasi-p3-leftover-remaining.py` 空了再开 **一个** PR 进 `main`。

NG-4 保留（不是 wasi-testsuite / 完整 0.3 宣称）。不加 `wasmtime-wasi`。不做 guest 线程（见 [`rfc-threads.md`](rfc-threads.md)）。基准测试延期。

`python3 ./scripts/wasi-p3-leftover-remaining.py` 打印 **Next: (WASI 0.3 leftover empty)**。长分支 **一个** draft PR 进 `main`；不要 merge 除非人工审过。
