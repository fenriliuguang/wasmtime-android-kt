# 非目标（轨 B）

**中文** | [English](non-goals.en.md)

> 写死「做什么」之外的边界，防止双轨并行时范围爆炸。  
> **长期阶段（2026-08-11+）** 与 [`long-term-plan.md`](long-term-plan.md) 对齐；短期 M0–M5 已归档。

## 绝对非目标（直至新 RFC）

| ID | 项 |
|----|-----|
| NG-1 | 用本仓 **静默替换** 轨 A 主验收 / 默认 Demo runtime |
| NG-2 | 以 **wasmtime4j** 作为本仓运行时依赖（含传递依赖跑 CM） |
| NG-3 | 重造完整 **Kotlin WebGPU 客户端 API**（三方图形引擎式） |
| NG-4 | 以「实现 **全部** WASI 0.3 worlds」或「一次过完 **全量** wasi-testsuite P3」为单一关门 / 对外 KPI（**主推已批准 P3 切片**见 [`wasi-p3-surface.md`](wasi-p3-surface.md)，≠ 全量套件承诺） |
| NG-5 | 在未另开合规 / 发布 RFC 前，宣传 **合规 wasi:webgpu 产品** 或「生产级 Android Wasm runtime」 |
| NG-6 | 默认 **对外 Maven Central 发布** |
| NG-7 | 在本仓实现 **第二套 Dawn Host**（应复用轨 A L2） |
| NG-8 | 以 sync-compat **冒充** 真 CM async / WASI 0.3 异步 DoD |
| NG-9 | 将 **wasi-gfx / 多 window** 升为与 `wasi:webgpu` 同级的近端 P0（显示面另 RFC） |
| NG-10 | 要求轨 A 为轨 B 修改而破坏 sync-compat 锁死条款 |
| NG-11 | 以非官方引擎或 4j 绑定替换「追踪官方 Wasmtime」政策（见 [`wasmtime-tracking.md`](wasmtime-tracking.md)） |

## 延期非目标（可另开 RFC）

| ID | 项 | 最早考虑 |
|----|-----|----------|
| DG-1 | Panama 桌面绑定 | 长期 L5 附近 |
| DG-2 | iOS / 桌面一等公民 | 非 Android-first 阶段 |
| DG-3 | 完整 WASI 云/CLI 运行时（全 package 产品化） | 产品化后；与 NG-4 切片策略区分 |
| DG-4 | 字节码解释器后备（无 Cranelift） | 有明确设备需求时 |
| DG-5 | 与轨 A 合并为单仓 monorepo | 仅当维护成本证明更优 |
| DG-6 | wasi-gfx 最小上屏胶水 | [`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) W4 选项 B |

## 允许的「看起来像非目标但其实要做」

| 项 | 说明 |
|----|------|
| 参考 4j 源码 / 补丁 | 学习可以；依赖不行 |
| 桌面 `.so` 开发构建 | 便利可以；门禁仍 Android-first |
| WASI 0.3 **已批准** package 子集 | **主推**（clocks/random/cli/… 按表面优先级）；不作「完整套件」宣称 |
| `wasi:webgpu` **提案**实现与反馈 | **P0**；推进 ≠ 合规宣称（NG-5） |
| 对照 `wasi-webgpu-wasmtime` 等上游宿主 | 形状/经验对照可以；不替代轨 A Dawn 主路径 |
