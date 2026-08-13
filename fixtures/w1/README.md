# W1/W2 wasi:webgpu fixtures

| File | Imports | Export | Behavior |
|------|---------|--------|----------|
| `webgpu_request_adapter.wasm` | `wasi:webgpu/webgpu@0.3.0-rc.2#request-adapter` → **async** `u32` | `run: async func() -> u32` | returns adapter rep from same L2 as M3 |
| `webgpu_request_device.wasm` | `request-adapter` + `adapter-request-device` → **async** `u32` | `run: async func() -> u32` | adapter then device; returns device rep |
| `webgpu_device_get_queue.wasm` | `request-adapter` + `adapter-request-device` (async) + **`device-get-queue` sync** `u32` | `run: async func() -> u32` | adapter → device → queue; returns queue rep |

**Transitional:** host registers flat names (not `[method]gpu.*`). **W2:** true CM async (`func_wrap_concurrent` + oneshot yield); pump via `run_concurrent` / `callRunConcurrent`. **W3 first slice:** `device-get-queue` is a **sync** getter on the same proposal instance (same L2 u32 as experimental). Experimental flat sync path unchanged. Not full option/resource compliance.

Regenerate:

```powershell
wasm-tools parse fixtures/w1/webgpu_request_adapter.wat -o fixtures/w1/webgpu_request_adapter.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_request_adapter.wasm
wasm-tools parse fixtures/w1/webgpu_request_device.wat -o fixtures/w1/webgpu_request_device.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_request_device.wasm
wasm-tools parse fixtures/w1/webgpu_device_get_queue.wat -o fixtures/w1/webgpu_device_get_queue.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_device_get_queue.wasm
```
