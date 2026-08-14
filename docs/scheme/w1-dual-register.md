# W1 刀切：`wasi:webgpu` 双注册（已交付）

**中文** | （暂无 EN）

> 路线图切片：**W1**（[`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) §4）。  
> 差距表：[`../mapping/gap-experimental-vs-wasi-webgpu.md`](../mapping/gap-experimental-vs-wasi-webgpu.md)。  
> **状态：已交付**（`feat/webgpu-w1-request-adapter`）。  
> **W2 进度：** `request-adapter` 与扁平 `adapter-request-device` 真 async **均已交付**（`feat/webgpu-w2-async-request-adapter` · `feat/webgpu-w2-async-request-device`）。  
> **W3 首片：** 已双注册过渡扁平 `device-get-queue`（**sync** getter，同 L2 u32）。终态 `[method]gpu-device.queue` / resource 表仍属后续 W3。  
> **W3：** 已双注册过渡扁平 `device-create-command-encoder`（**sync**，同 L2 u32）。终态 `[method]gpu-device.create-command-encoder` 仍属后续 W3。  
> **W3：** 已双注册过渡扁平 `command-encoder-finish`（**sync**，同 L2 u32）。终态 `[method]gpu-command-encoder.finish` 仍属后续 W3。  
> **W3：** 已双注册过渡扁平 `queue-submit1`（**sync** void，同 L2 单 buffer u32）。终态 `[method]gpu-queue.submit` / 提案 `list` 仍属后续 W3。  
> **W3：** 已双注册过渡扁平 `command-encoder-begin-render-pass-clear`（**sync**，同 L2 u32）。终态 `[method]gpu-command-encoder.begin-render-pass` 仍属后续 W3。  
> **W3：** 已双注册过渡扁平 `render-pass-end`（**sync** void，同 L2 u32）。终态 `[method]gpu-render-pass-encoder.end` 仍属后续 W3。  
> **W3 本片：** 已注册 WIT `gpu` + `get-gpu` + **`[method]gpu.request-adapter`**（真 async，resource self；返回仍为过渡 u32，非 `option<gpu-adapter>`）。扁平 `request-adapter` 仍保留。

## 1. 目的

在 **不撤** experimental 扁平路径的前提下，把提案 **`wasi:webgpu`** 的 package / interface 名挂上 Linker，使至少一条既有 L2 能力能以**提案坐标**被 Guest import。

| 保留 | 新增（W1） |
|------|------------|
| `experimental:webgpu-cm/host@0.8.0#request-adapter`（及现有扁平面） | `wasi:webgpu/webgpu@0.3.0-rc.2#request-adapter`（过渡扁平）→ 同一 L2 / u32 路径 |

W1 **不是**合规面、**不是**真 async、**不是**完整 resource 表。

## 2. 钉版（复述 W0；W1 重钉）

| 字段 | 值 |
|------|-----|
| 提案 package | **`wasi:webgpu@0.3.0-rc.2`** |
| tag / commit | **`v0.3.0-rc.2`** → `6a776bada0b66d3dbf9da304a49ff2947ce4e1f8` |
| 来源 | [WebAssembly/wasi-webgpu](https://github.com/WebAssembly/wasi-webgpu) · `wit/webgpu.wit`（见 gap 表） |

## 3. 交付形态：过渡扁平 `request-adapter`

| 路径 | instance | func |
|------|----------|------|
| experimental（不变） | `experimental:webgpu-cm/host@0.8.0` | `request-adapter` |
| W1 提案名（过渡） | `wasi:webgpu/webgpu@0.3.0-rc.2` | `request-adapter`（**非** `[method]gpu.request-adapter`） |

两条路径共享同一 L2 `exp_request_adapter` / u32；**W2 起**提案名路径为 `func_wrap_concurrent`，experimental 仍为 sync `func_wrap`。  
终态 resource 方法名属 **W3**。扁平 `adapter-request-device` 真 async **已交付**（同提案 instance；L2 `exp_adapter_request_device`）。扁平 `device-get-queue` sync **已交付**（同提案 instance；L2 `exp_device_get_queue`；非 `[method]gpu-device.queue`）。扁平 `device-create-command-encoder` sync **已交付**（同提案 instance；L2 `exp_create_command_encoder`；非 `[method]gpu-device.create-command-encoder`）。扁平 `command-encoder-finish` sync **已交付**（同提案 instance；L2 `exp_command_encoder_finish`；非 `[method]gpu-command-encoder.finish`）。扁平 `queue-submit1` sync **已交付**（同提案 instance；L2 `exp_queue_submit1`；单 buffer u32；非 `[method]gpu-queue.submit`）。扁平 `command-encoder-begin-render-pass-clear` sync **已交付**（同提案 instance；L2 `exp_begin_render_pass_clear`；Guest stub view `23`；非 `[method]gpu-command-encoder.begin-render-pass`）。扁平 `render-pass-end` sync **已交付**（同提案 instance；L2 `exp_render_pass_end`；void；非 `[method]gpu-render-pass-encoder.end`）。**W3 `[method]` 首片：** `get-gpu` + `[method]gpu.request-adapter` 真 async **已交付**（resource `gpu`；返回过渡 u32；扁平 `request-adapter` 仍在）。

### 3.2 异步边界（硬约束，仍有效）

| 允许 | 禁止 |
|------|------|
| **W2** 提案名 `request-adapter` / `adapter-request-device`：`func_wrap_concurrent` + oneshot/helper-thread yield → L2 | 用 Latch / 假 future **冒充**提案 `async func` |
| experimental 路径继续 **`func_wrap` sync + u32** | 把 W2 仪器文案写成「合规 wasi:webgpu」 |

**真 async（`func_wrap_concurrent`）是 W2 硬闸门；adapter / device 主链已过闸。**

## 4. 落地清单

| 项 | 路径 |
|----|------|
| Linker 双注册 | `native/src/cm.rs` |
| Guest | `fixtures/w1/webgpu_request_adapter.{wat,wasm}` · `webgpu_request_device.{wat,wasm}` · `webgpu_device_get_queue.{wat,wasm}` · `webgpu_create_command_encoder.{wat,wasm}` · `webgpu_command_encoder_finish.{wat,wasm}` · `webgpu_queue_submit.{wat,wasm}` · `webgpu_begin_render_pass.{wat,wasm}` · `webgpu_render_pass_end.{wat,wasm}` · `webgpu_method_request_adapter.{wat,wasm}` |
| Native smoke | `native/tests/wasi_webgpu_request_adapter.rs`（stub u32=7）· `wasi_webgpu_request_device.rs`（stub device=11）· `wasi_webgpu_device_get_queue.rs`（stub queue=13）· `wasi_webgpu_create_command_encoder.rs`（stub encoder=17）· `wasi_webgpu_command_encoder_finish.rs`（stub buffer=19）· `wasi_webgpu_queue_submit.rs`（stub submit `(13, 19)`）· `wasi_webgpu_begin_render_pass.rs`（stub pass=29，view=23）· `wasi_webgpu_render_pass_end.rs`（stub end pass=29）· `wasi_webgpu_method_request_adapter.rs`（`get-gpu` + `[method]` stub 7） |
| 仪器孪生 | `WasiWebGpuRequestAdapterInstrumentedTest` · `WasiWebGpuRequestDeviceInstrumentedTest` · `WasiWebGpuDeviceGetQueueInstrumentedTest` · `WasiWebGpuCreateCommandEncoderInstrumentedTest` · `WasiWebGpuCommandEncoderFinishInstrumentedTest` · `WasiWebGpuQueueSubmitInstrumentedTest` · `WasiWebGpuBeginRenderPassInstrumentedTest` · `WasiWebGpuRenderPassEndInstrumentedTest` · `WasiWebGpuMethodRequestAdapterInstrumentedTest` |
| 资产拷贝 | `smoke-app` `copyW1Fixtures` → `androidTest/assets/w1` |

## 5. 明确不在 W1（历史约束）

- present / native window / **wasi-gfx**（W4）  
- 完整 WIT **resource** 表、`[method]gpu.*` 终态名（W3）  
- WebGPU **CTS** 或合规宣称（NG-5）  
- 静默删除 experimental 扁平面（过渡期双注册）

## 6. 修订

- W1 已交付；W2：`request-adapter` + 扁平 `adapter-request-device` 真 async 已交付。W3：扁平 `device-get-queue` + `device-create-command-encoder` + `command-encoder-finish` + `queue-submit1` + `command-encoder-begin-render-pass-clear` + `render-pass-end` sync 已交付；**`get-gpu` + `[method]gpu.request-adapter`** 真 async 已交付（非 `option<gpu-adapter>`）。  
- 改 instance / 过渡名：更新本页 + gap §5 + `changelog/unreleased/` 碎片。
