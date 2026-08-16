# wasmtime-android-kt

**experimental · Android-first Java/Kotlin Wasm runtime（轨 B）**

**中文** | [English](README.en.md)

> **状态：短期 M0–M5 已归档；长期规划现行。**  
> **2026-08-16：** 结束与轨 A 并行排期；本仓靠拢官方 `wasi:webgpu` 形状 → [`docs/scheme/rfc-wasi-webgpu-canonical-shape.md`](docs/scheme/rfc-wasi-webgpu-canonical-shape.md)。  
> **现行主线：** WASI 0.3 正式面 + **wasi:webgpu 提案（P0，规范 WIT 形状）** + 追踪官方 Wasmtime → [`docs/scheme/long-term-plan.md`](docs/scheme/long-term-plan.md)。  
> 短期薄 L1 归档：[`docs/scheme/archive/m0-m5-thin-l1.md`](docs/scheme/archive/m0-m5-thin-l1.md)。  
> 姊妹项目（轨 A）：[`../wasi-webgpu-jvm-mvp`](../wasi-webgpu-jvm-mvp) — **展示用简单 Demo**（experimental cube + wasmtime4j + sync-compat）。  
> 详细章程：[`docs/scheme/charter.md`](docs/scheme/charter.md)。  
> **构建说明：** [`docs/build.md`](docs/build.md)。

## 一句话

**长期**：Android-first 的 Java/Kotlin Component 运行时——主推 **WASI 0.3** 已批准能力，提案面优先 **官方形状的 wasi:webgpu**，引擎只跟 **官方 Wasmtime**。  
**已验证底座（已归档）**：自研薄 L1（JNI）可插轨 A Dawn Host 当后端，并具备真 CM async。  
**轨 A** 只作展示 Demo；本仓是 wasi:webgpu Guest 形状的唯一推进面。

## 与轨 A

| 仓 | 仓库 | Runtime | Async | 角色 |
|----|------|---------|-------|------|
| **A** | `wasi-webgpu-jvm-mvp` | wasmtime4j + 补丁 | **锁死 sync-compat** | **展示用简单 Demo**（experimental cube） |
| **本仓** | `wasmtime-android-kt` | 官方 Wasmtime + 自研 JNI | 真 CM async | Android-first；**拥有** wasi:webgpu WIT 形状 |

```text
轨 A（Demo）：Guest ──► wasmtime4j ──► experimental 扁平面 ──► L2 ──► Dawn
本仓（产品路径）：Guest（钉版 wasi:webgpu WIT）──► 本仓 L1 + 规范编组 ──► Host 库 ──► Dawn
```

硬约束：**不重造 Dawn**；**不**静默替换轨 A 默认 runtime；**不再**并行推进同一条 Guest ABI。

## 快速开始（M0）

```powershell
# 1) 交叉编译 libwasmtime_android_kt.so → android/jniLibs/
.\scripts\build-native-android.ps1

# 2) 组装 smoke APK
.\gradlew.bat :smoke-app:assembleDebug

# 3) 真机 / 模拟器仪器（需已装 .so 且 ABI 匹配）
.\gradlew.bat :smoke-app:connectedDebugAndroidTest
```

前置与钉死版本见 [`docs/build.md`](docs/build.md)（NDK `28.2.13676358`、Rust `1.97.1`、AGP `9.3.1`）。

## 模块

| 路径 | 说明 |
|------|------|
| `runtime-api/` | Kotlin 公共面（无 Android 依赖） |
| `runtime-jni/` | `NativeLoader` / JNI 声明 |
| `android/` | AAR + `jniLibs` |
| `smoke-app/` | 最小 Activity + `LoadLibraryInstrumentedTest` |
| `native/` | Rust cdylib（`wasmtime` 47.0.2 + JNI） |
| `scripts/build-native-android.ps1` | cargo-ndk 流水线（正式 ABI） |
| `scripts/build-native-host.ps1` | 可选桌面宿主 cdylib |

## 文档索引

> **本表是常青入口，不要为每一刀加行。** 切片文档挂到主题页：[`wasi-p3-surface.md`](docs/scheme/wasi-p3-surface.md)、[`roadmap-wasi-webgpu.md`](docs/scheme/roadmap-wasi-webgpu.md)、[`docs/scheme/README.md`](docs/scheme/README.md)。

| 文档 | 说明 |
|------|------|
| [计划变更 RFC](docs/scheme/rfc-wasi-webgpu-canonical-shape.md) | **Accepted：** 结束双轨并行；规范形状；S 系列 |
| [长期计划](docs/scheme/long-term-plan.md) | **现行**主线：WASI 0.3 · wasi:webgpu · Wasmtime |
| [WASI 0.3 表面](docs/scheme/wasi-p3-surface.md) | 正式特性优先级 / 切片门禁 |
| [wasi:webgpu 路线图](docs/scheme/roadmap-wasi-webgpu.md) | 提案推进（P0；现行 **S 系列**） |
| [Wasmtime 追踪](docs/scheme/wasmtime-tracking.md) | 钉版 / 升级 / 回归 |
| [贡献指南](CONTRIBUTING.md) | PR / CI / 枢纽冻结；链到 VCS 与构建文档 |
| [版本控制工作流](docs/scheme/vcs-workflow.md) | 短命分支 + PR；Ruleset 清单 |
| [experimental ↔ wasi:webgpu 差距](docs/mapping/gap-experimental-vs-wasi-webgpu.md) | W0 对照表 |
| [短期归档 M0–M5](docs/scheme/archive/m0-m5-thin-l1.md) | 薄 L1 验证路径收口 |
| [章程](docs/scheme/charter.md) | 背景、原则、风险 |
| [方案索引](docs/scheme/README.md) | 阶段表 |
| [API 稳定性](docs/scheme/api-stability.md) | experimental `0.x` SemVer |
| [如何构建](docs/build.md) | NDK / cargo-ndk / Gradle |
| [贡献者 / 桌面壳](docs/contribute.md) | 可选宿主 native + JVM 冒烟 |
| [与轨 A 的边界](docs/scheme/dual-track.md) | 轨 A = Demo；本仓拥有形状 |
| [技术栈](docs/scheme/tech-stack.md) | Wasmtime / NDK / JDK |
| [里程碑史实](docs/scheme/milestones.md) | M0–M5 冻结 DoD |
| [非目标](docs/scheme/non-goals.md) | 明确不做 |
| [Changelog](CHANGELOG.md) | 已滚入历史；进行中见 [`changelog/unreleased/`](changelog/unreleased/) |
| [许可 / 第三方](THIRD_PARTY_NOTICES.md) | Apache-2.0 + 依赖摘要 |

## 当前交付

- **计划变更已立**（2026-08-16）：轨 A 仅 Demo；本仓靠拢官方 wasi:webgpu 形状  
- **长期规划文档**：见上「长期计划」  
- **短期底座已归档**：M0–M5 薄 L1  
- **不**依赖 wasmtime4j；**不**默认对外发布；**不**替换轨 A 默认 Demo runtime  
- 切片进度不写在本页：见 RFC / webgpu 路线图 / `changelog/unreleased/`  

## 许可

本仓采用 **Apache License 2.0**（见 [`LICENSE`](LICENSE)、[`NOTICE`](NOTICE)）。  
第三方依赖许可摘要：[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)（引擎 Wasmtime 为 Apache-2.0 WITH LLVM-exception）。

## 参考

- 轨 A：[`wasi-webgpu-jvm-mvp`](../wasi-webgpu-jvm-mvp) · [真 CM async 闸门归档](../wasi-webgpu-jvm-mvp/docs/scheme/archive-true-cm-async-dod.md) · [UPSTREAM §5](../wasi-webgpu-jvm-mvp/patches/UPSTREAM.md)  
- [Wasmtime](https://docs.wasmtime.dev/) · [Component Model Async](https://component-model.bytecodealliance.org/design/async.html) · [androidx.webgpu](https://developer.android.com/jetpack/androidx/releases/webgpu)  

