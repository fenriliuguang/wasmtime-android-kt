# 里程碑与 DoD（轨 B）

**中文** | [English](milestones.en.md)

> 与 [`charter.md`](charter.md) 目标堆叠对应。未勾选 = 未完成。  
> 构建复现：[`../build.md`](../build.md)。

## 总序

```text
M0 骨架 → M1 同步 CM → M2 真 async（硬闸门）→ M3 接 L2 → M4 Android 上屏 → M5 硬化
```

**M2 失败 ⇒ 停止 M3/M4 的 L2/图形投入**，先修 runtime（与轨 A true-cm-async 闸门同构，但底座换成官方 Wasmtime）。

---

## M0 — 仓与构建骨架

**目的：** 证明 Android 能加载「我们的」Wasmtime cdylib。

### DoD

- [x] Gradle 多模块骨架 + `native/` Rust cdylib  
- [x] `cargo ndk` 产出 `arm64-v8a` `.so`（`scripts/build-native-android.ps1`）  
- [x] `JNI_OnLoad` 在 ART 成功；仪器 `loadLibrary` + `nativeWasmtimeVersion`（真机 arm64 已验）  
- [x] README「如何构建」可复现（[`../build.md`](../build.md)）  
- [x] CHANGELOG  

### 非目标

- 尚不 instantiate component  
- 不接 L2 / Dawn  

---

## M1 — 同步 Component Model 最小环

**目的：** 无 async 的端到端 CM 闭环。

### DoD

- [x] Engine/Store/Linker/Component/Instance 最小 Kotlin API（slice 1）  
- [ ] 注册至少 1 个 sync host import + 1 个 guest export  
- [x] 假 world 往返断言（仪器：`add_one` export `run` → `a+1`）  
- [ ] Resource：至少 1 种 host resource，**u32 rep** 进出（可为假）  
- [ ] CHANGELOG（slice 1 已记；M1 关门再勾）  

### 非目标

- 真 async、Dawn、cube  

---

## M2 — 真 CM async（硬闸门）

**目的：** 证明官方 Wasmtime async 经自研 JNI 可用——这是轨 B 相对轨 A 的核心存在理由。

### DoD

- [ ] Host 经官方 concurrent API 注册 **async** import  
- [ ] Host 可 **创建并 complete / reject** future（假 payload 可）  
- [ ] Guest（最小 async smoke）观察到完成值或拒绝  
- [ ] 文档：线程模型初稿（谁驱动 `run_concurrent`）  
- [ ] CHANGELOG  
- [ ] **闸门：** 若不可行，书面记录根因并暂停 M3+  

### 非目标

- 扫完所有 WIT async；wasi-p3；接 Dawn  

---

## M3 — 接轨 A L2

**目的：** 薄 L1 插上已有灯的线路。

### DoD

- [ ] 依赖轨 A `host-api`（及必要 abi 常量）  
- [ ] 至少一条主链：`request-adapter` **或** experimental `request-adapter` 经本 L1 → L2（Cpu 或 Dawn）  
- [ ] 错误映射策略文档（trap vs result；可先 subset）  
- [ ] 轨 A 主 CI / cube **仍绿**（本仓变更不强制改轨 A 门禁）  
- [ ] CHANGELOG  

### 非目标

- 全量 wasi:webgpu world；替换轨 A Linker  

---

## M4 — Android 图形 smoke

**目的：** 真机可见；仍非轨 A 主验收替换。

### DoD

- [ ] 在 Android + Dawn L2 上跑通：cube 子集 **或** 专用 render smoke Guest  
- [ ] 遵守 [`threading-android.md`](../mapping/threading-android.md)  
- [ ] 独立仪器用例 / 脚本（名称勿覆盖轨 A 主脚本职责）  
- [ ] 差距清单：相对轨 A cube 缺什么  
- [ ] CHANGELOG  

### 非目标

- 宣布取代 `WasmtimeCmCubeInstrumentedTest`  
- wasi-gfx  

---

## M5 — 运行时硬化（长期入口）

**目的：** 从「webgpu 薄 L1」迈向「Android Wasm runtime」产品层。

### DoD（可拆子切片）

- [ ] 稳定错误类型与文档  
- [ ] arm64 + x86_64 正式产物布局  
- [ ] API 稳定性政策（experimental 下的 semver 约定）  
- [ ] 贡献者构建文档；可选桌面开发壳  
- [ ] 是否支持更多 WASI world 的路线图（单独 RFC）  

### 非目标

- 一次做完所有 WASI；合规认证  

---

## 跟踪表（状态）

| 里程碑 | 状态 | 日期 |
|--------|------|------|
| 文档立项 | **完成** | 2026-08-10 |
| M0 | 骨架 + arm64 `.so` + Gradle 绿；ART 仪器待设备 | 2026-08-10 |
| M1 | 未开工 | — |
| M2 | 未开工 | — |
| M3 | 未开工 | — |
| M4 | 未开工 | — |
| M5 | 未开工 | — |
