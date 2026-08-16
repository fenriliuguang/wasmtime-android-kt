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
| `webgpu_method_begin_render_pass.wasm` | `get-encoder` + **`[method]gpu-command-encoder.begin-render-pass` sync** `u32` | `run: async func() -> u32` | construct encoder → begin-clear(stub view 23); returns pass rep |
| `webgpu_method_render_pass_end.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.end` sync** void | `run: async func() -> u32` | construct pass → end; returns stub 29 |
| `webgpu_method_command_encoder_finish.wasm` | `get-encoder` + **`[method]gpu-command-encoder.finish` sync** `u32` | `run: async func() -> u32` | construct encoder → finish; returns command-buffer rep |
| `webgpu_method_queue_submit.wasm` | `get-queue` + **`[method]gpu-queue.submit` sync** void | `run: async func() -> u32` | construct queue → submit(stub 19); returns 19 |
| `webgpu_method_create_buffer.wasm` | `get-device` + **`[method]gpu-device.create-buffer` sync** `u32` | `run: async func() -> u32` | construct device → create-buffer (host-fixed descriptor); returns buffer rep |
| `webgpu_method_create_texture.wasm` | `get-device` + **`[method]gpu-device.create-texture` sync** `u32` | `run: async func() -> u32` | construct device → create-texture (host-fixed 1×1); returns texture rep |
| `webgpu_method_create_sampler.wasm` | `get-device` + **`[method]gpu-device.create-sampler` sync** `u32` | `run: async func() -> u32` | construct device → create-sampler (host-fixed default); returns sampler rep |
| `webgpu_method_create_shader_module.wasm` | `get-device` + **`[method]gpu-device.create-shader-module` sync** `u32` | `run: async func() -> u32` | construct device → create-shader-module (host-fixed WGSL); returns shader rep |
| `webgpu_method_write_buffer.wasm` | `get-queue` + **`[method]gpu-queue.write-buffer` sync** void | `run: async func() -> u32` | construct queue → write-buffer(stub 31); returns 31 |
| `webgpu_method_texture_create_view.wasm` | `get-texture` + **`[method]gpu-texture.create-view` sync** `u32` | `run: async func() -> u32` | construct texture → create-view (host-fixed 1×1); returns view rep |
| `webgpu_method_create_bind_group_layout.wasm` | `get-device` + **`[method]gpu-device.create-bind-group-layout` sync** `u32` | `run: async func() -> u32` | construct device → create-bind-group-layout (host-fixed empty entries); returns layout rep |
| `webgpu_method_create_pipeline_layout.wasm` | `get-device` + **`[method]gpu-device.create-pipeline-layout` sync** `u32` | `run: async func() -> u32` | construct device → create-pipeline-layout (host-fixed empty bind-group-layouts); returns layout rep |
| `webgpu_method_create_bind_group.wasm` | `get-device` + **`[method]gpu-device.create-bind-group` sync** `u32` | `run: async func() -> u32` | construct device → create-bind-group (host-fixed empty BGL + empty entries); returns bind-group rep |
| `webgpu_method_create_render_pipeline.wasm` | `get-device` + **`[method]gpu-device.create-render-pipeline` sync** `u32` | `run: async func() -> u32` | construct device → create-render-pipeline (host-fixed stub shader + triangle); returns pipeline rep |

**Transitional:** host still registers flat names. **W2:** true CM async (`func_wrap_concurrent` + oneshot yield); pump via `run_concurrent` / `callRunConcurrent`. **W3:** `device-get-queue`, `device-create-command-encoder`, `command-encoder-finish`, `queue-submit1`, `command-encoder-begin-render-pass-clear`, and `render-pass-end` are **sync** on the same proposal instance (same L2 u32 as experimental; submit is single-buffer, not proposal `list`; begin-clear / end use stub view `23`, instrument substitutes Cpu offscreen TextureView). **W3 `[method]`:** `get-gpu` + `[method]gpu.request-adapter` (resource self; still u32, not `option<gpu-adapter>`), `get-adapter` + `[method]gpu-adapter.request-device` (resource self; still u32, not `result<gpu-device, …>`), and `get-device` + `[method]gpu-device.queue` (resource self; still u32, not `gpu-queue`) and `[method]gpu-device.create-command-encoder` (resource self; still u32, no descriptor) and `[method]gpu-device.create-buffer` (resource self; still u32, host-fixed descriptor) and `[method]gpu-device.create-texture` (resource self; still u32, host-fixed 1×1) and `[method]gpu-device.create-sampler` (resource self; still u32, host-fixed default) and `[method]gpu-device.create-shader-module` (resource self; still u32, host-fixed WGSL) and `[method]gpu-queue.write-buffer` (resource self + stub buffer u32; host-fixed 4 bytes) and `get-texture` + `[method]gpu-texture.create-view` (resource self; still u32, host-fixed 1×1) and `[method]gpu-device.create-bind-group-layout` (resource self; still u32, host-fixed empty entries) and `[method]gpu-device.create-pipeline-layout` (resource self; still u32, host-fixed empty bind-group-layouts) and `[method]gpu-device.create-bind-group` (resource self; still u32, host-fixed empty BGL + empty entries) and `[method]gpu-device.create-render-pipeline` (resource self; still u32, host-fixed stub shader + triangle). Experimental flat sync path unchanged. Not full option/resource compliance.

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
wasm-tools parse fixtures/w1/webgpu_method_begin_render_pass.wat -o fixtures/w1/webgpu_method_begin_render_pass.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_begin_render_pass.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_end.wat -o fixtures/w1/webgpu_method_render_pass_end.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_end.wasm
wasm-tools parse fixtures/w1/webgpu_method_command_encoder_finish.wat -o fixtures/w1/webgpu_method_command_encoder_finish.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_command_encoder_finish.wasm
wasm-tools parse fixtures/w1/webgpu_method_queue_submit.wat -o fixtures/w1/webgpu_method_queue_submit.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_queue_submit.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_buffer.wat -o fixtures/w1/webgpu_method_create_buffer.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_buffer.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_texture.wat -o fixtures/w1/webgpu_method_create_texture.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_texture.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_sampler.wat -o fixtures/w1/webgpu_method_create_sampler.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_sampler.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_shader_module.wat -o fixtures/w1/webgpu_method_create_shader_module.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_shader_module.wasm
wasm-tools parse fixtures/w1/webgpu_method_write_buffer.wat -o fixtures/w1/webgpu_method_write_buffer.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_write_buffer.wasm
wasm-tools parse fixtures/w1/webgpu_method_texture_create_view.wat -o fixtures/w1/webgpu_method_texture_create_view.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_texture_create_view.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_bind_group_layout.wat -o fixtures/w1/webgpu_method_create_bind_group_layout.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_bind_group_layout.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_pipeline_layout.wat -o fixtures/w1/webgpu_method_create_pipeline_layout.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_pipeline_layout.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_bind_group.wat -o fixtures/w1/webgpu_method_create_bind_group.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_bind_group.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_render_pipeline.wat -o fixtures/w1/webgpu_method_create_render_pipeline.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_render_pipeline.wasm
```
