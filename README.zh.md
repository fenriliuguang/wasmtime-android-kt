# wasmtime-android-kt

**experimental · Android-first 的 Java/Kotlin Component 运行时**

[English](README.md) | **中文**

基于 **官方 Wasmtime** 的 Android（JNI / ART）嵌入，托管 [Component Model](https://component-model.bytecodealliance.org/) Guest（含 **真 CM async**），首发提案世界为规范形状的 [`wasi:webgpu`](https://github.com/WebAssembly/wasi-webgpu)。

本仓目标是成为 Wasm 组件链上 **可引用的 Android Host**——不是 UI 框架、不是重写的 Dawn、也不是生产级 WASI 发行版。**默认产品/测试构件包含 Dawn**；核心 AAR 不含。见 [`rfc-pluggable-gpu-backend.zh.md`](docs/scheme/rfc-pluggable-gpu-backend.zh.md)。

状态：**experimental `0.x`**。不宣称合规 wasi:webgpu / CTS。坐标 **`0.1.0`**。发布：[`.github/workflows/publish.yml`](.github/workflows/publish.yml)（[`rfc-l5-productization.md`](docs/scheme/rfc-l5-productization.md)）。

若与英文冲突，**以 [README.md](README.md) 为准**。

## 现行计划

| 优先级 | 内容 |
|--------|------|
| **P0** | 钉版 `wasi:webgpu@0.3.0-rc.2` Guest 形状 + 真 CM async — **2026-08-22 关闭** |
| **P1** | 已批准 WASI 0.3 官方形状 + 真机仪器 — **2026-08-26 关闭**（[`p1-wasi-p3.zh.md`](docs/archive/p1-wasi-p3.zh.md)） |
| **P2** | Wasmtime 钉 — **点名**（[`wasmtime-p2.md`](docs/agent/wasmtime-p2.md)） |
| **L5 / 0.1.0** | 产品子集 + gfx 循环 + **`0.1.0` 坐标 / 发布 workflow**（[`product-010.md`](docs/agent/product-010.md)） |

成功标准：[`rfc-ecosystem-contribution.zh.md`](docs/scheme/rfc-ecosystem-contribution.zh.md) — 可复现、可引用；**禁止**向上游开 GitHub issue。GPU 接线：[`rfc-pluggable-gpu-backend.zh.md`](docs/scheme/rfc-pluggable-gpu-backend.zh.md)。**P0 与 P1 已关闭。** **`0.1.0` 产品队列在 P010-PUB 后为空。** P2 Wasmtime 钉为点名。

## 快速开始

```powershell
.\scripts\build-native-android.ps1
.\gradlew.bat :smoke-app:assembleDebug
.\gradlew.bat :smoke-app:connectedDebugAndroidTest
```

版本钉死见 [`docs/build.md`](docs/build.md)。

推荐消费坐标（0.x 默认 bundle）：

```kotlin
implementation("io.github.fenriliuguang.wasmtime.android:android-webgpu:0.1.0")
```

不要直接依赖 `runtime-api` / `runtime-jni`。首次 Central 按下仍需要 secrets + arm64 `.so`；此前请源码检出本仓。GPU 仪器走仓内 `:host-dawn` + `androidx.webgpu`，见 [`docs/blocked-gpu-host.md`](docs/blocked-gpu-host.md)。

## 文档

英文为正文；中文为副文档（`.zh.md`）。史实见 [`docs/archive/README.md`](docs/archive/README.md)。
