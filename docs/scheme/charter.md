# 章程：wasmtime-android-kt（轨 B）

**中文** | [English](charter.en.md)

> **状态：短期 M0–M5 已归档（2026-08-11）；现行见 [`long-term-plan.md`](long-term-plan.md)。**  
> **2026-08-16：** 结束双轨并行排期；轨 A = 展示 Demo；本仓靠拢官方 wasi:webgpu 形状 → [`rfc-wasi-webgpu-canonical-shape.md`](rfc-wasi-webgpu-canonical-shape.md)。  
> 姊妹仓轨 A：[`wasi-webgpu-jvm-mvp`](../../../wasi-webgpu-jvm-mvp) — **锁死 sync-compat** 的 experimental cube Demo。  
> 索引：[`README.md`](../../README.md) · [`../build.md`](../build.md) · [`dual-track.md`](dual-track.md) · [`tech-stack.md`](tech-stack.md) · [`long-term-plan.md`](long-term-plan.md) · [`non-goals.md`](non-goals.md)

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
2. 轨 A 必须保持 **可演示**（现为简单 Demo）；不宜把它的默认验收绑在未验证的新 runtime 上。  
3. 官方 Wasmtime 已提供 `FutureProducer` / `FutureReader`、`func_wrap_concurrent`、`run_concurrent` 等 API，适合作为 Android 薄 L1 的底座。  
4. 长期需要的是 **Android-first 的 JVM 侧 Wasm runtime**，而不仅是给 wasi:webgpu 打补丁——本仓以此为产品愿景。**Guest 形状以钉版 `wasi:webgpu` WIT 为准，不再跟随轨 A experimental 扁平面。**

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

### 2.2 短期目标（验证路径）— **已完成并归档**

**自研薄 L1**（M0–M5，2026-08-11 收口）：

- 依赖 **官方 `wasmtime` crate**（钉版本，与轨 A 可对齐或略新）
- 自研 **JNI + 最小 Java/Kotlin API**（不经 wasmtime4j）
- 能：`compile/instantiate` component → 注册 host imports（含 resource rep）→ 调用 export
- 能：至少一条 **真 CM async** host import（future complete/reject）
- 能：在 Android 真机上与轨 A **同一 L2** 联调（Dawn clear→present smoke）

归档：[`archive/m0-m5-thin-l1.md`](archive/m0-m5-thin-l1.md) · 史实 DoD：[`milestones.md`](milestones.md)。

### 2.3 长期目标堆叠（现行）

详见 [`long-term-plan.md`](long-term-plan.md)。摘要：

```text
L0  底座冻结与 Wasmtime 追踪机制
L1  WASI 0.3 原语完备（含 stream）
L2  WASI 0.3 核心 package 子集（按需）
L3  wasi:webgpu 提案主链（P0；**规范 WIT 形状**；真 async）
L4  （可选）轨 A Demo 换默认 runtime——独立 RFC，非 L3 前置
L5  运行时产品化门槛
```

战略硬序：**wasi:webgpu（提案）→ WASI 0.3 正式面 → 官方 Wasmtime 追踪**。

### 2.4 成功标准（阶段性）

| 阶段 | 成功长什么样 |
|------|----------------|
| 短期成功（已达成） | Android 上真 async host import e2e 绿灯；轨 A CI/cube **零回归** |
| 中期成功 | WASI 0.3 原语（含 stream）可承载；`wasi:webgpu` **Guest 为钉版 WIT 类型**，真 async 仪器绿灯 |
| 长期成功 | 第三方以「Android 上跟官方 Wasmtime / WASI 0.3、优先官方形状 wasi:webgpu」理解本仓，而不必先理解 wasmtime4j 或轨 A 扁平面 |

---

## 3. 非目标（摘要）

完整表见 [`non-goals.md`](non-goals.md)。要点：

- **不**静默替换轨 A Demo 默认 runtime  
- **不**以 wasmtime4j 为运行时依赖；**只**追踪官方 Wasmtime  
- **不**重造完整 Kotlin WebGPU 客户端 API / 第二套 Dawn  
- **不**以「全量 WASI 0.3 套件 / 全量 testsuite」为单一 KPI（**主推**已批准 P3 **切片** + **wasi:webgpu 提案**）  
- **不**在未达标前宣传合规 wasi:webgpu / 生产级 runtime；**不**默认对外发布  
- **不**用 sync-compat 冒充真 CM async / WASI 0.3 异步  
- **不**再以 host-fixed 过渡 u32 作为 wasi:webgpu 新切片验收形态（NG-12）  
- **不再**与轨 A 并行推进同一条 Guest ABI  

---

## 4. 架构原则

1. **L2 不依赖 L1**（继承轨 A）：本仓 native **回调**进 Kotlin L2，L2 不 import 本仓实现细节。  
2. **Android-first**：桌面仅作开发便利；门禁与设计以真机为准。  
3. **薄绑定**：Java API 暴露「引擎 / store / linker / instance / future / resource」最小集；避免过早模仿 4j 全表面。  
4. **官方语义优先**：async 走 `func_wrap_concurrent` + `FutureProducer`/`FutureReader`，禁止再发明 sync-compat 当「真异步」。  
5. **轨 A = Demo**：独立 git 仓、独立 CI；不静默替换其默认 runtime；本仓 **拥有** wasi:webgpu 编组。  
6. **共享知识、隔离产物**：可参考轨 A 的补丁经验（JNI_VERSION、TBI、resource↔u32），但 **不**把 4j jar 当运行时依赖。  
7. **线程诚实**：Dawn `processEvents`、Surface、CM event loop 的同线程契约写进 [`threading-android.md`](../mapping/threading-android.md)。  
8. **形状合格**：新 `[method]` 与钉版 WIT 同构（RFC）；禁止再扩 host-fixed u32。

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

与轨 A 的集成形态（可选，非产品主线）：

- **选项 α**：轨 A `runtime-wasmtime` 旁路依赖本仓 AAR（feature flag）  
- **选项 β**：轨 A 仅通过接口适配器调用，本仓保持独立 Demo  

章程默认倾向 **α（可选依赖）**。**切轨 A 默认 runtime 前必须单独 RFC**；这 **不是** 本仓推进规范 wasi:webgpu 的前置。

---

## 5. 技术依赖（概要）

详见 [`tech-stack.md`](tech-stack.md)。

| 类别 | 选型（初订） | 备注 |
|------|----------------|------|
| Wasm 引擎 | 官方 `wasmtime` crate（建议钉 **47.x** 与轨 A 同源或文档说明差异） | `component-model` + `component-model-async` + `async` |
| 语言桥 | Rust cdylib + JNI | 首期不做 Panama（Android ART 主路径是 JNI） |
| JVM | JDK 17+ 构建；Android minSdk 与轨 A Demo 对齐（待代码期钉死） | |
| Android NDK | 与轨 A `build-wasmtime4j-android.ps1` 经验对齐 | Bionic；注意 `JNI_OnLoad` 版本 |
| Host（现行） | 依赖轨 A `host-api` / `host-webgpu` 当 **后端库**（experimental） | **不**把 Dawn 重新实现进本仓；**编组由本仓拥有** |
| Guest | 规范路径：与钉版 `wasi:webgpu` WIT 同构；cube 仅 Demo / 遗留 | |
| 构建 | Gradle + cargo-ndk | [`../build.md`](../build.md)；`scripts/build-native-android.ps1` |

---

## 6. 里程碑（摘要）

| 世代 | 文档 |
|------|------|
| **现行** L0–L5 | [`long-term-plan.md`](long-term-plan.md) |
| **已归档** M0–M5 | [`milestones.md`](milestones.md) · [`archive/m0-m5-thin-l1.md`](archive/m0-m5-thin-l1.md) |

短期史实硬序（已完成）：M0 → M1 → M2（真 async 闸门）→ M3 → M4 → M5。

---

## 7. 与轨 A 的契约

详见 [`dual-track.md`](dual-track.md)。

| 项 | 约定 |
|----|------|
| 轨 A | **展示 Demo**；async **锁死 sync-compat**；不再为真 CM async 改 Linker/L2 |
| 轨 A 默认 runtime | 仍 4j + CM cube；切换须独立 RFC（NG-1） |
| 共享 | Host **库**、补丁经验、映射史实 |
| 不共享 | wasmtime4j；experimental 扁平 ABI 作为本仓目标形状 |
| 形状 | 本仓按 `wasi:webgpu@0.3.0-rc.2` WIT；见 RFC |

---

## 8. 风险与缓解

| 风险 | 等级 | 缓解 |
|------|------|------|
| Concurrent host + JNI 线程模型难 | 高 | M2 单测先桌面/JVM；Android 仪器第二；契约文档先写 |
| Resource ↔ u32 再踩 4j 同类坑 | 高 | 直接吸收轨 A cm-resources 设计；从第一天走 rep 模型 |
| ART / TBI / JNI_VERSION | 中 | 复用轨 A android patch 经验清单 |
| 双轨人力稀释 | 中 | **已结束并行产品线**：轨 A 只作 Demo；本仓按 RFC S 系列推进 |
| Wasmtime 大版本 API 晃动 | 中 | 钉 crate 版本；changelog 跟踪 `component` concurrent |
| 范围膨胀成「完整 WASI 套件」 | 高 | [`non-goals.md`](non-goals.md) NG-4；按 [`wasi-p3-surface.md`](wasi-p3-surface.md) 切片 |
| Wasmtime major 晃动 | 中 | [`wasmtime-tracking.md`](wasmtime-tracking.md) 升级 RFC |
| wasi:webgpu 提案 WIT 漂移 | 中 | [`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) 钉版 + gap 表 |

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

- 计划变更 RFC：[`rfc-wasi-webgpu-canonical-shape.md`](rfc-wasi-webgpu-canonical-shape.md)  
- WASI 0.3 表面：[`wasi-p3-surface.md`](wasi-p3-surface.md)  
- wasi:webgpu 路线：[`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md)  
- Wasmtime 追踪：[`wasmtime-tracking.md`](wasmtime-tracking.md)  
- 短期归档：[`archive/m0-m5-thin-l1.md`](archive/m0-m5-thin-l1.md)  
- 轨 A 闸门：[`archive-true-cm-async-dod`](../../../wasi-webgpu-jvm-mvp/docs/scheme/archive-true-cm-async-dod.md)  
- WASI 0.3：https://wasi.dev/releases/wasi-p3 · https://bytecodealliance.org/articles/WASI-0.3  
- wasi:webgpu：https://github.com/WebAssembly/wasi-webgpu  
- Component Model async：https://component-model.bytecodealliance.org/design/async.html  
