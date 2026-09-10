# NativeGpu Remaining Table 队列

[English](nativegpu-remaining.md) | **中文**

与英文冲突时以英文为准。这是 gap §3 BIND leftover 的 **独立自动队列**，**不是** WASI leftover。WASI `remaining.py` 已空；不得再把 native-dawn 标成 Closed 而跳过本表。自动刀：`python3 ./scripts/nativegpu-remaining.py`。Record 洞与 gfx 非紧急永不 `Next:`。本 PR 补上 N-LIMITS / N-COPY / N-DYNOFF（此前队列忽视）；之后一刀一 PR。缺 `.so` 仍 Table。
