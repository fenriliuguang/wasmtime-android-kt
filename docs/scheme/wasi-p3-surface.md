# WASI 0.3（P3）正式特性表面与优先级

**中文** | （暂无 EN）

> 配套 [`long-term-plan.md`](long-term-plan.md) **P1**。  
> 规格基准：[WASI 0.3.0](https://github.com/WebAssembly/WASI/releases/tag/v0.3.0)（2026-06-11 批准）· 概述 [wasi.dev/releases/wasi-p3](https://wasi.dev/releases/wasi-p3) · 宣布 [BA: WASI 0.3 Launched](https://bytecodealliance.org/articles/WASI-0.3)。  
> **本页是排期表面，不是「全量实现承诺」。**

## 1. 立场

WASI 0.3 已把异步从 `wasi:io` **下沉**到 Component Model：

| WASI 0.2 | WASI 0.3 |
|----------|----------|
| `resource pollable` | `future<T>` |
| `input-stream` / `output-stream` | `stream<T>`（写方向翻转） |
| `poll` / `subscribe` | runtime `await` |
| `start-foo` / `finish-foo` | `async func` |

本仓短期已验证 **future + async host import**（M2）。长期主推的是：**在 Android JNI/Kotlin 薄 L1 上，把 WASI 0.3 的正式原语与按需 package 子集做实**，服务 P0（`wasi:webgpu`）与通用 Guest。

**不做：** 以「过完 wasi-testsuite 全部 P3」或「实现全部 worlds」作为唯一成功标准。

## 2. 分层

```text
原语层（本仓 L1 主责）     async func · future · stream · 调度泵
Package 层（Host 胶水）    clocks / random / cli / filesystem / sockets / http …
World 层（组合）           wasi:cli/command · wasi:http/service · …
```

原则：原语层缺口会阻塞所有 0.3 Guest；package / world **按 Guest 与 webgpu 需求开门**。

## 3. 原语层优先级

| ID | 能力 | 相对本仓现状 | 优先级 | 备注 |
|----|------|--------------|--------|------|
| P3-PRIM-1 | `async func` host/guest | M2 已有 concurrent 注册 + `run_concurrent` | **保持 / 产品化** | 文档化线程泵；多回调并发 |
| P3-PRIM-2 | `future<T>` 创建/完成/拒绝 | M2 oneshot 路径已通 | **保持 / 扩展** | 多 future、错误完成、生命周期 |
| P3-PRIM-3 | `stream<T>` 读/写端 | **读+写 smoke 已通**（读：`StreamReader`→guest `stream.read`；写：guest `stream.write`→host `take`/`StreamConsumer`；见 `fixtures/p3`） | **保持 / 扩展** | 多 chunk、背压、错误路径仍可扩；stdio 可开 |
| P3-PRIM-4 | stream+future 完成模式 | **最小面已通**（`take` 返回 `future<u32>` 字节数；随写端 smoke） | 随 P3-PRIM-3 | 完整 WASI「stream-plus-future」错误码面另切片 |
| P3-PRIM-5 | 写方向翻转（host 消费 guest `stream`） | **smoke 已通**（`fixtures/p3/stream_write`） | **保持** | stdout / 网络 send 可据此挂 Host |
| P3-PRIM-6 | 0.2 polyfill（可选） | 未做 | 低 | 上游/runtime 可侧；不挡 P0 |

**准入：** 任一原语切片须有可复现测试（优先 Android 仪器；桌面 JVM 可辅）+ 更新 [`../mapping/threading-m2-async.md`](../mapping/threading-m2-async.md) 或后继线程文档。

## 4. Package / World 优先级

官方 0.3 核心面（摘自 wasi.dev）：

| Package / World | 本仓优先级 | 开门条件（须同时） |
|-----------------|------------|-------------------|
| **CM 原语**（上表） | **必做底座** | L1 堆叠 |
| `wasi:clocks`（`system-clock` / `wait-until` / `wait-for`） | **高** | **`monotonic-clock.now` + `wait-for` + `wait-until` + `system-clock.now` + `monotonic-clock.resolution` smoke 已通**（`fixtures/wasi/monotonic_now` · `monotonic_wait_for` · `monotonic_wait_until` · `system_now` · `monotonic_resolution`；钉 `@0.3.0`）；`system-clock.now` 为过渡 `u64` unix 秒（非官方 `instant` record）；timezone / `system-clock.resolution` 另切片 |
| `wasi:random` | **高** | **`get-random-u64` + `get-random-bytes` smoke 已通**（`fixtures/wasi/random_u64` · `random_bytes`；钉 `@0.3.0`；`get-random-bytes` 为官方 `list<u8>`，host 长度上限 4096） |
| `wasi:cli` stdio（stream+future） | **中高** | **`stdout` / `stderr.write-via-stream` + `stdin.read-via-stream` smoke 已通**（`fixtures/wasi/cli_stdout` · `cli_stderr` · `cli_stdin`；钉 `@0.3.0`；stdout/stderr 过渡 `future<u32>` 字节数；stdin 过渡 `func() -> stream<u8>`，非官方 tuple+`result`/`error-code`）；`wasi:cli/command` 另切片 |
| `wasi:cli/command`（`async run`） | **中** | stdio + clocks/random 子集可用 |
| `wasi:filesystem` | **中** | 明确 Guest 阻塞；Android 沙箱路径策略先写文档 |
| `wasi:sockets` | **中低** | 网络权限与线程模型 RFC |
| `wasi:http`（`service` / `middleware` / async `handle`） | **中低** | 非 Android 首发叙事；可桌面先 |
| `wasi:io@0.2` | **不做 0.3 版** | 包已删除；勿再实现 pollable 主路径 |
| 其它未列出提案 package | **默认不做** | 除 [`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) |

> **`wasi:webgpu` 不在「已批准 WASI 0.3 核心 package」表内**——它是 **提案**，优先级在长期计划里高于多数正式 package 的「产品叙事」，但 **工程上仍依赖原语层（尤其 async / 未来 stream）**。见专章。

## 5. 与轨 A / Host 的归属

| 工作 | 落点 |
|------|------|
| Engine 配置、linker 注册、future/stream JNI、调度泵 | **本仓** |
| 设备相关（GPU / Surface） | **轨 A L2**（或未来同源 Host），经薄回调 |
| 纯逻辑 WASI（clocks/random/cli 子集） | 优先本仓 Kotlin Host stub；可日后抽 crate/模块 |
| HTTP / sockets 完整语义 | 另开 RFC；可评估 Wasmtime `wasmtime-wasi` 能力是否经 JNI 暴露（须 Android 线程与体积审查） |

禁止：在 Rust JNI 堆业务策略；禁止为赶进度恢复 sync-compat 假异步。

## 6. 测试与合规口径

| 层级 | 用途 |
|------|------|
| 自研 fixture | 原语与最小 package 的关门证据 |
| [wasi-testsuite](https://github.com/WebAssembly/wasi-testsuite) P3 子集 | **可选回归**；选用与已实现表面相交的用例 |
| 全量 suite / 认证 | **非**近端关门 |

宣称模板：

- 可：「本仓支持 WASI 0.3 原语 X / package Y 子集（钉 WIT 0.3.0）」。  
- 不可：「完整 WASI 0.3 兼容 runtime」（在未另开合规 RFC 前）。

## 7. 修订

- 调整某一 package 优先级一档：更新本表对应**一行** + `changelog/unreleased/` 碎片。不要顺手改其它 package 行或「下一刀」总述。  
- 将某提案升为与 webgpu 同级 P0：须长期计划修订 RFC。  
