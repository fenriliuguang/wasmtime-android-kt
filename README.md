# wasmtime-android-kt

**experimental · Android-first Java/Kotlin Wasm runtime（轨 B）**

**中文** | [English](README.en.md)

> **状态：短期 M0–M5 已归档；长期规划已立（2026-08-11，文档期）。**  
> **现行主线：** WASI 0.3 正式面 + **wasi:webgpu 提案（P0）** + 追踪官方 Wasmtime → [`docs/scheme/long-term-plan.md`](docs/scheme/long-term-plan.md)。  
> 短期薄 L1 归档：[`docs/scheme/archive/m0-m5-thin-l1.md`](docs/scheme/archive/m0-m5-thin-l1.md)。  
> 姊妹项目（轨 A）：[`../wasi-webgpu-jvm-mvp`](../wasi-webgpu-jvm-mvp) — 锁死 **sync-compat** + wasmtime4j；真机验收仍为 CM cube。  
> 详细章程：[`docs/scheme/charter.md`](docs/scheme/charter.md)。  
> **构建说明：** [`docs/build.md`](docs/build.md)。

## 一句话

**长期**：Android-first 的 Java/Kotlin Component 运行时——主推 **WASI 0.3** 已批准能力，提案面优先 **wasi:webgpu**，引擎只跟 **官方 Wasmtime**。  
**已验证底座（已归档）**：自研薄 L1（JNI）可插轨 A L2（Dawn），并具备真 CM async。

## 双轨关系

| 轨 | 仓库 | Runtime | Async | 角色 |
|----|------|---------|-------|------|
| **A** | `wasi-webgpu-jvm-mvp` | wasmtime4j + 本仓补丁 | **锁死 sync-compat** | 可演示 / CI / 真机 cube 主验收 |
| **B** | **本仓** `wasmtime-android-kt` | 官方 Wasmtime + 自研 JNI | 目标真 CM async | Android-first；不阻塞轨 A |

```text
轨 A：Guest ──► wasmtime4j ──► WasmtimeCmLinker ──► L2 ──► Dawn
轨 B：Guest ──► 本仓薄 L1（官方 Wasmtime）──► 同一 L2 ──► Dawn   （短期）
      远期可演进为更完整的 Android JVM Wasm runtime
```

硬约束：**L2 不依赖 L1**；两轨共享 Host / ABI 常量，**隔离** native 与 CI 门禁。

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

| 文档 | 说明 |
|------|------|
| [长期计划](docs/scheme/long-term-plan.md) | **现行**主线：WASI 0.3 · wasi:webgpu · Wasmtime |
| [WASI 0.3 表面](docs/scheme/wasi-p3-surface.md) | 正式特性优先级 / 切片门禁 |
| [wasi:webgpu 路线图](docs/scheme/roadmap-wasi-webgpu.md) | 提案推进（P0） |
| [Wasmtime 追踪](docs/scheme/wasmtime-tracking.md) | 钉版 / 升级 / 回归 |
| [贡献指南](CONTRIBUTING.md) | PR / CI / 权限；链到 VCS 与构建文档 |
| [版本控制工作流](docs/scheme/vcs-workflow.md) | 短命分支 + PR；Ruleset 清单 |
| [experimental ↔ wasi:webgpu 差距](docs/mapping/gap-experimental-vs-wasi-webgpu.md) | W0 对照表 |
| [短期归档 M0–M5](docs/scheme/archive/m0-m5-thin-l1.md) | 薄 L1 验证路径收口 |
| [章程](docs/scheme/charter.md) | 背景、原则、风险 |
| [方案索引](docs/scheme/README.md) | 阶段表 |
| [API 稳定性](docs/scheme/api-stability.md) | experimental `0.x` SemVer |
| [如何构建](docs/build.md) | NDK / cargo-ndk / Gradle |
| [贡献者 / 桌面壳](docs/contribute.md) | 可选宿主 native + JVM 冒烟 |
| [双轨契约](docs/scheme/dual-track.md) | 与轨 A 边界 |
| [技术栈](docs/scheme/tech-stack.md) | Wasmtime / NDK / JDK |
| [里程碑史实](docs/scheme/milestones.md) | M0–M5 冻结 DoD |
| [非目标](docs/scheme/non-goals.md) | 明确不做 |
| [Changelog](CHANGELOG.md) | 变更 |

## 当前交付

- **长期规划文档已立**（无新代码要求）：见上「长期计划」四件套  
- **短期底座已归档**：M0–M5 薄 L1（同步 CM、真 CM async、experimental webgpu→L2、Dawn smoke、错误/ABI/API 政策）  
- **不**依赖 wasmtime4j；**不**默认对外发布；**不**替换轨 A 主验收  

## 参考

- 轨 A：[`wasi-webgpu-jvm-mvp`](../wasi-webgpu-jvm-mvp) · [真 CM async 闸门归档](../wasi-webgpu-jvm-mvp/docs/scheme/archive-true-cm-async-dod.md) · [UPSTREAM §5](../wasi-webgpu-jvm-mvp/patches/UPSTREAM.md)  
- [Wasmtime](https://docs.wasmtime.dev/) · [Component Model Async](https://component-model.bytecodealliance.org/design/async.html) · [androidx.webgpu](https://developer.android.com/jetpack/androidx/releases/webgpu)  
