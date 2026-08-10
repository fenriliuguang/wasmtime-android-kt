# 章程：wasmtime-android-kt（轨 B）

**中文** | [English](charter.en.md)

> **状态：M0 工程骨架已落地（2026-08-10）。** CM API / async 仍属 M1+。  
> 姊妹轨 A：[`wasi-webgpu-jvm-mvp`](../../../wasi-webgpu-jvm-mvp) — **锁死 sync-compat**。  
> 索引：[`README.md`](../../README.md) · [`../build.md`](../build.md) · [`dual-track.md`](dual-track.md) · [`tech-stack.md`](tech-stack.md) · [`milestones.md`](milestones.md) · [`non-goals.md`](non-goals.md)

---

## 1. 背景

### 1.1 问题从哪来

轨 A（`wasi-webgpu-jvm-mvp`）已建成：

- **L2**：`WasiWebGpuHost` + Dawn（Android）/ CpuHost（桌面）
- **L1**：`runtime-wasmtime` 经 **wasmtime4j**（`47.0.2-1.5.0`）+ android / cm-resources 补丁
- **Guest**：`experimental:webgpu-cm` 旋转立方体；真机仪器验收稳定

标准 wasi:webgpu 0.3 大量方法是 WIT **`async func`**。轨 A 现用 **sync-compat**（Host 回调内 `CountDownLatch` + `processEvents`）跑通主链。

轨 A 曾立项「真 CM async」，切片 A Spike 证明：

| 层 | 结论 |
|----|------|
| Cargo `component-model-async` | 已编入 4j CM natives |
| Java future 完成面 | **缺失**（无 create/write/complete/reject） |
| `defineFunctionAsync` | 仅 `func_new_async` + 同步回调，≠ 官方 concurrent host |

因此轨 A **闸门关门**，停止在 4j 上改 L2/Linker 主链。详见轨 A [`archive-true-cm-async-dod.md`](../../../wasi-webgpu-jvm-mvp/docs/scheme/archive-true-cm-async-dod.md) 与 [`patches/UPSTREAM.md`](../../../wasi-webgpu-jvm-mvp/patches/UPSTREAM.md) §5。

### 1.2 为什么另开轨 B

1. **4j 缺口是绑定层问题**，不是「Android 不能跑官方 Wasmtime CM async」。  
2. 轨 A 必须保持 **可演示、可回归**；不宜把主验收绑在未验证的新 runtime 上。  
3. 官方 Wasmtime 已提供 `FutureProducer` / `FutureReader`、`func_wrap_concurrent`、`run_concurrent` 等 API，适合作为 Android 薄 L1 的底座。  
4. 长期需要的是 **Android-first 的 JVM 侧 Wasm runtime**，而不仅是给 wasi:webgpu 打补丁——轨 B 以此为产品愿景，短期仍以薄 L1 验证路径。

### 1.3 设计隐喻（继承轨 A）

> 先造 **灯的线路（L2 Host）**，再换 **插座（L1 Runtime）**。

轨 B = 新插座；灯（Dawn / L2）尽量不动。

---

## 2. 愿景与目标堆叠

### 2.1 长期愿景（产品）

**专为 Android 适配的 Java/Kotlin Wasm 运行时**：

- 以 **Component Model** 为一等公民（不只是 core wasm）
- 一等支持 **CM async**（future / 必要的 concurrent host）
- 原生面向 **Bionic / ART / 多 ABI**（arm64-v8a 优先，x86_64 模拟器次之）
- API 面以 **Kotlin/Java 友好** 为先（明确生命周期、线程、错误类型）
- 可托管一类 WASI / 自定义 world；**首发场景**仍是 webgpu Host 胶水，但不把运行时绑死在单一 world

仍为 **experimental** 直至单独宣布；**不**自动等于合规 wasi:webgpu 产品。

### 2.2 短期目标（验证路径）

**自研薄 L1**：

- 依赖 **官方 `wasmtime` crate**（钉版本，与轨 A 可对齐或略新）
- 自研 **JNI + 最小 Java/Kotlin API**（不经 wasmtime4j）
- 能：`compile/instantiate` component → 注册 host imports（含 resource rep）→ 调用 export
- 能：至少一条 **真 CM async** host import（future complete/reject）
- 能：在 Android 真机上与轨 A **同一 L2** 联调（先 smoke，后追齐 cube 子集）

### 2.3 目标堆叠（由下到上）

```text
L0  工程骨架（Gradle/Android NDK/Rust cdylib、CI 草稿）     ← 代码期 M0
L1a 同步 CM：instantiate + sync host + 调 export            ← M1
L1b 真 CM async：FutureReader + concurrent host + smoke     ← M2
L1c 接轨 A L2：adapter/device/map 或 experimental 子集      ← M3
L1d Android 上屏：cube 子集或专用 smoke Guest               ← M4
L2* 运行时产品化：文档、ABI 稳定、多 ABI、错误模型           ← M5+（长期）
```

`*` 表示超出「薄 L1」进入「Android Wasm runtime」产品层；**不**阻塞 M1–M4。

### 2.4 成功标准（阶段性）

| 阶段 | 成功长什么样 |
|------|----------------|
| 短期成功 | Android 上一条真 async host import e2e 绿灯；轨 A CI/cube **零回归** |
| 中期成功 | 同一 L2 上可跑与轨 A 对等的 CM Guest 子集（或明确差距清单） |
| 长期成功 | 第三方能以「Android JVM Wasm runtime」理解本仓，而不必先理解 wasmtime4j |

---

## 3. 非目标（摘要）

完整表见 [`non-goals.md`](non-goals.md)。要点：

- **不**在轨 B 初期替换轨 A 主验收 / Demo  
- **不** fork 维护完整 wasmtime4j 作为主路径  
- **不**重造完整 Kotlin WebGPU 客户端 API（L2 原则继承）  
- **不**以完整 WASI Preview3 / wasi-http 为短期关门条件  
- **不**宣传已合规 wasi:webgpu；**不**默认对外发布  
- **不**在无里程碑证据前把轨 A 依赖切到本仓  

---

## 4. 架构原则

1. **L2 不依赖 L1**（继承轨 A）：本仓 native **回调**进 Kotlin L2，L2 不 import 本仓实现细节。  
2. **Android-first**：桌面仅作开发便利；门禁与设计以真机为准。  
3. **薄绑定**：Java API 暴露「引擎 / store / linker / instance / future / resource」最小集；避免过早模仿 4j 全表面。  
4. **官方语义优先**：async 走 `func_wrap_concurrent` + `FutureProducer`/`FutureReader`，禁止再发明 sync-compat 当「真异步」。  
5. **双轨隔离**：独立 git 仓、独立 CI；轨 A 锁 sync-compat 的文档口径不变。  
6. **共享知识、隔离产物**：可参考轨 A 的补丁经验（JNI_VERSION、TBI、resource↔u32），但 **不**把 4j jar 当运行时依赖。  
7. **线程诚实**：Dawn `processEvents`、Surface、CM event loop 的同线程契约写进 [`threading-android.md`](../mapping/threading-android.md)。

### 4.1 模块切分（M0 已建）

```text
wasmtime-android-kt/
  runtime-api/          # Kotlin/Java 公共 API（无 Android 依赖）
  runtime-jni/          # JNI 封装
  native/               # Rust cdylib（wasmtime + 桥）
  android/              # AAR / jniLibs 打包（arm64 / x86_64）
  smoke-app/            # 最小 Android 仪器 / loadLibrary smoke
  scripts/              # build-native-android.ps1
  docs/                 # 章程 + build.md
```

与轨 A 的集成形态（后期）：

- **选项 α**：轨 A `runtime-wasmtime` 旁路依赖本仓 AAR（feature flag）  
- **选项 β**：轨 A 仅通过接口适配器调用，本仓保持独立 Demo  

章程默认倾向 **α（可选依赖）**，但 **切主验收前必须双轨并行绿**。

---

## 5. 技术依赖（概要）

详见 [`tech-stack.md`](tech-stack.md)。

| 类别 | 选型（初订） | 备注 |
|------|----------------|------|
| Wasm 引擎 | 官方 `wasmtime` crate（建议钉 **47.x** 与轨 A 同源或文档说明差异） | `component-model` + `component-model-async` + `async` |
| 语言桥 | Rust cdylib + JNI | 首期不做 Panama（Android ART 主路径是 JNI） |
| JVM | JDK 17+ 构建；Android minSdk 与轨 A Demo 对齐（待代码期钉死） | |
| Android NDK | 与轨 A `build-wasmtime4j-android.ps1` 经验对齐 | Bionic；注意 `JNI_OnLoad` 版本 |
| Host（短期） | 依赖轨 A 已发布/本地的 `host-api` / `host-webgpu`（experimental） | **不**把 Dawn 重新实现进本仓 |
| Guest | 新建最小 async smoke；cube 复用轨 A `guest/cube-cm`（只读依赖） | |
| 构建 | Gradle + cargo-ndk | [`../build.md`](../build.md)；`scripts/build-native-android.ps1` |

---

## 6. 里程碑（摘要）

详见 [`milestones.md`](milestones.md)。

| ID | 名称 | 一句话 DoD |
|----|------|------------|
| **M0** | 仓与构建骨架 | Android `.so` 可加载；`JNI_OnLoad` OK |
| **M1** | 同步 CM 最小环 | 假 world：host sync import + guest export 往返 |
| **M2** | 真 CM async | 一条 future complete/reject e2e（可假 payload） |
| **M3** | 接 L2 | 至少 `request-adapter` 或 experimental 等价经本 L1→L2 |
| **M4** | Android 图形 smoke | cube 子集或专用 Guest 上屏；**不**取代轨 A 仪器门禁 |
| **M5** | 运行时硬化 | 错误模型、多 ABI、文档、可选桌面开发壳 |

**硬序：** M0 → M1 → M2（M2 失败则停 L2 接线，先修 runtime）。M3/M4 依赖 M2。

---

## 7. 与轨 A 的契约

详见 [`dual-track.md`](dual-track.md)。

| 项 | 约定 |
|----|------|
| 轨 A async | **锁死 sync-compat**；不再为真 CM async 改 Linker/L2 |
| 轨 A 主验收 | 仍 `run-android-instrumented.ps1` + CM cube |
| 共享 | L2 API、ABI 常量、映射文档、Guest wasm 字节（只读） |
| 不共享 | wasmtime4j jar、4j 补丁构建产物作为轨 B 运行时 |
| 升级 | 轨 B 成熟后，轨 A **可选**切换 L1；切换是独立 RFC，不是默认 |

---

## 8. 风险与缓解

| 风险 | 等级 | 缓解 |
|------|------|------|
| Concurrent host + JNI 线程模型难 | 高 | M2 单测先桌面/JVM；Android 仪器第二；契约文档先写 |
| Resource ↔ u32 再踩 4j 同类坑 | 高 | 直接吸收轨 A cm-resources 设计；从第一天走 rep 模型 |
| ART / TBI / JNI_VERSION | 中 | 复用轨 A android patch 经验清单 |
| 双轨人力稀释 | 中 | 轨 A 只做稳性/文档；轨 B 固定里程碑，禁止无 DoD 扩张 |
| Wasmtime 大版本 API 晃动 | 中 | 钉 crate 版本；changelog 跟踪 `component` concurrent |
| 范围膨胀成「完整 WASI」 | 高 | [`non-goals.md`](non-goals.md) 硬表；P3 不作关门 |

---

## 9. 合规与宣称

- 包名建议（代码期再钉）：`io.github.fenriliuguang.wasmtime.android` 或同类  
- README / POM：**experimental**；未达标前 **不得**宣称「生产级 Android Wasm runtime」或「合规 wasi:webgpu」  
- 对外发布：默认 **否**；与轨 A 相同策略  

---

## 10. 初始化交付清单

- [x] 仓库目录 `d:\projects\wasmtime-android-kt`  
- [x] 根 README（ZH/EN）  
- [x] 本章程 + dual-track / tech-stack / milestones / non-goals / threading-android  
- [x] CHANGELOG  
- [x] M0 Gradle 多模块 + `native/` cdylib + cargo-ndk 脚本 + `docs/build.md`  
- [x] M0 本机交叉编译 `arm64-v8a` `.so` + Gradle `assembleDebug`  
- [ ] M0 ART 仪器绿灯（见 [`milestones.md`](milestones.md)；需设备）  

---

## 11. 链接

- 轨 A 闸门：[`archive-true-cm-async-dod`](../../../wasi-webgpu-jvm-mvp/docs/scheme/archive-true-cm-async-dod.md)  
- 轨 A 线程：[`threading.md`](../../../wasi-webgpu-jvm-mvp/docs/mapping/threading.md)  
- Wasmtime component async API：`FutureProducer` / `FutureReader` / `func_wrap_concurrent`  
- Component Model async 说明：https://component-model.bytecodealliance.org/design/async.html  
