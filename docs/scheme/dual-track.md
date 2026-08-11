# 双轨契约（轨 A ↔ 轨 B）

**中文** | [English](dual-track.en.md)

> 与章程 [`charter.md`](charter.md) 配套。约束两仓如何并行而不互相拖垮。

## 1. 角色

| 轨 | 仓库路径 | 一句话 |
|----|----------|--------|
| **A** | `d:\projects\wasi-webgpu-jvm-mvp` | experimental wasi:webgpu Host MVP；**L1=wasmtime4j**；**async=sync-compat（锁死）** |
| **B** | `d:\projects\wasmtime-android-kt` | Android-first JVM Wasm runtime；**L1=官方 Wasmtime 自研薄绑定**；目标真 CM async |

## 2. 锁死条款（轨 A）

自 2026-08-10 起，轨 A **明确锁死**：

1. 默认与主验收路径保持 **sync-compat**（`requestAdapter` / `requestDevice` / `mapAsync` 等可继续内部等待）。  
2. **不再**为「真 CM async」改 `DawnWasiWebGpuHost` 主回调路径、`WasmtimeCmLinker` 主链 future、或迁仪器到 async Guest。  
3. 真 CM async 闸门归档保持有效：[`archive-true-cm-async-dod.md`](../../../wasi-webgpu-jvm-mvp/docs/scheme/archive-true-cm-async-dod.md)。  
4. 轨 A 可继续：稳性、缺口矩阵非 async 项、文档、工程债、**不依赖** 4j future writer 的增强。  
5. 若未来切换 L1 到轨 B，必须单独 RFC + 双轨绿灯，**不得**静默替换。

## 3. 共享面（允许）

| 资产 | 方式 |
|------|------|
| L2 `host-api` / `host-webgpu` | 轨 B 以 **依赖**（mavenLocal / composite / 源码路径）消费；不复制 Dawn 逻辑 |
| `abi-cm` / `abi-wasi` 常量与结果形状 | 依赖或生成同源；避免分叉字符串 |
| Guest `cube_cm.wasm` / WIT | 只读引用轨 A 路径或复制字节并注明来源版本 |
| 映射文档（threading / errors / gap） | 轨 B 可链到轨 A；差异写在本仓 `docs/mapping` |
| 补丁**经验**（JNI_VERSION、TBI、resource↔u32） | 文字迁移到轨 B native 设计；**不是**依赖 4j `.so` |

## 4. 隔离面（禁止混用）

| 禁止 | 原因 |
|------|------|
| 轨 B runtime 依赖 `ai.tegmentum:wasmtime4j` | 双轨意义丧失 |
| 轨 A CI 强制构建轨 B native | 拖慢主验收 |
| 未成熟时把轨 A Demo 默认切到轨 B | 破坏真机基线 |
| 在轨 A 内嵌整份轨 B 源码树当子模块「临时」 | 边界模糊；应用独立仓 + 明确版本 |
| 用 sync-compat「冒充」轨 B 的真 async DoD | 重复闸门失败模式 |

## 5. 集成策略（代码期）

### 5.1 短期（M1–M3）

- 轨 B 自带 smoke；可选 Android 空壳 Activity。  
- 与 L2 联调用 **显式依赖** 轨 A engineered artifacts（`publishEngineeredToMavenLocal`）或 Gradle `includeBuild`。  

### 5.2 中期（M4）

- 轨 A `android-demo` 可增加 **可选**「轨 B runtime」入口（feature / BuildConfig），默认仍 4j。  
- 仪器：轨 B 用例 **独立** class / 脚本；**不**替换 `run-android-instrumented.ps1` 主门禁。  

### 5.3 长期（切换 RFC）

切换条件草案：

1. M4 DoD 连续绿（约定设备）  
2. 线程 / 生命周期文档完整  
3. 与轨 A cube 对等的最小回归清单通过  
4. 书面 RFC：回滚方案、版本钉死、谁维护 natives  

## 6. 沟通口径

- 对外谈「wasi-webgpu Android Demo」→ **轨 A**。  
- 对外谈「Android JVM 上跑官方 Wasmtime CM async」→ **轨 B**。  
- 禁止把轨 B 计划进度写成轨 A「已支持真 async」。  

## 7. 变更流程

| 变更 | 落点 |
|------|------|
| sync-compat 行为 / cube 验收 | 仅轨 A |
| Wasmtime 版本、JNI、future API | 仅轨 B |
| L2 接口变更 | 轨 A 先改；轨 B 跟随；保持 semver/experimental 标注 |
| 轨 B 公共 API / 版本号 | 见 [`api-stability.md`](api-stability.md) |
| 双轨契约本身 | 两仓文档同步改（本页 + 轨 A 索引） |
