# API 稳定性政策（experimental）

**中文** | （暂无 EN）

> M5：在 **不** 对外 Maven Central 默认发布（NG-6）的前提下，约定本仓版本号与破坏性变更怎么写、怎么破。

## 1. 总则

1. 本仓在 **1.0.0 之前** 一律视为 **experimental**。  
2. 版本号遵循 **SemVer 2.0** 形状：`MAJOR.MINOR.PATCH[-prerelease]`。  
3. 当前坐标形态：`0.1.0-experimental`（根 `gradle.properties` → `wasmtime.android.version`；`smoke-app` `versionName`；`nativeRuntimeId`）。  
4. **不**宣称生产级 / 合规 wasi:webgpu 产品（见 [`non-goals.md`](non-goals.md) NG-5）。

## 2. `0.x` 规则（现行）

| 变更 | 版本怎么走 | 是否允许不公告就破 |
|------|------------|-------------------|
| 破坏 Kotlin/Java 公共 API、JNI 符号、错误类型语义 | 至少 **`0.MINOR+1.0`**；重大整包重塑可跳 MINOR | 允许，但 **必须** 写 CHANGELOG |
| 向后兼容增强（新 API / 新 ABI 产物字段） | `0.MINOR+1` 或 `0.x.PATCH`（小修） | — |
| 纯缺陷修复、文档、仪器 | `0.x.PATCH` | — |
| 仅内部 / `internal` / `native` 私有实现 | 可不升版；若行为可观察仍建议 PATCH + CHANGELOG | — |

`0.x` **不**提供「MINOR 内 API 冻结」承诺。依赖方应以 **钉死完整版本** + 读 CHANGELOG 为准。

## 3. 稳定性分层

| 层 | 示例 | `0.x` 稳定性 |
|----|------|----------------|
| **公开 Kotlin API** | `Engine` / `Store` / `Component` / `Linker` / `Instance`；`WasmtimeException` 族 | 可破；破则升 MINOR + CHANGELOG |
| **公开常量** | `NativeLibraryNames`；loadLibrary 名 | 视同公开 API |
| **JNI 入口** | `NativeBridge` `external` 方法（含签名） | 视同公开 API；改名/改参 = 破坏 |
| **experimental host 桥** | `ExperimentalWebGpuBridge`；扁平 import 名 | **最不稳定**；可随 M4 差距清单大改；仍写 CHANGELOG |
| **Guest fixture / 仪器** | `fixtures/m*`、`*InstrumentedTest` | 不构成库 API；改动不强制升版 |
| **轨 A L2 依赖** | `host-api` / `host-webgpu` / `abi-cm` | **后端库**坐标；升版说明钉到哪一版。Guest WIT 形状 **不**跟随 experimental 扁平面（见 RFC） |

## 4. 与轨 A 的版本关系

- 本仓 **不**与轨 A 锁同一 semver 数字。  
- Dawn / Host **库**版本：本仓 CHANGELOG 记所钉 `host-webgpu` 坐标。Host 能力缺口可在本仓调用层或轨 A 库补丁解决，**不是**「轨 A 先改扁平 ABI、本仓跟随」（见 [`dual-track.md`](dual-track.md) §7 · RFC）。  
- 产品 Guest 钉 `wasi:webgpu@0.3.0-rc.2`。`experimental:webgpu-cm@0.8.0` 仅遗留 / Demo；分叉写在差距文档。

## 5. 何时考虑 `1.0.0`

须 **单独 RFC**，建议同时满足：

1. 短期 M5 已归档（见 [`milestones.md`](milestones.md)）；长期须另满足 [`long-term-plan.md`](long-term-plan.md) L5 与公开 API 冻结清单 / 发布政策 

2. 公开 API 有冻结清单与弃用周期（至少一轮 MINOR）  
3. 双 ABI 产物与校验进入常规 CI  
4. 书面决定是否 / 如何对外发布（仍可保持非 Central）  

在此之前 **禁止** 把 `0.x-experimental` 说成「稳定 API」。

## 6. CHANGELOG 约定

其它文档里写「更新 CHANGELOG」均指新增 [`changelog/unreleased/<yyyy-mm-dd>-<slug>.md`](../../changelog/unreleased/README.md)，**不是**改根 `CHANGELOG.md`（并行短 PR 插 Unreleased 会冲突）。维护者用 `.\scripts\roll-changelog.ps1` 滚入。

- 破坏性变更用明确用语：`BREAKING` 或中文「破坏性」。  
- 错误类型 / 产物布局 / 本政策本身的修订记入碎片，随 roll-up / 版本发布章节归档。  
- 不把「仅 fixtures / 仪器」写成库 API 变更，除非影响公共模块行为。

## 7. 非目标（本政策）

- 不引入完整内部 semver bot / 自动发版流水线  
- 不承诺二进制兼容跨 MINOR 的 `.so`  
- 不规定 Maven `groupId` 最终坐标（未默认发布）  
