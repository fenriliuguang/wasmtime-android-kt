# wasmtime-android-kt

**experimental · Android-first Java/Kotlin Wasm runtime（轨 B）**

**中文** | [English](README.en.md)

> **状态：M0–M4 已真机验收；M5 进行中（错误模型 + 产物布局，2026-08-11）。** M4 为 Dawn clear→present 专用 smoke（非轨 A cube 替换）。  
> 姊妹项目（轨 A）：[`../wasi-webgpu-jvm-mvp`](../wasi-webgpu-jvm-mvp) — 锁死 **sync-compat** + wasmtime4j；真机验收仍为 CM cube。  
> 详细章程：[`docs/scheme/charter.md`](docs/scheme/charter.md)。  
> **构建说明：** [`docs/build.md`](docs/build.md)。

## 一句话

**长期**：面向 Android 的 Java/Kotlin Wasm 运行时（Component Model 优先）。  
**短期**：基于 **官方 Wasmtime** 的自研薄 L1（JNI），可插到轨 A 已有 L2（`WasiWebGpuHost` / Dawn），并具备真 CM async 能力。

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
| `scripts/build-native-android.ps1` | cargo-ndk 流水线 |

## 文档索引

| 文档 | 说明 |
|------|------|
| [章程（主计划）](docs/scheme/charter.md) | 背景、依赖、目标堆叠、里程碑、风险、DoD |
| [如何构建](docs/build.md) | NDK / cargo-ndk / Gradle 复现步骤 |
| [轨 A L2 依赖](docs/build-track-a-deps.md) | mavenLocal `host-api` / `abi-cm` |
| [错误模型](docs/mapping/errors.md) | L1 异常分层（M5）；M3 subset 见 [errors-m3](docs/mapping/errors-m3.md) |
| [Native 产物布局](docs/mapping/artifacts.md) | jniLibs ABI / `build-info.json` / 校验脚本 |
| [M4 vs cube 差距](docs/mapping/gap-m4-vs-cube.md) | 相对轨 A cube 缺什么 |
| [方案索引](docs/scheme/README.md) | 阶段表 |
| [双轨契约](docs/scheme/dual-track.md) | 与轨 A 的边界、共享面、禁止事项 |
| [技术栈与依赖](docs/scheme/tech-stack.md) | Wasmtime / NDK / JDK / 构建 |
| [里程碑与 DoD](docs/scheme/milestones.md) | M0–M5 堆叠 |
| [非目标](docs/scheme/non-goals.md) | 明确不做 |
| [Android 线程契约](docs/mapping/threading-android.md) | Dawn / Surface / CM scheduler |
| [M2 `run_concurrent` 泵](docs/mapping/threading-m2-async.md) | 谁驱动 async 事件循环 |
| [Changelog](CHANGELOG.md) | 变更 |

## 当前交付

- 计划文档 + **M0 Gradle / native 骨架**  
- `JNI_OnLoad` → `JNI_VERSION_1_6`；`loadLibrary` 仪器用例  
- **无** CM instantiate / async / L2 接线（M1+）  
- **不**依赖 wasmtime4j  

## 参考

- 轨 A：[`wasi-webgpu-jvm-mvp`](../wasi-webgpu-jvm-mvp) · [真 CM async 闸门归档](../wasi-webgpu-jvm-mvp/docs/scheme/archive-true-cm-async-dod.md) · [UPSTREAM §5](../wasi-webgpu-jvm-mvp/patches/UPSTREAM.md)  
- [Wasmtime](https://docs.wasmtime.dev/) · [Component Model Async](https://component-model.bytecodealliance.org/design/async.html) · [androidx.webgpu](https://developer.android.com/jetpack/androidx/releases/webgpu)  
