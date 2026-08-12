# 版本控制与协作工作流

**中文** | （暂无 EN）

> 配套 [`long-term-plan.md`](long-term-plan.md) · [`../contribute.md`](../contribute.md)。  
> 目标：可审、可回滚、可 CI；**开源后能自然接受外部 PR**。  
> 初订：2026-08-11。

## 1. 决策摘要

| 问题 | 决定 |
|------|------|
| 默认集成方式 | **`main` + 短命功能分支 + PR** |
| 是否开多条长期并行线（如常驻 `feature/stream` / `feature/webgpu`） | **否** |
| 「并行」指什么 | 同时存在 **少量短命 PR**（通常 ≤2–3），各自独立 DoD，常合主干 |
| 合并单元 | 一 PR 一事；可独立 revert |
| `main` 要求 | 始终可构建；未完成能力不靠长期分叉隐瞒 |

## 2. 为什么不用「多长期分支最后大合并」

本仓冲突热点集中：`native/` JNI、Linker 注册、线程泵、公开 Kotlin API、仪器用例。长期分叉会：

- 放大合并冲突，毁掉可审历史  
- 让外部贡献者不知以哪条线为底  
- 削弱 bisect / 按 PR 回滚能力  

战略上的多线（webgpu / stream / clocks）映射为 **多个短 PR 的排期**，不是多条永久 branch。

## 3. 分支命名与寿命

| 前缀 | 用途 | 寿命 |
|------|------|------|
| `docs/<topic>` | 纯文档 / 规划 | 合并即删；建议 &lt; 1 周 |
| `feat/<slice>` | 功能切片（如 `feat/p3-stream-read`） | 合并即删；建议 &lt; 2 周 |
| `fix/<issue>` | 缺陷 | 合并即删 |
| `chore/<topic>` | 工具链 / 追踪表刷新 | 合并即删 |

禁止：

- 无主的常驻 `feature/*` 作为第二主干  
- 在功能分支上顺手做 Wasmtime **major** 升级（须独立 PR + [`wasmtime-tracking.md`](wasmtime-tracking.md) RFC）  
- 用分支代替 feature flag 长期隐藏破坏性半成品（`0.x` 可破 API，但须可审、可 CHANGELOG）

## 4. PR 规则

1. **一 PR 一事**：例如「stream 读端 smoke」与「升 Wasmtime major」不得混装。  
2. **自带门禁证据**：至少说明跑了哪些命令；触及 native 时按 tracking 回归最低集。  
3. **文档同车**：公开行为 / 差距 / 钉版变更与代码同 PR。  
4. **CHANGELOG**：用户可见行为写入 Unreleased（或随版本节）。  
5. **合并策略（现行建议）**  
   - 开源前：维护者可 **squash merge** 到 `main`，保持线性历史。  
   - 开源后：对外部 PR 默认 squash（或 rebase 成清晰少数提交）；避免无意义的 merge 泡泡。  
6. **删除已合分支**：合并后删远程/本地功能分支。

## 5. 长期计划下的并行矩阵（排期，非常驻分支）

```text
可同时开短 PR：
  A  文档 / WIT 钉版 / gap 表 / Wasmtime 跟踪表刷新
  C  wasi:webgpu W0–W1（命名、resource、文档；future-only 可先）
  D  wasi:clocks / wasi:random 极小面（通常不依赖 stream）

须先合入再扩面：
  B  WASI 0.3 stream 原语（JNI/Kotlin）——合并优先级高于依赖 stream 的 package

逻辑串行硬闸门：
  webgpu W2（真 async adapter/device）失败 ⇒ 停 W3 扩面
  stream 未就绪 ⇒ 不开 cli stdio / 大型流式 world
```

| 切片 | 可否与其它短 PR 并行 | 备注 |
|------|----------------------|------|
| 文档 / 钉版 / tracking | **是** | 随时合 |
| `feat/…-stream-…` | **优先单线** 占有 `native/` 热点 | 合入后再开 stdio |
| webgpu W0–W1 | **是**（相对 stream / clocks） | 勿回退 sync-compat |
| webgpu W2 | 依赖 L1 future 泵稳定；与 stream **可部分并行** 但勿抢同一 JNI 文件无协调 | 闸门 |
| clocks / random | **是** | 小面 |
| cli stdio / fs 流 | **否**（等 stream） | |

同一时刻建议 **最多 2–3 个** 未合并功能 PR，避免热点腐烂。

## 6. `main` 与保护（开源就绪清单）

开源或接受外部 PR 前建议具备：

- [ ] `main` 禁止直推（**Ruleset**，非经典 Branch protection）：Settings → Rules → Rulesets；目标 Default branch；勾选 Require PR / linear history / block force push；详见 GitHub [Creating rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/creating-rulesets-for-a-repository)  
- [ ] PR 必经审查（一人维护：Required approvals = **0** 亦可；有第二人再升 1）  
- [x] CI：[`.github/workflows/ci.yml`](../../.github/workflows/ci.yml)（`cargo test` + `:runtime-api:compileKotlin`）；Ruleset 挂状态检查名 **`CI`**  
- [x] [`CONTRIBUTING.md`](../../CONTRIBUTING.md) → 本文 + [`../contribute.md`](../contribute.md)  
- [x] Issue / PR 模板：[`.github/`](../../.github/)  
- [x] 许可证：[`LICENSE`](../../LICENSE)（Apache-2.0）+ [`NOTICE`](../../NOTICE) + [`THIRD_PARTY_NOTICES.md`](../../THIRD_PARTY_NOTICES.md)  
- [x] 权限口径写入 CONTRIBUTING（协作者 Write；外人 Fork + PR）— **须在 GitHub 网页落实** Collaborators  

合入本清单对应 PR 后：在 Ruleset 中启用 **Require status checks → `CI`**，再把 Enforcement 设为 Active。

## 7. 当前仓库实操（2026-08-12）

| 动作 | 决定 |
|------|------|
| 是否新建 `feature/stream`、`feature/webgpu`、`feature/clocks` 等长期线 | **不新建** |
| 本批规划文档 PR 分支 | `docs/long-term-plan-vcs-workflow` → 合入 `main` 后删除 |
| 已推进短命切片 | `docs/w0-wasi-webgpu-gap` · `feat/p3-stream-read` · `feat/p3-stream-write` · `feat/wasi-random`（`get-random-u64`） · `feat/wasi-clocks`（`monotonic-clock.now`） · `feat/wasi-cli-stdout`（`write-via-stream` 过渡 `future<u32>`） · **`docs/webgpu-w1-dual-register`**（W1 刀切计划） |
| 可开下一刀 | cli stdin/stderr · clocks `wait-*` / system-clock · **webgpu W1 代码** `feat/webgpu-w1-…`（见 [`w1-dual-register.md`](w1-dual-register.md)）— 勿抢同一 `native/cm.rs` 无协调 |
| 已推进短命切片 | `docs/w0-wasi-webgpu-gap` · `feat/p3-stream-read` · `feat/p3-stream-write` · `feat/wasi-random`（`get-random-u64`） · `feat/wasi-clocks`（`monotonic-clock.now`） · `feat/wasi-clocks-wait-for`（`monotonic-clock.wait-for`） · **`docs/webgpu-w1-dual-register`**（W1 刀切计划） |
| 可开下一刀 | cli stdio 子集 · clocks `wait-until` / `system-clock` · **webgpu W1 代码** `feat/webgpu-w1-…`（见 [`w1-dual-register.md`](w1-dual-register.md)）— 勿抢同一 `native/cm.rs` 无协调 |

## 8. 修订

- 小修订：PR + CHANGELOG Docs。  
- 改变「禁止长期并行线」或合并策略：更新本页并在长期计划文档地图中留链。  
