# 路线图：主推 `wasi:webgpu` 提案

**中文** | （暂无 EN）

> 配套 [`long-term-plan.md`](long-term-plan.md) **P0**。  
> 提案仓库：[WebAssembly/wasi-webgpu](https://github.com/WebAssembly/wasi-webgpu)（撰写时 **Phase 2**）。  
> 生态旁证：`wasi-webgpu-wasmtime`（wgpu-core Host，跟 Wasmtime 代际）· [wasi-gfx 与 wasi:webgpu 分工](https://wasi-gfx.dev/blog/posts/future-of-wasi-gfx/)。  
> 本仓背景：轨 A 因 wasmtime4j 无 future writer 无法走真 CM async；轨 B 为打开 **提案所需的 async WIT** 而存在。

## 1. 为什么是 P0

1. **触发点**：标准 / 提案 `wasi:webgpu` 大量方法是 WIT `async func`；轨 A 只能 sync-compat；本仓 M2 已证明官方 Wasmtime 路径可行。  
2. **范围清晰**：提案明确 **不做** windowing；显示面交给 `wasi-gfx` 等——与本仓「不重造 wasi-gfx / 多 window」（NG-9）一致。  
3. **Android 叙事**：真机 Dawn（轨 A L2）+ 薄 L1 是可演示、可回归的宿主路径。  
4. **提案推进**：实现反馈、WIT 钉版、CTS 子集、与上游/Wasmtime 宿主对齐，比空泛「多挂 WASI package」更符合本仓使命。

## 2. 目标与非目标

### 2.1 目标

| ID | 目标 |
|----|------|
| WG-1 | 钉一份提案 WIT 坐标（版本 / commit / RC 标签），与 Guest 工具链一致 |
| WG-2 | 经本仓 L1 **真 CM async** 注册提案中的关键 `async func`（禁止 sync-compat 冒充） |
| WG-3 | Host 实现继续走 **轨 A L2**（Dawn / Cpu）；本仓只做 linker / resource / async 边界 |
| WG-4 | Android 仪器主链：`requestAdapter` / `requestDevice`（或提案等价名）→ 可观测成功 |
| WG-5 | 向提案 / 生态提供可引用的差距与线程契约（本仓 `docs/mapping`） |
| WG-6 | （中期）渲染或 compute 切片；明确与 `wasi-gfx` 边界（上屏仍可暂用轨 A experimental surface 路径） |

### 2.2 非目标（本路线图）

- 宣称 **合规 wasi:webgpu 产品** 或通过完整 WebGPU CTS（未另开合规 RFC 前）  
- 在本仓实现 **第二套** GPU Host（NG-7）  
- 把 **wasi-gfx / 多 window** 升为与 webgpu 同级短期目标（NG-9）  
- 静默替换轨 A cube 主验收（NG-1）  
- 以桌面 `wasi-webgpu-wasmtime`+wgpu **替换** Android Dawn 主路径（可作对照实验，不作门禁替换）

## 3. 与现状的差距（起点）

| 现状（M3–M4 归档） | 提案方向 |
|--------------------|----------|
| `experimental:webgpu-cm@0.8.0` 扁平 import 名 | 提案 package / interface 分层 WIT |
| 子集：adapter/device/queue/surface/clear/present | 完整设备 / 资源 / 编解码面远大于子集 |
| 部分路径仍可能受 L2 sync-compat 影响 | 目标：Guest 所见为原生 async；L2 内部可逐步去锁 |
| 上屏用专用 smoke Guest | 长期对齐提案 Guest；上屏边界或需 wasi-gfx / 平台 surface 胶水 |
| 差距史实 | [`../mapping/gap-m4-vs-cube.md`](../mapping/gap-m4-vs-cube.md)（experimental 坐标） |

**W0 交付：** [`../mapping/gap-experimental-vs-wasi-webgpu.md`](../mapping/gap-experimental-vs-wasi-webgpu.md) — 函数级对照表（experimental 名 ↔ 提案 WIT 名 ↔ async/sync ↔ L2 有无）；提案钉 `wasi:webgpu@0.3.0-rc.2`。

## 4. 切片堆叠（建议序）

活状态（Todo / In Progress / Done）见 GitHub Project [wasmtime-android-kt progress](https://github.com/users/fenriliuguang/projects/1)，**不要**在本页枚举每一刀。本节只定切片**定义与硬序**。

```text
W0  钉版与差距表（文档）
    提案 WIT 版本 · 与 Wasmtime / wit-bindgen 对齐说明 · gap 表

W1  链接与 resource 边界（已交付）
    提案 instance 双注册过渡扁平 `request-adapter` → 同一 L2 sync u32
    见 [`w1-dual-register.md`](w1-dual-register.md)；终态 `[method]` / resource 表属 W3

W2  真 async 主链（硬闸门；adapter + device 已交付）
    request-adapter / adapter-request-device（提案名过渡扁平）`func_wrap_concurrent` + 仪器 `callRunConcurrent`
    禁止 Latch 冒充；非合规宣称；`[method]` 终态名属 W3

W3  队列与缓冲关键面（resource 面史诗仍在进行）
    `device-get-queue` 过渡扁平 sync 已交付；**`[method]gpu-device.queue`** sync 已交付（仍 u32，非 `gpu-queue` resource）
    `device-create-command-encoder` 过渡扁平 sync 已交付；**`[method]gpu-device.create-command-encoder`** sync 已交付（仍 u32，无 descriptor）
    `command-encoder-begin-render-pass-clear` 过渡扁平 sync 已交付；**`[method]gpu-command-encoder.begin-render-pass`** sync 已交付（stub view `23`）
    `render-pass-end` 过渡扁平 sync 已交付；**`[method]gpu-render-pass-encoder.end`** sync void 已交付
    `command-encoder-finish` 过渡扁平 sync 已交付；**`[method]gpu-command-encoder.finish`** sync 已交付
    `queue-submit1` 过渡扁平 sync 已交付；**`[method]gpu-queue.submit`** sync 已交付（单 buffer u32，非提案 `list`）
    **W3+** `[method]gpu-device.create-buffer` sync 已交付（host 固定 descriptor，仍 u32）
    **W3+** `[method]gpu-device.create-texture` sync 已交付（host 固定 1×1，仍 u32）
    按差距表选高频 async/sync 方法切片；每片独立 DoD

W4  呈现路径策略（选型已立；文档）
    近端默认选项 A：继续轨 A experimental surface（过渡）
    选项 B：引入 wasi-gfx 最小胶水（须单独 RFC，默认不升 P0；= DG-6 / NG-9）
    选项 C：headless compute-only 演示（后期可选，不替换 A）
    见 [`w4-present-strategy.md`](w4-present-strategy.md)

W5  提案反馈与可选 CTS 子集
    文档化 Android/Dawn 特有问题；可选上游 issue；CTS 子集不挡 W2
```

**硬闸门：** W2 失败 ⇒ 停止扩大 WIT 表面，先修 L1 async / 线程泵（与当年 M2 同构）。

## 5. 依赖关系

```text
Wasmtime 追踪（版本含 CM async / 必要时 stream）
    → WASI 0.3 原语（尤其 future；stream 若提案缓冲需要）
        → 本路线图 W0–W2
            → 轨 A L2 接口跟随（轨 A 先改，本仓跟）
```

上游生态对照（**非**运行时依赖）：

| 资产 | 用法 |
|------|------|
| `wasi-webgpu` WIT | 钉版来源 |
| `wasi-webgpu-wasmtime` | API 形状 / 宿主经验对照；**不**作 Android 默认 `.so` |
| 轨 A `host-api` / `host-webgpu` | **实际** Android Host |
| 轨 A `guest/cube-cm` | 过渡 Guest；逐步换提案坐标 Guest |

## 6. 沟通口径

| 场合 | 说法 |
|------|------|
| 谈本仓使命 | Android JVM 上推进 **wasi:webgpu（提案）** + WASI 0.3 原语，底座为官方 Wasmtime |
| 谈轨 A Demo | 仍是 experimental CM cube + sync-compat |
| 谈合规 | 未宣布前 **不是** 合规 wasi:webgpu 实现 |
| 谈 wasi-gfx | 显示 / window 另册；本仓不把它当近端 P0 |

## 7. 修订

- W 切片增删、WIT 钉版变更：更新本页对应节 + `changelog/unreleased/` 碎片 +（若有）gap 表。不要为「下一刀」去改根 README 或 `vcs-workflow` 清单；活状态改 Project 卡片。  
- 将 wasi-gfx 升为与本页同级：长期计划修订 RFC。  
- W4 呈现路径已书面选型（近端 A）：见 [`w4-present-strategy.md`](w4-present-strategy.md)。  
