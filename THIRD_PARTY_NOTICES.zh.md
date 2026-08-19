# 第三方许可摘要

[English](THIRD_PARTY_NOTICES.md) | **中文**

本仓自有代码见根目录 [`LICENSE`](LICENSE)（Apache-2.0）与 [`NOTICE`](NOTICE)。  
以下为**直接依赖与主要传递依赖**的许可扫描摘要（`cargo license`，2026-08-11）；完整 SPDX 以各 crate / Maven 构件为准。

## 1. Rust（`native/`）

| 依赖 | 角色 | SPDX（扫描结果） |
|------|------|------------------|
| `wasmtime` (+ Cranelift 等) | 引擎 | Apache-2.0 WITH LLVM-exception |
| `jni` | JNI 绑定 | Apache-2.0 OR MIT |
| `futures` | async 工具 | Apache-2.0 OR MIT |
| `pollster` | 阻塞驱动 | Apache-2.0 OR MIT |
| `getrandom` | WASI random CSPRNG | Apache-2.0 OR MIT |

传递依赖主要为 **Apache-2.0 OR MIT**、**MIT**、**MIT OR Unlicense**、**Zlib**，以及少量 **BSD-2/3-Clause** / **Unicode-3.0**（如 `encoding_rs`、`unicode-ident`）。均为宽松许可，与本仓 Apache-2.0 兼容。

重新扫描：

```powershell
cargo license --manifest-path native/Cargo.toml --avoid-dev-deps --avoid-build-deps
```

## 2. Guest（`guest/m2-async-smoke/`）

| 依赖 | SPDX（扫描结果） |
|------|------------------|
| `wit-bindgen`（及 wasm-tools 相关） | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT |

## 3. JVM / Android（Gradle）

| 依赖 | 典型 SPDX |
|------|-----------|
| AndroidX（`core-ktx`、`appcompat`、测试库） | Apache-2.0 |
| `androidx.webgpu:webgpu:1.0.0-alpha05` | Dawn Java + 捆绑 `.so`。AndroidX 构件 Apache-2.0；Dawn 原生多为 BSD-3-Clause，见 AAR `NOTICE`。**不要把 `.so` 提交进 git。** |
| Kotlin stdlib / Gradle 插件 | Apache-2.0 |
| JUnit 4 | EPL-1.0 |
| Vendor 的 Host Kotlin（`:host-dawn` `…experimental.host` / `dawn` / `abicm`） | MIT。拷自 `wasi-webgpu-jvm-mvp`（Copyright 焚日流光 2026）。全文：[`host-dawn/third_party/wasi-webgpu-jvm-mvp/LICENSE`](host-dawn/third_party/wasi-webgpu-jvm-mvp/LICENSE)。 |

JUnit 仅用于测试，不进入发布 AAR 运行时。

## 4. Vendor 的 WIT（`third_party/wasi-webgpu/`）

| 树 | 许可 |
|----|------|
| `wasi-webgpu` 标签 `v0.3.0-rc.2` | W3C Community CLA。见 [`third_party/wasi-webgpu/v0.3.0-rc.2/LICENSE.md`](third_party/wasi-webgpu/v0.3.0-rc.2/LICENSE.md)。 |

钉版变更时才刷新。正文以[英文](THIRD_PARTY_NOTICES.md)为准。

## 5. 分发说明

发布或再分发含 `libwasmtime_android_kt.so` 的二进制时，须保留本仓 `LICENSE` / `NOTICE`，并遵守上游 Wasmtime（Apache-2.0 WITH LLVM-exception）及其他嵌入依赖的再分发条件。
