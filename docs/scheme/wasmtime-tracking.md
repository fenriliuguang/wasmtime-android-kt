# 官方 Wasmtime 依赖追踪

**中文** | （暂无 EN）

> 配套 [`long-term-plan.md`](long-term-plan.md) **P2** · [`tech-stack.md`](tech-stack.md)。  
> 原则：**只依赖官方 `wasmtime`（及同组织显式选用的官方附属 crate）**；不依赖 wasmtime4j。

## 1. 追踪什么

| 类别 | 内容 |
|------|------|
| 版本 | crates.io / GitHub release 的 `wasmtime` semver |
| Features | `component-model`、CM async、WASI 相关 feature（若启用 `wasmtime-wasi` 等须单列） |
| CM async API | `func_wrap_concurrent`、`FutureReader`/`FutureProducer`、`run_concurrent`、**stream** 相关 API |
| WASI 0.3 | 上游对 P3 worlds / `wasmtime-wasi` 的默认启用与 breaking |
| Android / 链接 | NDK、页大小、体积、依赖 libc；发行说明中的平台注意 |
| 安全 | RustSec / 上游安全公告 |

本仓 **不**把「追最新 major」当 KPI；把「可知、可升级、可回滚」当 KPI。

## 2. 当前钉死（基线）

| 项 | 值（2026-08-11） | 出处 |
|----|------------------|------|
| `wasmtime` | **47.0.2** | `native/Cargo.toml` |
| 对齐意图 | 与轨 A wasmtime4j 所钉 Wasmtime **代际**一致 | [`tech-stack.md`](tech-stack.md) |
| Features（摘要） | `component-model` + async 相关（见 Cargo.toml） | 构建时以 lockfile 为准 |
| 产物 | `libwasmtime_android_kt.so` / 桌面 cdylib | [`../mapping/artifacts.md`](../mapping/artifacts.md) |

升级后必须同步改：本表基线、`tech-stack.md`、CHANGELOG、必要时 `docs/build.md`。

## 3. 跟踪表（活页）

> 每次评估上游或升级前更新「上次核查」行；不必为每次上游 patch 改代码。

| 字段 | 当前记录 |
|------|----------|
| 上次核查日期 | 2026-08-11 |
| 本仓钉版本 | 47.0.2 |
| 上游最新稳定（核查时） | （填写 crates.io / GitHub；文档期未强制拉网） |
| WASI 0.3 / CM async 默认 | 上游自 46 起将 WASI 0.3.0 + CM async 作为主线能力（见 BA 宣布）；本仓 47.x 已用 CM async |
| 与长期计划相关的缺口 | **stream** JNI/Kotlin 面未暴露；WASI package 未接 `wasmtime-wasi` |
| 已知风险 | major 升级可能改 concurrent API；Android 交叉编译需回归 M0 加载 + M2 async |
| 下一评估触发 | 见 §5 |

### 3.1 关注上游信号（清单）

- [Wasmtime 发布说明](https://github.com/bytecodealliance/wasmtime/releases)  
- [Bytecode Alliance / WASI 0.3](https://bytecodealliance.org/articles/WASI-0.3)  
- docs.rs：`wasmtime::component` concurrent / stream  
- （可选）`wasmtime-wasi` 对 P3 的 feature 与 Android 可用性  
- 轨 A 是否仍钉同一 Wasmtime 代际（双轨文档说明差异即可，不强制同 patch）

## 4. 升级策略

### 4.1 级别

| 变更 | 流程 |
|------|------|
| **patch**（47.0.2 → 47.0.x） | 更新 Cargo；跑 native 构建 + 既有仪器/ JVM smoke；CHANGELOG；更新本表 |
| **minor**（若上游有） | 同 patch + 浏览 component/WASI release notes |
| **major**（47 → 48+） | **升级 RFC**（短文即可）：动机、API  diff、回归清单、回滚钉版；合并前双 ABI 构建 |

### 4.2 回归最低集（升级门禁）

1. `scripts/build-native-android.ps1`（至少 arm64；正式发版双 ABI）  
2. M0 等价：`loadLibrary` + `nativeWasmtimeVersion`  
3. M2 等价：真 CM async future smoke  
4. 若已接 WASI/webgpu 切片：对应仪器至少一条  
5. `scripts/verify-native-android.ps1`（发版或 `-RequireAll` 时）

### 4.3 禁止

- 引入 `ai.tegmentum:wasmtime4j` 或 4j native 作运行时  
- 无 RFC 的 major 跳跃  
- 为「用上某 WASI 实验 feature」而破坏 Android 主路径构建  

## 5. 评估节奏

| 触发 | 动作 |
|------|------|
| 本仓准备开 L1 stream / WASI package 切片前 | 核查上游 API 是否稳定；更新 §3 |
| 上游安全公告涉及 `wasmtime` | 立即评估 patch |
| 轨 A 或提案 WIT 要求更新代际 | 开 major/minor 升级 RFC |
| 每季度（建议） | 刷新「上游最新稳定」一行，即使不升级 |

## 6. 与附属 crate

| Crate | 政策 |
|-------|------|
| `wasmtime` | **必须** |
| `wasmtime-wasi` / WASI 预置 Host | **可选**；启用前写切片 RFC（体积、线程、Android 文件系统语义） |
| `cranelift-*` | 经 `wasmtime` 传递；不直接钉除非排障需要 |
| 第三方 `wasi-webgpu-wasmtime` | **对照实现 only**；非本仓 cdylib 依赖 |

## 7. 修订

- 基线版本变更：本页 §2 + tech-stack + Cargo + CHANGELOG 同 PR。  
- 改变「只跟官方 Wasmtime」政策：章程级 RFC。  
