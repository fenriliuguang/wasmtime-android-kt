# W1/W2 wasi:webgpu fixtures

| File | Imports | Export | Behavior |
|------|---------|--------|----------|
| `webgpu_request_adapter.wasm` | `wasi:webgpu/webgpu@0.3.0-rc.2#request-adapter` → **async** `u32` | `run: async func() -> u32` | returns adapter rep from same L2 as M3 |
| `webgpu_request_device.wasm` | `request-adapter` + `adapter-request-device` → **async** `u32` | `run: async func() -> u32` | adapter then device; returns device rep |
| `webgpu_device_get_queue.wasm` | `request-adapter` + `adapter-request-device` (async) + **`device-get-queue` sync** `u32` | `run: async func() -> u32` | adapter → device → queue; returns queue rep |
| `webgpu_create_command_encoder.wasm` | `request-adapter` + `adapter-request-device` (async) + **`device-create-command-encoder` sync** `u32` | `run: async func() -> u32` | adapter → device → encoder; returns encoder rep |
| `webgpu_command_encoder_finish.wasm` | `request-adapter` + `adapter-request-device` (async) + `device-create-command-encoder` + **`command-encoder-finish` sync** `u32` | `run: async func() -> u32` | adapter → device → encoder → finish; returns command-buffer rep |
| `webgpu_queue_submit.wasm` | `request-adapter` + `adapter-request-device` (async) + `device-get-queue` + `device-create-command-encoder` + `command-encoder-finish` + **`queue-submit1` sync** void | `run: async func() -> u32` | adapter → device → queue + encoder → finish → submit; returns command-buffer rep |
| `webgpu_begin_render_pass.wasm` | `request-adapter` + `adapter-request-device` (async) + `device-create-command-encoder` + **`command-encoder-begin-render-pass-clear` sync** `u32` | `run: async func() -> u32` | adapter → device → encoder → begin-clear(stub view 23); returns render-pass rep |
| `webgpu_render_pass_end.wasm` | `request-adapter` + `adapter-request-device` (async) + `device-create-command-encoder` + `command-encoder-begin-render-pass-clear` + **`render-pass-end` sync** void | `run: async func() -> u32` | adapter → device → encoder → begin-clear(stub view 23) → end; returns render-pass rep |
| `webgpu_method_request_adapter.wasm` | `get-gpu` + **`[method]gpu.request-adapter` async** `u32` | `run: async func() -> u32` | construct gpu → method request-adapter; returns adapter rep |
| `webgpu_method_request_device.wasm` | `get-adapter` + **`[method]gpu-adapter.request-device` async** `u32` | `run: async func() -> u32` | construct adapter → method request-device; returns device rep |
| `webgpu_method_device_queue.wasm` | `get-device` + **`[method]gpu-device.queue` sync** `u32` | `run: async func() -> u32` | construct device → method queue; returns queue rep |
| `webgpu_method_create_command_encoder.wasm` | `get-device` + **`[method]gpu-device.create-command-encoder` sync** `u32` | `run: async func() -> u32` | construct device → method create-encoder; returns encoder rep |

**Transitional:** host still registers flat names. **W2:** true CM async (`func_wrap_concurrent` + oneshot yield); pump via `run_concurrent` / `callRunConcurrent`. **W3:** `device-get-queue`, `device-create-command-encoder`, `command-encoder-finish`, `queue-submit1`, `command-encoder-begin-render-pass-clear`, and `render-pass-end` are **sync** on the same proposal instance (same L2 u32 as experimental; submit is single-buffer, not proposal `list`; begin-clear / end use stub view `23`, instrument substitutes Cpu offscreen TextureView). **W3 `[method]`:** `get-gpu` + `[method]gpu.request-adapter` (resource self; still u32, not `option<gpu-adapter>`), `get-adapter` + `[method]gpu-adapter.request-device` (resource self; still u32, not `result<gpu-device, …>`), and `get-device` + `[method]gpu-device.queue` (resource self; still u32, not `gpu-queue`) and `[method]gpu-device.create-command-encoder` (resource self; still u32, no descriptor). Experimental flat sync path unchanged. Not full option/resource compliance.

Regenerate:

```powershell
wasm-tools parse fixtures/w1/webgpu_request_adapter.wat -o fixtures/w1/webgpu_request_adapter.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_request_adapter.wasm
wasm-tools parse fixtures/w1/webgpu_request_device.wat -o fixtures/w1/webgpu_request_device.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_request_device.wasm
wasm-tools parse fixtures/w1/webgpu_device_get_queue.wat -o fixtures/w1/webgpu_device_get_queue.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_device_get_queue.wasm
wasm-tools parse fixtures/w1/webgpu_create_command_encoder.wat -o fixtures/w1/webgpu_create_command_encoder.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_create_command_encoder.wasm
wasm-tools parse fixtures/w1/webgpu_command_encoder_finish.wat -o fixtures/w1/webgpu_command_encoder_finish.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_command_encoder_finish.wasm
wasm-tools parse fixtures/w1/webgpu_queue_submit.wat -o fixtures/w1/webgpu_queue_submit.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_queue_submit.wasm
wasm-tools parse fixtures/w1/webgpu_begin_render_pass.wat -o fixtures/w1/webgpu_begin_render_pass.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_begin_render_pass.wasm
wasm-tools parse fixtures/w1/webgpu_render_pass_end.wat -o fixtures/w1/webgpu_render_pass_end.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_render_pass_end.wasm
wasm-tools parse fixtures/w1/webgpu_method_request_adapter.wat -o fixtures/w1/webgpu_method_request_adapter.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_request_adapter.wasm
wasm-tools parse fixtures/w1/webgpu_method_request_device.wat -o fixtures/w1/webgpu_method_request_device.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_request_device.wasm
wasm-tools parse fixtures/w1/webgpu_method_device_queue.wat -o fixtures/w1/webgpu_method_device_queue.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_device_queue.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_command_encoder.wat -o fixtures/w1/webgpu_method_create_command_encoder.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_command_encoder.wasm
```
