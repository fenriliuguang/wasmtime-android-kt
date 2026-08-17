# 归档：短期目标 M0–M5（薄 L1 验证路径）

**中文** | （暂无 EN）

> **状态：已归档（2026-08-11）。**  
> 短期验证路径已收口；现行规划见 [`../long-term-plan.md`](../long-term-plan.md)。  
> 原里程碑正文保留为史实：[`../milestones.md`](../milestones.md)。

## 1. 阶段定位（史实）

| 项 | 内容 |
|----|------|
| 代号 | 薄 L1 验证路径（轨 B 短期） |
| 目的 | 证明：官方 Wasmtime + 自研 JNI 可在 Android 上跑同步 CM、**真 CM async**，并插上轨 A 同一 L2（Dawn smoke） |
| 收口日 | 2026-08-11（M0–M5 DoD 全部勾选） |
| 后续 | **不再**以本堆叠为产品主线；能力作为长期计划的已验证底座 |

一句话结论：

> 轨 B **存在理由**（相对轨 A）已用官方 concurrent API 验证；下一步从「webgpu 薄 L1 smoke」转向 **WASI 0.3 正式面 + wasi:webgpu 提案推进**。

## 2. DoD 收口摘要

| 里程碑 | 一句话交付 | 关键证据 |
|--------|------------|----------|
| **M0** | ART `loadLibrary` + Wasmtime 版本探针 | `LoadLibraryInstrumentedTest`；[`../../build.md`](../../build.md) |
| **M1** | 同步 CM：host import / export / resource u32 | fixtures `m1/*`；Kotlin `Engine`/`Store`/`Linker`/`Instance` |
| **M2** | 真 CM async 硬闸门（future complete） | `AsyncCmGetInstrumentedTest`；[`../../mapping/threading-m2-async.md`](../../mapping/threading-m2-async.md) |
| **M3** | L1→轨 A L2 `request-adapter`（Cpu） | `RequestAdapterInstrumentedTest`；[`../../mapping/errors-m3.md`](../../mapping/errors-m3.md) |
| **M4** | Dawn clear→present 专用 smoke（非轨 A cube 替换） | `DawnRenderSmokeInstrumentedTest`；[`../../mapping/gap-m4-vs-cube.md`](../../mapping/gap-m4-vs-cube.md) |
| **M5** | 错误模型 / 双 ABI / API 政策 / 贡献者壳 / worlds 初稿 RFC | [`../api-stability.md`](../api-stability.md)；[`../../contribute.md`](../../contribute.md)；[`../rfc-wasi-worlds.md`](../rfc-wasi-worlds.md) |

完整勾选表见 [`../milestones.md`](../milestones.md)。

## 3. 已验证能力（可被长期计划继承）

1. **官方 Wasmtime**（钉 `47.0.2`）经 Rust cdylib + JNI 加载于 ART（arm64 一等）。  
2. **Component Model 同步环** + **resource rep=u32**。  
3. **真 CM async**：`func_wrap_concurrent` + `FutureReader` + `run_concurrent`（禁止再以 sync-compat 冒充）。  
4. **薄 L1 插 L2**：experimental `webgpu-cm` 子集 → 轨 A `host-api` / Dawn；L2 不依赖 L1。  
5. **工程硬化雏形**：typed errors、`jniLibs` 双 ABI + `build-info.json`、`0.x-experimental` 政策。

## 4. 明确未完成 / 不在本归档宣称内

- 未替换轨 A 主验收 / CM cube（见 dual-track NG-1）  
- 未宣称合规 `wasi:webgpu` 或生产级 runtime  
- 未实现标准 WASI 0.3 worlds（cli / http / …）主机面  
- 未跟进提案 `wasi:webgpu` 的标准 WIT（仍为 experimental 扁平子集）  
- 未默认 Maven Central 发布  

差距史实：[`../../mapping/gap-m4-vs-cube.md`](../../mapping/gap-m4-vs-cube.md)。

## 5. 文档处置

| 文档 | 处置 |
|------|------|
| [`../milestones.md`](../milestones.md) | **冻结史实**；顶部指向本归档与长期计划 |
| [`../charter.md`](../charter.md) §2.2 / §6 | 短期堆叠标为已完成；愿景承接长期计划 |
| [`../rfc-wasi-worlds.md`](../rfc-wasi-worlds.md) | **Superseded**（短期「有条件支持」口径）→ 由长期计划 + WASI P3 / webgpu 专章取代 |
| `docs/mapping/*`（M2/M3/M4） | 保留为工程契约与差距史实；非新产品 KPI |
| 本页 | 短期阶段唯一归档入口 |

## 6. 修订规则

- 本归档 **只增不改结论**：若发现史实错误，可补勘误附录，不得改写「已完成」勾选语义。  
- 新产品范围、里程碑编号从 **L0+**（见长期计划）另起，**不**复用 M0–M5 作未完成项。  
