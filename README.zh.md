# wasmtime-android-kt

**experimental · Android-first 的 Java/Kotlin Component 运行时**

[English](README.md) | **中文**

基于 **官方 Wasmtime** 的 Android（JNI / ART）嵌入，托管 [Component Model](https://component-model.bytecodealliance.org/) Guest（含 **真 CM async**），首发提案世界为规范形状的 [`wasi:webgpu`](https://github.com/WebAssembly/wasi-webgpu)。

本仓目标是成为 Wasm 组件链上 **可引用的 Android Host**——不是 UI 框架、不是重写的 Dawn、也不是生产级 WASI 发行版。**默认产品/测试构件包含 Dawn**；核心 AAR 不含。见 [`rfc-pluggable-gpu-backend.zh.md`](docs/scheme/rfc-pluggable-gpu-backend.zh.md)。

状态：**experimental**。不宣称合规 wasi:webgpu 产品；不默认发 Maven Central。

若与英文冲突，**以 [README.md](README.md) 为准**。

## 现行计划

| 优先级 | 内容 |
|--------|------|
| **P0** | 钉版 `wasi:webgpu@0.3.0-rc.2` Guest 形状 + 真 CM async — **2026-08-22 关闭** |
| **P1** | 已批准 WASI 0.3 官方形状 + 真机仪器 — **2026-08-26 关闭**（[`p1-wasi-p3.zh.md`](docs/archive/p1-wasi-p3.zh.md)） |
| **P2** | **现行：** 追踪官方 Wasmtime（[`wasmtime-p2.md`](docs/agent/wasmtime-p2.md)） |

成功标准：[`rfc-ecosystem-contribution.zh.md`](docs/scheme/rfc-ecosystem-contribution.zh.md) — 可复现、可引用；**禁止**向上游开 GitHub issue。GPU 接线：[`rfc-pluggable-gpu-backend.zh.md`](docs/scheme/rfc-pluggable-gpu-backend.zh.md)。**P0 与 P1 已关闭。**

## 快速开始

```powershell
.\scripts\build-native-android.ps1
.\gradlew.bat :smoke-app:assembleDebug
.\gradlew.bat :smoke-app:connectedDebugAndroidTest
```

版本钉死见 [`docs/build.md`](docs/build.md)。

带 GPU 的仪器测试仍依赖 **未发布** 的 Host 库，本次文档改动 **没有** 删除该依赖——见 [`docs/blocked-gpu-host.md`](docs/blocked-gpu-host.md)。

## 文档

英文为正文；中文为副文档（`.zh.md`）。史实见 [`docs/archive/README.md`](docs/archive/README.md)。
