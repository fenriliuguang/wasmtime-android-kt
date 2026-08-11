# 技术栈与依赖（轨 B）

**中文** | [English](tech-stack.en.md)

> 初订于 2026-08-10；**代码期可修订**，修订须更新本章 + CHANGELOG。  
> 配套：[`charter.md`](charter.md) · [`long-term-plan.md`](long-term-plan.md) · [`wasmtime-tracking.md`](wasmtime-tracking.md)（版本升级流程）。

## 1. 引擎：官方 Wasmtime

| 项 | 初订 |
|----|------|
| Crate | `wasmtime`（Bytecode Alliance） |
| 建议主版本 | **47.x**（与轨 A wasmtime4j 钉的 Wasmtime 代际对齐，降低 Guest/WIT 漂移）；若需更新 API，文档写明与轨 A 差异 |
| Features（最小） | `component-model`, 隐含/显式 `component-model-async`, 运行所需 `async` / runtime |
| 可选稍后 | `cranelift` 默认；`wasmtime-wasi` / WASI 0.3 Host — 按 [`wasi-p3-surface.md`](wasi-p3-surface.md) 切片启用，流程见 [`wasmtime-tracking.md`](wasmtime-tracking.md) |
| 禁用依赖 | **不**链接 `wasmtime4j-native` / 不 `dlopen` 4j `.so` |

### 1.1 必须使用的官方 async 面（M2+）

- `LinkerInstance::func_wrap_concurrent`（或等价 concurrent 注册）  
- `FutureReader::new` + `FutureProducer`（host 写端）  
- Store 侧 `run_concurrent` / event loop 驱动（具体 API 随钉版本核对 docs.rs）  
- **禁止**仅用「sync 回调包进 `async move`」当作真 async DoD  

### 1.2 同步面（M1）

- 常规 `func_new` / typed linker  
- Component 编译与 instantiate  
- Resource：host dynamic resource + **rep=u32** 模型（对齐轨 A L2 `GpuHandle.raw`）

## 2. 绑定层：Rust → JNI → Kotlin/Java

| 层 | 技术 | 说明 |
|----|------|------|
| Native | Rust `cdylib` | 单 `.so`：`libwasmtime_android_kt.so`（已钉） |
| JNI | `jni` crate | `JNI_OnLoad` 返回 ART 可接受版本（轨 A 经验：`JNI_VERSION_1_6`） |
| Java API | 手写 Kotlin/Java | 最小类型；避免 JSON 编组为主路径（4j ConcurrentCallCodec 覆辙） |
| Panama | **首期不做** | Android 主路径是 JNI；桌面若以后加 Panama 不得挡 Android |

### 2.1 编组策略（初订）

**优先：** 对热点路径用 **定长/显式 JNI 参数** 或 **有限集 ComponentVal 专用编解码**（Kotlin 侧），减少「任意 JSON ↔ Val」。  

**可接受：** 启动期/冷路径用结构化字节（flatbuffer/cbor）——须有 schema，禁止无符号 u64 十进制陷阱。  

Resource：guest↔host 一律 **u32 rep**；表在 L2；native 负责 ResourceAny ↔ U32（吸收轨 A cm-resources 思路）。

## 3. Android / NDK

| 项 | 初订 |
|----|------|
| ABI | **arm64-v8a** 一等；**x86_64** 模拟器二等 |
| NDK | **28.2.13676358**（与轨 A `build-wasmtime4j-android.ps1` 对齐） |
| STL / 链接 | 按 wasmtime + jni 静态需求；注意 16KB page / 现代 Android |
| 加载 | `System.loadLibrary`；注意多 classloader |
| 指针 | ARM64 TBI/PAC：句柄按 **unsigned** 处理（轨 A Validation / ConcurrentCallCodec 教训） |

构建流水线：

```text
Rust (scripts/build-native-android.ps1 / cargo ndk)
  → android/jniLibs/<abi>/libwasmtime_android_kt.so
  → :android AAR → :smoke-app
```

详见 [`../build.md`](../build.md)。

## 4. JVM / Gradle

| 项 | 初订 |
|----|------|
| 语言 | Kotlin 优先，必要 Java |
| JDK | 17+ 构建；与 AGP 兼容 |
| Android Gradle Plugin | **9.3.1**（与轨 A 同代） |
| 模块 | 见章程 §4.1（M0 已建） |
| 发布 | 默认仅本地 / 私有；experimental |

## 5. 与轨 A Host 的依赖

| Artifact（轨 A） | 轨 B 用法 |
|------------------|-----------|
| `host-api` | M3+ 编译依赖；接口面 |
| `host-webgpu` | Android Dawn 实现；仪器联调 |
| `abi-cm` / `abi-wasi` | 函数名 / resource 名常量 |
| `runtime-wasmtime` | **不**依赖 |
| `android-demo` | 可选后期联调；不反向依赖本仓 |

轨 A 本地发布：`publishEngineeredToMavenLocal`（见轨 A `docs/maven-local.md`）。

## 6. Guest / WIT

| 阶段 | Guest |
|------|--------|
| M1 | 自制最小 sync component（可 wat/wit-bindgen） |
| M2 | 自制最小 **async** import smoke |
| M4 | 复用轨 A `guest/cube-cm` 或裁剪子集；版本钉 `@0.8.0` experimental |

工具链：`wit-bindgen`、`wasm-tools`；版本写入锁文件。

## 7. 测试矩阵（规划）

| 层级 | 环境 | 内容 |
|------|------|------|
| 单元 | 桌面 JVM + 桌面 `.so`（可选） | API / future complete |
| 仪器 | 真机 arm64 | M0 加载；M2 async；M4 上屏 |
| 回归 | 轨 A 脚本 | **证明未破坏** sync-compat 门禁 |

## 8. 明确不依赖

- tegmentum wasmtime4j（Maven / 源码作运行时）  
- 浏览器 WebAssembly JS API  
- 默认把第三方 `wasi-webgpu-wasmtime` 链进 Android cdylib（对照实现另论）  

## 9. 上游态度

- **官方 Wasmtime：唯一引擎依赖**；版本与特性按 [`wasmtime-tracking.md`](wasmtime-tracking.md) 追踪；issue/PR **可以**按需（与轨 A「不对 4j 提 PR」政策分离）  
- WASI 0.3 正式规格与 wasi:webgpu 提案：产品优先级见长期计划；实现跟 Wasmtime 代际对齐  
- wasmtime4j：仅作反面教材与经验来源；不作为依赖  
