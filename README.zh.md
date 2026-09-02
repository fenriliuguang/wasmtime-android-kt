# wasmtime-android-kt

**experimental · Android-first 的 Java/Kotlin Component 运行时**

[English](README.md) | **中文**

基于 **官方 Wasmtime** 的 Android（JNI / ART）嵌入，托管 [Component Model](https://component-model.bytecodealliance.org/) Guest（含 **真 CM async**），首发提案世界为规范形状的 [`wasi:webgpu`](https://github.com/WebAssembly/wasi-webgpu)。

本仓目标是成为 Wasm 组件链上 **可引用的 Android Host**——不是 UI 框架、不是重写的 Dawn、也不是生产级 WASI 发行版。**默认产品/测试构件包含 Dawn**；核心 AAR 不含。见 [`rfc.md`](docs/scheme/rfc.md)。

状态：**experimental `0.x`**。不宣称合规 wasi:webgpu / CTS。坐标 **`0.1.0`**（未发布）。发包：`.github/workflows/publish.yml`（`main` 上的 `v*` 标签，或从 `main` 手动触发；GitHub Environment `release`）。

若与英文冲突，**以 [README.md](README.md) 为准**。禁止向上游开 GitHub issue。

## 快速开始

```powershell
.\scripts\build-native-android.ps1
.\gradlew.bat :smoke-app:assembleDebug
.\gradlew.bat :smoke-app:connectedDebugAndroidTest
```

版本钉：[`tech-stack.md`](docs/scheme/tech-stack.md)。协作：[`CONTRIBUTING.md`](CONTRIBUTING.md)。

## 演示

打包 guest wasm，用本 Android 运行时加载并上屏：[wasmtime-android-kt-examples](https://github.com/fenriliuguang/wasmtime-android-kt-examples)。本仓不内嵌该 app。仓外门禁：`.\scripts\verify-examples-gate.ps1`（includeBuild，不用 mavenLocal）。

推荐消费坐标（未发布）：

```kotlin
implementation("io.github.fenriliuguang.wasmtime.android:android-webgpu:0.1.0")
```

## 文档

英文为正文；中文为副文档（`.zh.md`）。
