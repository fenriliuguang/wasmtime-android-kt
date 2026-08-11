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

- [x] Engine/Store/Linker/Component/Instance 最小 Kotlin API  
- [x] 注册至少 1 个 sync host import + 1 个 guest export（`add` ← Kotlin；`run`）  
- [x] 假 world 往返断言（仪器：`add_one` / `host_add` / `widget_echo`）  
- [x] Resource：host `widget`，**u32 rep** 经 guest `run` 进出  
- [x] CHANGELOG  

### 非目标

- 真 async、Dawn、cube  

---

## M2 — 真 CM async（硬闸门）

**目的：** 证明官方 Wasmtime async 经自研 JNI 可用——这是轨 B 相对轨 A 的核心存在理由。

### DoD

- [x] Host 经官方 concurrent API 注册 **async** import（`func_wrap_concurrent("get")`）  
- [x] Host 可 **创建并 complete** future（`FutureReader` + oneshot；reject 路径保留 `Err`）  
- [x] Guest（`fixtures/m2/async_get`）观察到完成值 `42`（真机仪器）  
- [x] 文档：线程模型初稿（[`../mapping/threading-m2-async.md`](../mapping/threading-m2-async.md)）  
- [x] CHANGELOG  
- [x] **闸门：** 已通过 — 可进入 M3  

### 非目标

- 扫完所有 WIT async；wasi-p3；接 Dawn  

---

## M3 — 接轨 A L2

**目的：** 薄 L1 插上已有灯的线路。

### DoD

- [x] 依赖轨 A `host-api`（及 `abi-cm`；mavenLocal）  
- [x] 至少一条主链：experimental `request-adapter` 经本 L1 → L2（**Cpu**）  
- [x] 错误映射策略文档（[`../mapping/errors-m3.md`](../mapping/errors-m3.md)）  
- [x] 轨 A 主 CI / cube **仍绿**（本仓仅消费 engineered 坐标，未改轨 A）  
- [x] CHANGELOG  

### 非目标

- 全量 wasi:webgpu world；替换轨 A Linker  

---

## M4 — Android 图形 smoke

**目的：** 真机可见；仍非轨 A 主验收替换。

### DoD

- [x] 在 Android + Dawn L2 上跑通：**专用 render smoke Guest**（clear→present；`fixtures/m4/render_smoke`）  
- [x] 遵守 [`threading-android.md`](../mapping/threading-android.md)（GpuThread 合一）  
- [x] 独立仪器用例：`DawnRenderSmokeInstrumentedTest`（不覆盖轨 A cube 主验收）  
- [x] 差距清单：[`../mapping/gap-m4-vs-cube.md`](../mapping/gap-m4-vs-cube.md)  
- [x] CHANGELOG  

### 非目标

- 宣布取代 `WasmtimeCmCubeInstrumentedTest`  
- wasi-gfx  

---

## M5 — 运行时硬化（长期入口）

**目的：** 从「webgpu 薄 L1」迈向「Android Wasm runtime」产品层。

### DoD（可拆子切片）

- [x] 稳定错误类型与文档（`Wasmtime*Exception` + [`../mapping/errors.md`](../mapping/errors.md)；2026-08-11）  
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
| M0 | **完成**（真机 loadLibrary） | 2026-08-11 |
| M1 | **完成**（同步 CM） | 2026-08-11 |
| M2 | **完成**（真 CM async 闸门） | 2026-08-11 |
| M3 | **完成**（L1→Cpu L2 request-adapter） | 2026-08-11 |
| M4 | **完成**（Dawn clear→present smoke；真机 arm64 已验） | 2026-08-11 |
| M5 | 进行中（错误模型切片已落地） | 2026-08-11 |
