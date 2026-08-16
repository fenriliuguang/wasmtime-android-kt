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

### 2.1 编组策略（2026-08-16 起以 RFC 为准）

权威：[`rfc-wasi-webgpu-canonical-shape.md`](rfc-wasi-webgpu-canonical-shape.md) §6。

**Guest 边界（硬）：** 走 Component Model 规范 lowering。`own` / `borrow` / record / `option` / `result` / `list` / `string` 与钉版 WIT 同构。禁止再把「host 固定 descriptor + 过渡 u32」当新切片目标。

**实现分层：**

| 层 | 职责 |
|----|------|
| `native/` Linker | 按 WIT 注册；Rust 签名与 WIT 同构 |
| 编解码（S1 起有限集） | 只为当前切片用到的 WIT 类型 lowering |
| Kotlin L2 回调 | 接已解码参数或后端句柄 |
| L2 / Dawn 句柄 | **rep 仍可 u32**；表在 native / Host |

**禁止：** 无 schema 的 JSON 作为主路径（4j ConcurrentCallCodec 覆辙）；「成功才返回 u32、失败 panic」冒充 `result`。

Resource：guest↔host 的 **rep** 仍可 u32（对齐 Dawn `GpuHandle.raw`）；**Guest 所见**必须是 resource，不是裸 u32 返回值。

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
| `host-api` | 编译依赖；**后端**接口面（不是 Guest ABI 源） |
| `host-webgpu` | Android Dawn 实现；仪器联调 |
| `abi-cm` / `abi-wasi` | 可参考常量；**产品 import 名以钉版 WIT 为准** |
| `runtime-wasmtime` | **不**依赖 |
| `android-demo` | 展示 Demo；不反向依赖本仓 |

轨 A 本地发布：`publishEngineeredToMavenLocal`（见轨 A `docs/maven-local.md`）。

## 6. Guest / WIT

| 阶段 | Guest |
|------|--------|
| M1 | 自制最小 sync component（可 wat/wit-bindgen） |
| M2 | 自制最小 **async** import smoke |
| 规范路径（S1+） | 与 `wasi:webgpu@0.3.0-rc.2` WIT **同构** 的 component（wit-bindgen 或手写 wat） |
| 遗留 | `experimental:webgpu-cm@0.8.0` / 轨 A `cube-cm`：Demo 与冻结过渡回归，**不**再扩面 |

工具链：`wit-bindgen`、`wasm-tools`；版本写入锁文件。

## 7. 测试矩阵（规划）

| 层级 | 环境 | 内容 |
|------|------|------|
| 单元 | 桌面 JVM + 桌面 `.so`（可选） | API / future complete |
| 仪器 | 真机 arm64 | M0 加载；M2 async；M4 上屏 |
| 回归 | 轨 A 脚本（仅当要证明没碰 4j Demo） | **不是**本仓扩面门禁 |

## 8. 明确不依赖

- tegmentum wasmtime4j（Maven / 源码作运行时）  
- 浏览器 WebAssembly JS API  
- 默认把第三方 `wasi-webgpu-wasmtime` 链进 Android cdylib（对照实现另论）  

## 9. 上游态度

- **官方 Wasmtime：唯一引擎依赖**；版本与特性按 [`wasmtime-tracking.md`](wasmtime-tracking.md) 追踪；issue/PR **可以**按需（与轨 A「不对 4j 提 PR」政策分离）  
- WASI 0.3 正式规格与 wasi:webgpu 提案：产品优先级见长期计划；实现跟 Wasmtime 代际对齐  
- wasmtime4j：仅作反面教材与经验来源；不作为依赖  
