# RFC：结束双轨并行，靠拢官方 wasi:webgpu 形状

**状态：Accepted（本 PR）** · 2026-08-16  
**中文** | （暂无 EN）

> 取代「两仓并行推进、本仓 Guest ABI 跟随轨 A experimental 扁平面」的排期。  
> 配套修订：[`dual-track.md`](dual-track.md) · [`long-term-plan.md`](long-term-plan.md) · [`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) · [`tech-stack.md`](tech-stack.md) · [`non-goals.md`](non-goals.md)。

## 1. 决策摘要

| 问题 | 决定 |
|------|------|
| 还和轨 A 并行排期吗？ | **否。** 本仓是 wasi:webgpu / WASI 0.3 的唯一推进面。 |
| 轨 A 现在是什么？ | **展示用简单 Demo**（experimental CM cube + wasmtime4j + sync-compat）。不再要求本仓切片对齐它的扁平面。 |
| 本仓 Guest 朝哪靠？ | 钉版 **`wasi:webgpu@0.3.0-rc.2`** 的 WIT **形状**（resource / `[method]` / `async func` / record · list · option · result）。 |
| 编码怎么规范化？ | Guest 所见走 **Component Model 规范 lowering**。禁止再把「host 固定 descriptor + 过渡 u32 / void」当新切片的目标形态。 |
| 现有过渡 `[method]` 怎么办？ | **冻结扩面**：可保留作回归，直到被规范形状切片替换。不再新开 host-fixed u32 刀。 |
| Dawn / GPU Host？ | **不重造 Dawn。** 可继续把轨 A `host-api` / `host-webgpu` 当 **后端库** 调用；**ABI 与编组由本仓拥有**，不等轨 A 先改扁平名。 |
| 还宣称合规吗？ | **否**（NG-5 仍有效）。靠拢形状 ≠ 过 CTS / 合规产品。 |

## 2. 为什么改

W0–W3 的过渡策略已经完成它的历史任务：Guest 能 import 提案 **名字**，L1 真 async 泵过闸，仪器能跑。

它同时把本仓锁在一条 **非标准形状** 上：

- 返回 `u32` 而不是 `own<gpu-adapter>` / `own<gpu-queue>`
- descriptor / `list` / `option` / `result` 由 host 固定或整段忽略
- `experimental:webgpu-cm@0.8.0` 与 `wasi:webgpu` **双注册**，产品坐标含糊
- 切片节奏绑在「轨 A L2 已有某扁平回调」上，而不是绑在 WIT

轨 A 继续并行只会放大分叉：一边锁死 sync-compat + experimental cube，一边要官方 wasi:webgpu。从本 PR 起，**只推进本仓**。

## 3. 轨 A 的新角色

`wasi-webgpu-jvm-mvp`：

- **是**：可演示的 experimental WebGPU cube；Dawn / Cpu Host 的现成后端；历史补丁经验来源。
- **不是**：本仓的 ABI 源、切片看板、或「先改 L2 再跟」的上游。
- **不要求**：为本仓改 sync-compat 锁死条款、迁 Linker、或把 Demo 默认切到本仓 runtime（NG-1 / NG-10 精神仍在：不静默替换它的 Demo；也不再让它挡本仓形状）。

本仓仪器 **不再**以「与轨 A cube 对等」为扩面门禁。轨 A cube 回归只在明确要证明「没碰 4j Demo」时才跑。

## 4. 本仓的新角色

Android-first JVM Component 运行时，Guest 主坐标：

```text
wasi:webgpu/webgpu@0.3.0-rc.2
```

引擎仍是官方 Wasmtime；WASI 0.3 正式面（clocks / random / cli …）优先级不变。  
webgpu 的 **Guest 形状** 以提案 WIT 为准，不以 `experimental:webgpu-cm` 为准。

## 5. 「形状」定义（硬）

对照钉版 `wit/webgpu.wit`（tag `v0.3.0-rc.2`）。一条方法算「形状合格」当且仅当：

| 维 | 合格 | 不合格（W3 过渡，冻结） |
|----|------|-------------------------|
| 名字 | `[method]gpu-*.…` 与 WIT 一致（含 `write-buffer-with-copy` 等正式名） | 扁平 `request-adapter`、experimental 名、或 WIT 名但参数被砍掉 |
| self | `borrow<resource>` | 无 self / 另造 `get-*` 空 resource 当唯一入口且方法仍吃 u32 |
| 返回 | `own<resource>` / `option` / `result` / `list` / `string` / void，按 WIT | 一律 `u32` 或忽略返回 |
| 入参 | Guest 传入的 record / list / option 参与编组 | host 固定 descriptor；Guest 参数全部 `_` 丢掉 |
| async | WIT `async func` → `func_wrap_concurrent` + 真 yield | sync wrap / Latch 冒充 |

`get-gpu` / `get-device` 这类 **测试用构造器** 可以暂时留在仪器 Guest 里，但 **不得**写进产品 WIT 表面；规范切片要用提案入口（从 `gpu.request-adapter` 链下去）。

## 6. 编码规范化

### 6.1 原则

1. **官方 lowering**：用 Wasmtime component 类型（`Resource<T>`、`Option`、`Result`、`Vec<u8>` / `list`、record 结构体），不要再发明「全 u32 + Kotlin 里写死」。  
2. **有限集、显式**：只为当前切片用到的 WIT 类型写编解码；禁止无 schema 的 JSON。  
3. **rep 仍可 u32**：L2 / Dawn 句柄继续 u32；**Guest 边界**必须是 resource，native 表负责 `Resource ↔` 后端句柄。  
4. **string / list**：走 CM canonical ABI（指针+长度或 Wasmtime 提供的 list 类型），禁止「host 再塞 4 字节」。  
5. **result / option**：错误走 WIT `result<_, *-error>`；缺省走 `option`。不允许用「成功才返回 u32、失败 panic」冒充。

### 6.2 编组落地（本仓）

| 层 | 职责 |
|----|------|
| `native/` Linker | 按 WIT 注册 `[method]`；`func_wrap` / `func_wrap_concurrent` 的 Rust 签名与 WIT 同构 |
| 编解码模块（新，随 S1 起） | record / enum / flags / option / result / list 的有限集 lowering；每刀只加本切片类型 |
| Kotlin L2 回调 | 只接 **已经解码完** 的结构化参数（或后端句柄）；不再靠「JNI 忽略 Guest」 |
| Guest | `wit-bindgen` 或手写 **与 WIT 同构** 的 component；仪器 fixture 必须能代表真实 import 形状 |

tech-stack §2.1 以本节约束为准。

## 7. 对现有过渡面的处置

| 资产 | 处置 |
|------|------|
| `experimental:webgpu-cm@0.8.0` 扁平面 | **遗留**。不为它加新函数。M4 smoke / 轨 A 联调路径可暂时保留。 |
| W1 双注册的过渡扁平 `wasi:webgpu#request-adapter` 等 | **遗留**。规范切片改挂真正 `[method]` 形状后，扁平名可在独立 chore 里摘掉。 |
| 已挂的过渡 `[method]`（返回 u32 / void、host-fixed） | **冻结**。只在对应 S 切片里 **替换签名与 Guest**，不要平行再挂一个假名。 |
| `get-*` 测试构造器 | 仪器可暂时保留；S 切片的产品路径必须能从 `gpu` 资源链下去。 |

**禁止（本 RFC 生效后）：** 再开「W3+ 挂一个 host-fixed `[method]`、Guest 传 stub u32」的 PR。

## 8. Host / Dawn 边界

```text
Guest (wasi:webgpu WIT)
  → 本仓 L1（官方 Wasmtime + 规范编组）
    → 后端：轨 A host-api / host-webgpu（Dawn / Cpu）或日后等价实现
      → GPU
```

- **NG-7 修订口径**：不在本仓重写 Dawn。允许、也鼓励把轨 A Host **当库**用。  
- **不再**「轨 A 先加扁平回调，本仓再挂同名」。本仓缺能力时：在本仓编组层提需求；若后端缺实现，**本仓 PR 可以扩后端调用**，或在轨 A 仓提 **仅 Host 能力** 的补丁——那是库维护，不是双轨并行产品线。  
- 呈现：提案仍无 present。近端可继续用 experimental `surface-*` 做 **Demo 上屏**；产品形状走 `gpu-canvas-context`（W4 策略页改为「遗留上屏 / 规范 canvas」分层，见该页修订）。

## 9. 切片硬序（S 系列）

活状态仍在 GitHub Project；**本页只定硬序与 DoD**。W0–W2 史实保留；W3 过渡史诗 **收口不再扩**。

```text
S0  本 RFC（文档）——本 PR
    结束双轨并行；定义形状与编组；冻结 host-fixed 扩面

S1  编组脊柱 + 第一个规范返回类型
    Linker 能把 `[method]gpu-device.queue` 做成
      (borrow<gpu-device>) -> own<gpu-queue>
    仪器 Guest 按 WIT 形状 import；禁止再返回过渡 u32
    可暂时保留 get-device 仅用于该刀夹具，但方法本身必须是 own<gpu-queue>

S2  第一个规范 async + option
    `[method]gpu.request-adapter`：
      async (borrow<gpu>, option<gpu-request-adapter-options>) -> option<own<gpu-adapter>>
    真 concurrent；禁止 Latch

S3  第一个规范 result
    `[method]gpu-adapter.request-device`：
      async (..., option<gpu-device-descriptor>) -> result<own<gpu-device>, request-device-error>

S4  第一个规范 record 入参
    `[method]gpu-device.create-buffer`：Guest 传入 `gpu-buffer-descriptor`，返回 `own<gpu-buffer>`

S5  第一个规范 list
    `[method]gpu-queue.submit`：`list<borrow<gpu-command-buffer>>`
    以及（可后一刀）`get-mapped-range-get-with-copy` → `result<list<u8>, …>`

S6+ 按 WIT 高频路径替换其余已冻结过渡方法
    （encoder / pass / map-async 正式 result 等）
    一律先形状、再语义加深；每刀独立可 revert
```

**每刀 DoD（S1 起）：**

1. Guest import 的函数类型与钉版 WIT **同构**（可用 `wasm-tools` 对照 / 手写 wat 注释引用 WIT 行）。  
2. 触及 `native/`：`cargo test --locked --tests` 绿；新测试只加 `native/tests/*.rs`。  
3. 有仪器孪生或书面说明为何本刀仅 native。  
4. 文档只改本切片行；changelog 碎片。  
5. **不**新加 experimental 扁平名；**不**宣称合规。

**硬闸门：** S2 若不能真 async，停止扩 option/result 表面，先修泵（同 W2）。

## 10. 沟通口径

| 场合 | 说法 |
|------|------|
| 谈本仓 | Android JVM 上推进 **官方形状的 wasi:webgpu（提案）** + WASI 0.3，引擎为官方 Wasmtime |
| 谈轨 A | **展示用 Demo**（experimental cube / sync-compat）；不是本仓上游 |
| 谈合规 | 仍 **不是** 合规 wasi:webgpu 产品 |
| 谈「已实现某方法」 | 必须能回答：Guest 看到的是 WIT 类型，还是过渡 u32 |

## 11. 非目标（本 RFC 增量）

仍有效：NG-2、NG-3、NG-4、NG-5、NG-6、NG-8、NG-9、NG-11。

| ID | 修订 |
|----|------|
| NG-1 | 不静默把轨 A **Demo 默认 runtime** 换成本仓。轨 A 可以继续当展示。 |
| NG-7 | 不重造 Dawn。**允许**本仓拥有 WIT 编组并把轨 A Host 当后端。 |
| NG-10 | 不要求轨 A 为了本仓破坏 sync-compat。本仓也不再等轨 A 扁平 ABI。 |

新增：**NG-12** — 本 RFC 生效后，禁止再以「host-fixed + 过渡 u32」作为 wasi:webgpu **新切片的验收形态**。

## 12. 本 PR 范围

- 本 RFC + 方案索引 / 长期计划 / 路线图 / 双轨页 / tech-stack 编组 / non-goals / 根 README 口径对齐  
- **不**改 `native/`、fixtures、仪器代码  

合入 `main` 后，下一刀就是 **S1**（`gpu-device.queue` → `own<gpu-queue>`），独立 `feat/` PR。
