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
| `webgpu_method_request_adapter.wasm` | `get-gpu` + **`[method]gpu.request-adapter` async** `option<own<gpu-adapter>>` | `run: async func() -> u32` | construct gpu → method request-adapter(none) → drop own; harness returns 1 |
| `webgpu_method_request_device.wasm` | `get-adapter` + **`[method]gpu-adapter.request-device` async** `result<own<gpu-device>, request-device-error>` | `run: async func() -> u32` | construct adapter → method request-device(none) → drop own on ok; harness returns 1 |
| `webgpu_method_device_queue.wasm` | `get-device` + **`[method]gpu-device.queue` sync** `own<gpu-queue>` | `run: async func() -> u32` | construct device → method queue → drop own; harness returns 1 |
| `webgpu_method_create_command_encoder.wasm` | `get-device` + **`[method]gpu-device.create-command-encoder` sync** `own<gpu-command-encoder>` | `run: async func() -> u32` | construct device → create-encoder (descriptor=none) → drop own; harness returns 1 |
| `webgpu_method_begin_render_pass.wasm` | `get-encoder` + **`[method]gpu-command-encoder.begin-render-pass` sync** `own<gpu-render-pass-encoder>` | `run: async func() -> u32` | construct encoder → begin (empty color-attachments) → drop own; harness returns 1 |
| `webgpu_method_render_pass_end.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.end` sync** void | `run: async func() -> u32` | construct pass → end; returns stub 29 |
| `webgpu_method_command_encoder_finish.wasm` | `get-encoder` + **`[method]gpu-command-encoder.finish` sync** `own<gpu-command-buffer>` | `run: async func() -> u32` | construct encoder → finish (descriptor=none) → drop own; harness returns 1 |
| `webgpu_method_queue_submit.wasm` | `get-queue` + `get-command-buffer` + **`[method]gpu-queue.submit` sync** void (`list<borrow<gpu-command-buffer>>`) | `run: async func() -> u32` | construct queue + command-buffer → submit(one-element list) → drop owns; harness returns 1 |
| `webgpu_method_create_buffer.wasm` | `get-device` + **`[method]gpu-device.create-buffer` sync** `own<gpu-buffer>` | `run: async func() -> u32` | construct device → create-buffer (`gpu-buffer-descriptor` size=4 COPY_DST\|VERTEX) → drop own; harness returns 1 |
| `webgpu_method_create_texture.wasm` | `get-device` + **`[method]gpu-device.create-texture` sync** `own<gpu-texture>` | `run: async func() -> u32` | construct device → create-texture (`gpu-texture-descriptor` 1×1×1 rgba8unorm RENDER_ATTACHMENT) → drop own; harness returns 1 |
| `webgpu_method_create_sampler.wasm` | `get-device` + **`[method]gpu-device.create-sampler` sync** `own<gpu-sampler>` | `run: async func() -> u32` | construct device → create-sampler (descriptor=none) → drop own; harness returns 1 |
| `webgpu_method_create_shader_module.wasm` | `get-device` + **`[method]gpu-device.create-shader-module` sync** `own<gpu-shader-module>` | `run: async func() -> u32` | construct device → create-shader-module (`gpu-shader-module-descriptor` empty code; L2 host-fixed WGSL) → drop own; harness returns 1 |
| `webgpu_method_write_buffer.wasm` | `get-queue` + **`[method]gpu-queue.write-buffer` sync** void | `run: async func() -> u32` | construct queue → write-buffer(stub 31); returns 31 |
| `webgpu_method_texture_create_view.wasm` | `get-texture` + **`[method]gpu-texture.create-view` sync** `own<gpu-texture-view>` | `run: async func() -> u32` | construct texture → create-view (descriptor=none) → drop own; harness returns 1 |
| `webgpu_method_create_bind_group_layout.wasm` | `get-device` + **`[method]gpu-device.create-bind-group-layout` sync** `own<gpu-bind-group-layout>` | `run: async func() -> u32` | construct device → create-bind-group-layout (empty entries; L2 host-fixed empty) → drop own; harness returns 1 |
| `webgpu_method_create_pipeline_layout.wasm` | `get-device` + **`[method]gpu-device.create-pipeline-layout` sync** `own<gpu-pipeline-layout>` | `run: async func() -> u32` | construct device → create-pipeline-layout (empty bind-group-layouts; L2 host-fixed empty) → drop own; harness returns 1 |
| `webgpu_method_create_bind_group.wasm` | `get-device` + `get-bind-group-layout` + **`[method]gpu-device.create-bind-group` sync** `own<gpu-bind-group>` | `run: async func() -> u32` | construct device + layout → create-bind-group (empty entries; L2 host-fixed empty BGL + empty entries) → drop own; harness returns 1 |
| `webgpu_method_create_render_pipeline.wasm` | `get-device` + **`[method]gpu-device.create-render-pipeline` sync** `u32` | `run: async func() -> u32` | construct device → create-render-pipeline (host-fixed stub shader + triangle); returns pipeline rep |
| `webgpu_method_create_compute_pipeline.wasm` | `get-device` + **`[method]gpu-device.create-compute-pipeline` sync** `u32` | `run: async func() -> u32` | construct device → create-compute-pipeline (host-fixed stub shader + empty layout); returns pipeline rep |
| `webgpu_method_write_texture.wasm` | `get-queue` + **`[method]gpu-queue.write-texture` sync** void | `run: async func() -> u32` | construct queue → write-texture(stub 37); returns 37 |
| `webgpu_method_begin_compute_pass.wasm` | `get-encoder` + **`[method]gpu-command-encoder.begin-compute-pass` sync** `own<gpu-compute-pass-encoder>` | `run: async func() -> u32` | construct encoder → begin-compute-pass (descriptor=none) → drop own; harness returns 1 |
| `webgpu_method_compute_pass_end.wasm` | `get-compute-pass` + **`[method]gpu-compute-pass-encoder.end` sync** void | `run: async func() -> u32` | construct compute-pass → end; returns 79 |
| `webgpu_method_copy_buffer_to_buffer.wasm` | `get-encoder` + `get-buffer` + **`[method]gpu-command-encoder.copy-buffer-to-buffer` sync** void | `run: async func() -> u32` | construct encoder + two buffers → copy (offsets/size none; L2 host-fixed 4-byte) → drop owns; harness returns 1 |
| `webgpu_method_compute_pass_set_pipeline.wasm` | `get-compute-pass` + `get-compute-pipeline` + **`[method]gpu-compute-pass-encoder.set-pipeline` sync** void | `run: async func() -> u32` | construct compute-pass + pipeline → set-pipeline (borrow; L2 host-fixed compute pipeline) → drop own; harness returns 1 |
| `webgpu_method_compute_pass_set_bind_group.wasm` | `get-compute-pass` + `get-bind-group` + **`[method]gpu-compute-pass-encoder.set-bind-group` sync** `result<_, set-bind-group-error>` | `run: async func() -> u32` | construct compute-pass + bind-group → set-bind-group (index 0, some, offsets none) → ok → drop own; harness returns 1 |
| `webgpu_method_compute_pass_dispatch_workgroups.wasm` | `get-compute-pass` + **`[method]gpu-compute-pass-encoder.dispatch-workgroups` sync** void | `run: async func() -> u32` | construct compute-pass → dispatch(x=1, y/z=some(1); L2 host-fixed 1×1×1); harness returns 1 |
| `webgpu_method_render_pass_set_pipeline.wasm` | `get-pass` + `get-render-pipeline` + **`[method]gpu-render-pass-encoder.set-pipeline` sync** void | `run: async func() -> u32` | construct pass + pipeline → set-pipeline (borrow; L2 host-fixed triangle pipeline) → drop own; harness returns 1 |
| `webgpu_method_render_pass_draw.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.draw` sync** void | `run: async func() -> u32` | construct pass → draw(vertex-count=3, other options none; L2 host-fixed draw(3)); harness returns 1 |
| `webgpu_method_render_pass_set_bind_group.wasm` | `get-pass` + `get-bind-group` + **`[method]gpu-render-pass-encoder.set-bind-group` sync** `result<_, set-bind-group-error>` | `run: async func() -> u32` | construct pass + bind-group → set-bind-group (index 0, some, offsets none) → ok → drop own; harness returns 1 |
| `webgpu_method_render_pass_set_vertex_buffer.wasm` | `get-pass` + `get-buffer` + **`[method]gpu-render-pass-encoder.set-vertex-buffer` sync** void | `run: async func() -> u32` | construct pass + buffer → set-vertex-buffer (slot 0, some, offset/size none; L2 host-fixed VERTEX slot 0) → drop own; harness returns 1 |
| `webgpu_method_buffer_map_async.wasm` | `get-buffer` + **`[method]gpu-buffer.map-async` async** `result<_, map-async-error>` | `run: async func() -> u32` | construct buffer → map-async(READ, offset/size none) → ok; harness returns 1 |
| `webgpu_method_buffer_unmap.wasm` | `get-buffer` + **`[method]gpu-buffer.unmap` sync** void | `run: async func() -> u32` | construct buffer → unmap; returns 31 |

**Transitional:** host still registers flat names. **W2:** true CM async (`func_wrap_concurrent` + oneshot yield); pump via `run_concurrent` / `callRunConcurrent`. **W3:** `device-get-queue`, `device-create-command-encoder`, `command-encoder-finish`, `queue-submit1`, `command-encoder-begin-render-pass-clear`, and `render-pass-end` are **sync** on the same proposal instance (same L2 u32 as experimental; submit is single-buffer, not proposal `list`; begin-clear / end use stub view `23`, instrument substitutes Cpu offscreen TextureView). **W3 `[method]`:** `get-gpu` + `[method]gpu.request-adapter` (S2: `option<own<gpu-adapter>>`), `get-adapter` + `[method]gpu-adapter.request-device` (S3: `result<own<gpu-device>, request-device-error>`), and `get-device` + `[method]gpu-device.queue` (S1: `own<gpu-queue>`) and `[method]gpu-device.create-command-encoder` (S6: Guest descriptor=none → `own<gpu-command-encoder>`) and `[method]gpu-command-encoder.finish` (S7: Guest descriptor=none → `own<gpu-command-buffer>`) and `[method]gpu-device.create-buffer` (S4: Guest `gpu-buffer-descriptor` → `own<gpu-buffer>`) and `[method]gpu-queue.submit` (S5: Guest `list<borrow<gpu-command-buffer>>` → drop owns; harness 1) and `[method]gpu-device.create-texture` (S6+: Guest `gpu-texture-descriptor` → `own<gpu-texture>`) and `[method]gpu-device.create-sampler` (S8: Guest descriptor=none → `own<gpu-sampler>`) and `[method]gpu-device.create-shader-module` (S6+: Guest `gpu-shader-module-descriptor` → `own<gpu-shader-module>`; L2 host-fixed WGSL) and `[method]gpu-queue.write-buffer` (resource self + stub buffer u32; host-fixed 4 bytes) and `get-texture` + `[method]gpu-texture.create-view` (S8: Guest descriptor=none → `own<gpu-texture-view>`) and `[method]gpu-device.create-bind-group-layout` (S6+: Guest `gpu-bind-group-layout-descriptor` → `own<gpu-bind-group-layout>`; L2 host-fixed empty entries) and `[method]gpu-device.create-pipeline-layout` (S6+: Guest `gpu-pipeline-layout-descriptor` → `own<gpu-pipeline-layout>`; L2 host-fixed empty bind-group-layouts) and `[method]gpu-device.create-bind-group` (S6+: Guest `gpu-bind-group-descriptor` → `own<gpu-bind-group>`; L2 host-fixed empty BGL + empty entries) and `[method]gpu-device.create-render-pipeline` (resource self; still u32, host-fixed stub shader + triangle) and `[method]gpu-device.create-compute-pipeline` (resource self; still u32, host-fixed stub shader + empty layout) and `[method]gpu-queue.write-texture` (resource self + stub texture u32; host-fixed 1×1) and `[method]gpu-command-encoder.begin-compute-pass` (S8: Guest descriptor=none → `own<gpu-compute-pass-encoder>`) and `get-compute-pass` + `[method]gpu-compute-pass-encoder.end` (resource self; void) and `[method]gpu-command-encoder.copy-buffer-to-buffer` (S6+: Guest borrow src/dst + option offsets/size none → drop owns; harness 1; L2 host-fixed 4-byte copy) and `[method]gpu-compute-pass-encoder.set-pipeline` (S6+: Guest `borrow<gpu-compute-pipeline>`; L2 host-fixed compute pipeline) and `[method]gpu-compute-pass-encoder.set-bind-group` (S6+: index + option bind-group + option offsets → `result<_, set-bind-group-error>`; L2 host-fixed empty bind-group) and `[method]gpu-compute-pass-encoder.dispatch-workgroups` (S6+: x + option y/z; L2 host-fixed 1×1×1 after set-pipeline + empty bind-group) and `[method]gpu-render-pass-encoder.set-pipeline` (S6+: Guest `borrow<gpu-render-pipeline>`; L2 host-fixed triangle pipeline) and `[method]gpu-render-pass-encoder.set-bind-group` (S6+: same result shape as compute; L2 host-fixed empty bind-group) and `[method]gpu-render-pass-encoder.set-vertex-buffer` (S6+: slot + option buffer + option offset/size; L2 host-fixed VERTEX slot 0) and `[method]gpu-render-pass-encoder.draw` (S6+: vertex-count + three option<u32>; L2 host-fixed draw(3) after set-pipeline). Experimental flat sync path unchanged. Not full option/resource compliance.

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
wasm-tools parse fixtures/w1/webgpu_method_create_compute_pipeline.wat -o fixtures/w1/webgpu_method_create_compute_pipeline.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_compute_pipeline.wasm
wasm-tools parse fixtures/w1/webgpu_method_write_texture.wat -o fixtures/w1/webgpu_method_write_texture.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_write_texture.wasm
wasm-tools parse fixtures/w1/webgpu_method_begin_compute_pass.wat -o fixtures/w1/webgpu_method_begin_compute_pass.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_begin_compute_pass.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pass_end.wat -o fixtures/w1/webgpu_method_compute_pass_end.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pass_end.wasm
wasm-tools parse fixtures/w1/webgpu_method_copy_buffer_to_buffer.wat -o fixtures/w1/webgpu_method_copy_buffer_to_buffer.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_copy_buffer_to_buffer.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pass_set_pipeline.wat -o fixtures/w1/webgpu_method_compute_pass_set_pipeline.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pass_set_pipeline.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pass_set_bind_group.wat -o fixtures/w1/webgpu_method_compute_pass_set_bind_group.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pass_set_bind_group.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pass_dispatch_workgroups.wat -o fixtures/w1/webgpu_method_compute_pass_dispatch_workgroups.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pass_dispatch_workgroups.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_set_pipeline.wat -o fixtures/w1/webgpu_method_render_pass_set_pipeline.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_set_pipeline.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_draw.wat -o fixtures/w1/webgpu_method_render_pass_draw.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_draw.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_set_bind_group.wat -o fixtures/w1/webgpu_method_render_pass_set_bind_group.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_set_bind_group.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_set_vertex_buffer.wat -o fixtures/w1/webgpu_method_render_pass_set_vertex_buffer.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_set_vertex_buffer.wasm
wasm-tools parse fixtures/w1/webgpu_method_buffer_map_async.wat -o fixtures/w1/webgpu_method_buffer_map_async.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_buffer_map_async.wasm
wasm-tools parse fixtures/w1/webgpu_method_buffer_unmap.wat -o fixtures/w1/webgpu_method_buffer_unmap.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_buffer_unmap.wasm
```
