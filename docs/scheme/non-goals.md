# 非目标（轨 B）

**中文** | [English](non-goals.en.md)

> 写死「做什么」之外的边界，防止双轨并行时范围爆炸。

## 绝对非目标（直至新 RFC）

| ID | 项 |
|----|-----|
| NG-1 | 用本仓 **静默替换** 轨 A 主验收 / 默认 Demo runtime |
| NG-2 | 以 **wasmtime4j** 作为本仓运行时依赖（含传递依赖跑 CM） |
| NG-3 | 重造完整 **Kotlin WebGPU 客户端 API**（三方图形引擎式） |
| NG-4 | 短期关门条件绑定完整 **WASI Preview3** / `wasi-http` / `wasi-nn` |
| NG-5 | 宣传 **合规 wasi:webgpu 产品** 或「生产级 Android Wasm runtime」（未达 M5 政策前） |
| NG-6 | 默认 **对外 Maven Central 发布** |
| NG-7 | 在本仓实现 **第二套 Dawn Host**（应复用轨 A L2） |
| NG-8 | 以 sync-compat **冒充** M2 真 CM async DoD |
| NG-9 | wasi-gfx / 多 window 抽象作为短期目标 |
| NG-10 | 要求轨 A 为轨 B 修改而破坏 sync-compat 锁死条款 |

## 延期非目标（可另开 RFC）

| ID | 项 | 最早考虑 |
|----|-----|----------|
| DG-1 | Panama 桌面绑定 | M5 后 |
| DG-2 | iOS / 桌面一等公民 | 非 Android-first 阶段 |
| DG-3 | 完整 WASI 云/CLI 运行时 | 产品化后 |
| DG-4 | 字节码解释器后备（无 Cranelift） | 有明确设备需求时 |
| DG-5 | 与轨 A 合并为单仓 monorepo | 仅当维护成本证明更优 |

## 允许的「看起来像非目标但其实要做」

| 项 | 说明 |
|----|------|
| 参考 4j 源码 / 补丁 | 学习可以；依赖不行 |
| 桌面 `.so` 开发构建 | 便利可以；门禁仍 Android-first |
| 最小 WASI 子集 | 若 Guest 工具链需要，可极小 stub，不作产品卖点 |
