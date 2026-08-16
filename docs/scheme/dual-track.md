# 与轨 A 的边界（轨 A = 展示 Demo）

**中文** | [English](dual-track.en.md)

> **2026-08-16 修订：** 结束「两仓并行推进产品线」。权威决策见 [`rfc-wasi-webgpu-canonical-shape.md`](rfc-wasi-webgpu-canonical-shape.md)。  
> 本页只保留 **边界**：轨 A 仍锁死 sync-compat；本仓不静默替换它的 Demo；Host 可当库用。  
> 本仓产品主线：[`long-term-plan.md`](long-term-plan.md) · wasi:webgpu 形状：[`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md)。

## 1. 角色（现行）

| 仓 | 路径 | 一句话 |
|----|------|--------|
| **轨 A** | `d:\projects\wasi-webgpu-jvm-mvp` | **展示用简单 Demo**：experimental CM cube + wasmtime4j + **sync-compat**。不是本仓 ABI 上游。 |
| **本仓** | `d:\projects\wasmtime-android-kt` | Android-first JVM Component 运行时；**唯一**推进官方 `wasi:webgpu` Guest 形状的地方。引擎 = 官方 Wasmtime。 |

不再排期「轨 A 先改 L2 扁平面，本仓跟随」。

## 2. 轨 A 锁死条款（仍有效）

自 2026-08-10 起，轨 A **自己的**主路径仍锁死（本仓不要求它为 wasi:webgpu 形状改 Linker）：

1. 默认与主验收路径保持 **sync-compat**。  
2. **不再**为「真 CM async」改 `DawnWasiWebGpuHost` 主回调、`WasmtimeCmLinker` 主链 future、或迁仪器到 async Guest。  
3. 真 CM async 闸门归档保持有效：[`archive-true-cm-async-dod.md`](../../../wasi-webgpu-jvm-mvp/docs/scheme/archive-true-cm-async-dod.md)。  
4. 轨 A 可继续：稳性、文档、Demo 体验、**不依赖** 4j future writer 的修补。  
5. 若未来把轨 A Demo **默认 runtime** 切到本仓，必须单独 RFC，**不得**静默替换（NG-1）。

## 3. 共享面（允许）

| 资产 | 方式 |
|------|------|
| L2 `host-api` / `host-webgpu` | 本仓以 **后端库**（mavenLocal / composite / 源码路径）调用 Dawn / Cpu；**不**复制 Dawn |
| 补丁**经验**（JNI_VERSION、TBI、resource↔u32） | 文字迁移到本仓 native；**不是**依赖 4j `.so` |
| Guest `cube_cm.wasm` | 仅 Demo / 遗留 smoke；**不是**规范形状门禁 |
| 映射文档 | 本仓 `docs/mapping` 自洽；可链到轨 A 史实 |

**不再共享为「同源 ABI」：** experimental 扁平函数名、过渡 u32 返回值。Guest 形状由本仓按钉版 WIT 拥有（RFC §5–§6）。

## 4. 隔离面（禁止混用）

| 禁止 | 原因 |
|------|------|
| 本仓 runtime 依赖 `ai.tegmentum:wasmtime4j` | 引擎政策 |
| 轨 A CI 强制构建本仓 native | 拖慢 Demo 验收 |
| 未成熟时把轨 A Demo 默认切到本仓 | NG-1 |
| 在轨 A 内嵌整份本仓源码树当子模块 | 边界模糊 |
| 用 sync-compat「冒充」本仓真 async DoD | NG-8 |
| 以轨 A 扁平面是否已有某回调，作为本仓是否开切片的门禁 | RFC：形状由 WIT 定 |

## 5. 集成策略（现行）

- 本仓自带 smoke / 仪器；需要 GPU 时 **显式依赖** 轨 A engineered artifacts。  
- 轨 A `android-demo` **可以**继续只用 4j。可选「本仓 runtime」入口永远是 feature flag，默认仍 4j。  
- 本仓仪器 **不以**「与轨 A cube 对等」为扩面门禁。  
- 后端若缺能力：优先在本仓编组层表达 WIT；必要时对轨 A 提 **仅 Host 能力** 补丁（库维护，不是双产品线）。

长期「把轨 A Demo 默认切到本仓」仍须独立 RFC（回滚、版本钉死、谁维护 natives）。**那不是本仓推进 wasi:webgpu 的前置条件。**

## 6. 沟通口径

- 谈「可演示的 experimental WebGPU cube」→ **轨 A**。  
- 谈「Android JVM 上推进官方形状的 wasi:webgpu + WASI 0.3」→ **本仓**。  
- 禁止把本仓计划进度写成轨 A「已支持真 async」或「已合规 wasi:webgpu」。  
- 禁止再说「两仓并行推进同一条 Guest ABI」。

## 7. 变更流程

| 变更 | 落点 |
|------|------|
| cube / sync-compat / 4j Demo | 仅轨 A |
| Wasmtime 版本、JNI、CM 编组、wasi:webgpu Guest 形状 | **仅本仓** |
| Dawn / Host **能力**（新 GPU 操作） | 本仓可提需求；实现可在本仓调用层或轨 A Host 库 |
| 本仓公共 API / 版本号 | 见 [`api-stability.md`](api-stability.md) |
| 本页契约 | 本仓文档 PR；不要求轨 A 同步改排期看板 |
