# RFC：可插拔 GPU 后端（默认 Dawn，产品可拆）

**状态：Accepted** · 2026-08-17  
[English](rfc-pluggable-gpu-backend.md) | **中文**

与英文冲突时以英文为准。

## 决策

- 本仓**提供** Dawn 接线；**测试运行 / 默认产品坐标**包含 Dawn 构件。  
- 核心 AAR **不含** Dawn `.so`。使用者可只依赖 runtime，自带符合 SPI 的 Host，或不接 GPU。  
- Guest 无 GPU：`gpu.request-adapter` → **`none`**（对标 WebGPU `null`），不是 trap，也不是「找不到资源」。  
- 日后上包管理器：**runtime** 与 **host-dawn** 分开提交；另提供二者的 **bundle** 作为默认依赖。  
- 「动态」= 进程内选择 classpath / 显式 `setWebGpuBackend`，不是运行时下载 `.so`。

## 模块

`android`（L1）→ 可选 `host-dawn`（SPI 实现 + Dawn `.so`）→ 默认 bundle `android-webgpu`。  
SPI 由本仓 `runtime-api` 拥有，不暴露外仓 `WasiWebGpuHost` 类型。第三方实现该 SPI，不自己编 Guest WIT。今日 androidx.webgpu / 捆绑 Dawn = `:host-dawn`；未来系统 API = 另一 `host-*` 模块。

解析顺序：显式接线 → ServiceLoader（优先 `id=dawn`）→ 无后端则 `none`。Linker 始终注册 `wasi:webgpu`。

代码落地：`:host-dawn` / `:android-webgpu`；未接线 `request-adapter` → `none`。**Vendor 已拍板：** 拷 mvp 的 Host Kotlin；Dawn `.so` 用 `androidx.webgpu`。见 [`../blocked-gpu-host.md`](../blocked-gpu-host.md)。
