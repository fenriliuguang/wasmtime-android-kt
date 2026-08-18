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
| `webgpu_method_render_pass_end.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.end` sync** void | `run: async func() -> u32` | construct pass → end; harness returns 1 |
| `webgpu_method_command_encoder_finish.wasm` | `get-encoder` + **`[method]gpu-command-encoder.finish` sync** `own<gpu-command-buffer>` | `run: async func() -> u32` | construct encoder → finish (descriptor=none) → drop own; harness returns 1 |
| `webgpu_method_queue_submit.wasm` | `get-queue` + `get-command-buffer` + **`[method]gpu-queue.submit` sync** void (`list<borrow<gpu-command-buffer>>`) | `run: async func() -> u32` | construct queue + command-buffer → submit(one-element list) → drop owns; harness returns 1 |
| `webgpu_method_create_buffer.wasm` | `get-device` + **`[method]gpu-device.create-buffer` sync** `own<gpu-buffer>` | `run: async func() -> u32` | construct device → create-buffer (`gpu-buffer-descriptor` size=4 COPY_DST\|VERTEX) → drop own; harness returns 1 |
| `webgpu_method_create_texture.wasm` | `get-device` + **`[method]gpu-device.create-texture` sync** `own<gpu-texture>` | `run: async func() -> u32` | construct device → create-texture (`gpu-texture-descriptor` 1×1×1 rgba8unorm RENDER_ATTACHMENT) → drop own; harness returns 1 |
| `webgpu_method_create_sampler.wasm` | `get-device` + **`[method]gpu-device.create-sampler` sync** `own<gpu-sampler>` | `run: async func() -> u32` | construct device → create-sampler (descriptor=none) → drop own; harness returns 1 |
| `webgpu_method_create_shader_module.wasm` | `get-device` + **`[method]gpu-device.create-shader-module` sync** `own<gpu-shader-module>` | `run: async func() -> u32` | construct device → create-shader-module (`gpu-shader-module-descriptor` empty code; L2 host-fixed WGSL) → drop own; harness returns 1 |
| `webgpu_method_write_buffer.wasm` | `get-queue` + `get-buffer` + **`[method]gpu-queue.write-buffer-with-copy` sync** `result<_, write-buffer-error>` | `run: async func() -> u32` | construct queue + buffer → write (offset 0, empty data, offset/size none; L2 host-fixed 4 bytes) → ok → drop buffer; harness returns 1 |
| `webgpu_method_texture_create_view.wasm` | `get-texture` + **`[method]gpu-texture.create-view` sync** `own<gpu-texture-view>` | `run: async func() -> u32` | construct texture → create-view (descriptor=none) → drop own; harness returns 1 |
| `webgpu_method_create_bind_group_layout.wasm` | `get-device` + **`[method]gpu-device.create-bind-group-layout` sync** `own<gpu-bind-group-layout>` | `run: async func() -> u32` | construct device → create-bind-group-layout (empty entries; L2 host-fixed empty) → drop own; harness returns 1 |
| `webgpu_method_create_pipeline_layout.wasm` | `get-device` + **`[method]gpu-device.create-pipeline-layout` sync** `own<gpu-pipeline-layout>` | `run: async func() -> u32` | construct device → create-pipeline-layout (empty bind-group-layouts; L2 host-fixed empty) → drop own; harness returns 1 |
| `webgpu_method_create_bind_group.wasm` | `get-device` + `get-bind-group-layout` + **`[method]gpu-device.create-bind-group` sync** `own<gpu-bind-group>` | `run: async func() -> u32` | construct device + layout → create-bind-group (empty entries; L2 host-fixed empty BGL + empty entries) → drop own; harness returns 1 |
| `webgpu_method_create_render_pipeline.wasm` | `get-device` + `get-shader-module` + **`[method]gpu-device.create-render-pipeline` sync** `own<gpu-render-pipeline>` | `run: async func() -> u32` | construct device + shader → create-render-pipeline (shader borrow, layout auto, other options none; L2 host-fixed stub shader + triangle) → drop own; harness returns 1 |
| `webgpu_method_create_render_pipeline_async.wasm` | `get-device` + `get-shader-module` + **`[method]gpu-device.create-render-pipeline-async` async** `result<own<gpu-render-pipeline>, create-pipeline-error>` | `run: async func() -> u32` | same descriptor as sync create; drop own on ok; harness returns 1; L2 host-fixed stub shader + triangle |
| `webgpu_method_create_compute_pipeline.wasm` | `get-device` + `get-shader-module` + **`[method]gpu-device.create-compute-pipeline` sync** `own<gpu-compute-pipeline>` | `run: async func() -> u32` | construct device + shader → create-compute-pipeline (shader borrow, layout auto; L2 host-fixed stub shader + empty layout) → drop own; harness returns 1 |
| `webgpu_method_create_compute_pipeline_async.wasm` | `get-device` + `get-shader-module` + **`[method]gpu-device.create-compute-pipeline-async` async** `result<own<gpu-compute-pipeline>, create-pipeline-error>` | `run: async func() -> u32` | same descriptor as sync create; drop own on ok; harness returns 1; L2 host-fixed stub shader + empty layout |
| `webgpu_method_write_texture.wasm` | `get-queue` + `get-texture` + **`[method]gpu-queue.write-texture-with-copy` sync** void | `run: async func() -> u32` | construct queue + texture → write (empty data, layout none, size 1×1×1; L2 host-fixed 1×1) → drop texture; harness returns 1 |
| `webgpu_method_begin_compute_pass.wasm` | `get-encoder` + **`[method]gpu-command-encoder.begin-compute-pass` sync** `own<gpu-compute-pass-encoder>` | `run: async func() -> u32` | construct encoder → begin-compute-pass (descriptor=none) → drop own; harness returns 1 |
| `webgpu_method_compute_pass_end.wasm` | `get-compute-pass` + **`[method]gpu-compute-pass-encoder.end` sync** void | `run: async func() -> u32` | construct compute-pass → end; harness returns 1 |
| `webgpu_method_copy_buffer_to_buffer.wasm` | `get-encoder` + `get-buffer` + **`[method]gpu-command-encoder.copy-buffer-to-buffer` sync** void | `run: async func() -> u32` | construct encoder + two buffers → copy (offsets/size none; L2 host-fixed 4-byte) → drop owns; harness returns 1 |
| `webgpu_method_copy_buffer_to_texture.wasm` | `get-encoder` + `get-buffer` + `get-texture` + **`[method]gpu-command-encoder.copy-buffer-to-texture` sync** void | `run: async func() -> u32` | construct encoder + buffer + texture → copy (layout/mip/origin/aspect none, size 1×1×1; L2 host-fixed 4-byte buffer copy) → drop owns; harness returns 1 |
| `webgpu_method_copy_texture_to_buffer.wasm` | `get-encoder` + `get-texture` + `get-buffer` + **`[method]gpu-command-encoder.copy-texture-to-buffer` sync** void | `run: async func() -> u32` | construct encoder + texture + buffer → copy (layout/mip/origin/aspect none, size 1×1×1; L2 host-fixed 4-byte buffer copy) → drop owns; harness returns 1 |
| `webgpu_method_copy_texture_to_texture.wasm` | `get-encoder` + `get-texture` + **`[method]gpu-command-encoder.copy-texture-to-texture` sync** void | `run: async func() -> u32` | construct encoder + two textures → copy (mip/origin/aspect none, size 1×1×1; L2 host-fixed 4-byte buffer copy) → drop owns; harness returns 1 |
| `webgpu_method_clear_buffer.wasm` | `get-encoder` + `get-buffer` + **`[method]gpu-command-encoder.clear-buffer` sync** void | `run: async func() -> u32` | construct encoder + buffer → clear (offset/size none; L2 host-fixed 4-byte buffer copy) → drop own; harness returns 1 |
| `webgpu_method_resolve_query_set.wasm` | `get-encoder` + `get-query-set` + `get-buffer` + **`[method]gpu-command-encoder.resolve-query-set` sync** void | `run: async func() -> u32` | construct encoder + query-set + buffer → resolve (first 0, count 1, offset 0); harness returns 1 |
| `webgpu_method_push_debug_group.wasm` | `get-encoder` + **`[method]gpu-command-encoder.push-debug-group` sync** void | `run: async func() -> u32` | construct encoder → push-debug-group(""); harness returns 1 |
| `webgpu_method_pop_debug_group.wasm` | `get-encoder` + **`[method]gpu-command-encoder.pop-debug-group` sync** void | `run: async func() -> u32` | construct encoder → pop-debug-group; harness returns 1 |
| `webgpu_method_insert_debug_marker.wasm` | `get-encoder` + **`[method]gpu-command-encoder.insert-debug-marker` sync** void | `run: async func() -> u32` | construct encoder → insert-debug-marker(""); harness returns 1 |
| `webgpu_method_compute_pass_set_pipeline.wasm` | `get-compute-pass` + `get-compute-pipeline` + **`[method]gpu-compute-pass-encoder.set-pipeline` sync** void | `run: async func() -> u32` | construct compute-pass + pipeline → set-pipeline (borrow; L2 host-fixed compute pipeline) → drop own; harness returns 1 |
| `webgpu_method_compute_pass_set_bind_group.wasm` | `get-compute-pass` + `get-bind-group` + **`[method]gpu-compute-pass-encoder.set-bind-group` sync** `result<_, set-bind-group-error>` | `run: async func() -> u32` | construct compute-pass + bind-group → set-bind-group (index 0, some, offsets none) → ok → drop own; harness returns 1 |
| `webgpu_method_compute_pass_dispatch_workgroups.wasm` | `get-compute-pass` + **`[method]gpu-compute-pass-encoder.dispatch-workgroups` sync** void | `run: async func() -> u32` | construct compute-pass → dispatch(x=1, y/z=some(1); L2 host-fixed 1×1×1); harness returns 1 |
| `webgpu_method_compute_pass_dispatch_workgroups_indirect.wasm` | `get-compute-pass` + `get-buffer` + **`[method]gpu-compute-pass-encoder.dispatch-workgroups-indirect` sync** void | `run: async func() -> u32` | construct compute-pass + buffer → dispatch-indirect(offset 0; L2 host-fixed 1×1×1) → drop own; harness returns 1 |
| `webgpu_method_compute_pass_set_immediates.wasm` | `get-compute-pass` + **`[method]gpu-compute-pass-encoder.set-immediates` sync** void | `run: async func() -> u32` | construct compute-pass → set-immediates(range 0, empty data, offset/size none); harness returns 1 |
| `webgpu_method_compute_pass_push_debug_group.wasm` | `get-compute-pass` + **`[method]gpu-compute-pass-encoder.push-debug-group` sync** void | `run: async func() -> u32` | construct compute-pass → push-debug-group(""); harness returns 1 |
| `webgpu_method_compute_pass_pop_debug_group.wasm` | `get-compute-pass` + **`[method]gpu-compute-pass-encoder.pop-debug-group` sync** void | `run: async func() -> u32` | construct compute-pass → pop-debug-group; harness returns 1 |
| `webgpu_method_compute_pass_insert_debug_marker.wasm` | `get-compute-pass` + **`[method]gpu-compute-pass-encoder.insert-debug-marker` sync** void | `run: async func() -> u32` | construct compute-pass → insert-debug-marker(""); harness returns 1 |
| `webgpu_method_render_pass_set_pipeline.wasm` | `get-pass` + `get-render-pipeline` + **`[method]gpu-render-pass-encoder.set-pipeline` sync** void | `run: async func() -> u32` | construct pass + pipeline → set-pipeline (borrow; L2 host-fixed triangle pipeline) → drop own; harness returns 1 |
| `webgpu_method_render_pass_draw.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.draw` sync** void | `run: async func() -> u32` | construct pass → draw(vertex-count=3, other options none; L2 host-fixed draw(3)); harness returns 1 |
| `webgpu_method_render_pass_set_bind_group.wasm` | `get-pass` + `get-bind-group` + **`[method]gpu-render-pass-encoder.set-bind-group` sync** `result<_, set-bind-group-error>` | `run: async func() -> u32` | construct pass + bind-group → set-bind-group (index 0, some, offsets none) → ok → drop own; harness returns 1 |
| `webgpu_method_render_pass_set_vertex_buffer.wasm` | `get-pass` + `get-buffer` + **`[method]gpu-render-pass-encoder.set-vertex-buffer` sync** void | `run: async func() -> u32` | construct pass + buffer → set-vertex-buffer (slot 0, some, offset/size none; L2 host-fixed VERTEX slot 0) → drop own; harness returns 1 |
| `webgpu_method_render_pass_set_viewport.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.set-viewport` sync** void | `run: async func() -> u32` | construct pass → set-viewport(0,0,1,1,0,1); harness returns 1 |
| `webgpu_method_render_pass_set_scissor_rect.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.set-scissor-rect` sync** void | `run: async func() -> u32` | construct pass → set-scissor-rect(0,0,1,1); harness returns 1 |
| `webgpu_method_render_pass_set_blend_constant.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.set-blend-constant` sync** void | `run: async func() -> u32` | construct pass → set-blend-constant(0,0,0,1); harness returns 1 |
| `webgpu_method_render_pass_set_stencil_reference.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.set-stencil-reference` sync** void | `run: async func() -> u32` | construct pass → set-stencil-reference(0); harness returns 1 |
| `webgpu_method_render_pass_set_index_buffer.wasm` | `get-pass` + `get-buffer` + **`[method]gpu-render-pass-encoder.set-index-buffer` sync** void | `run: async func() -> u32` | construct pass + buffer → set-index-buffer (uint16, offset/size none; L2 host-fixed VERTEX slot 0) → drop own; harness returns 1 |
| `webgpu_method_render_pass_draw_indexed.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.draw-indexed` sync** void | `run: async func() -> u32` | construct pass → draw-indexed(index-count=3, other options none; L2 host-fixed draw(3)); harness returns 1 |
| `webgpu_method_render_pass_draw_indirect.wasm` | `get-pass` + `get-buffer` + **`[method]gpu-render-pass-encoder.draw-indirect` sync** void | `run: async func() -> u32` | construct pass + buffer → draw-indirect(offset 0; L2 host-fixed draw(3)) → drop own; harness returns 1 |
| `webgpu_method_render_pass_draw_indexed_indirect.wasm` | `get-pass` + `get-buffer` + **`[method]gpu-render-pass-encoder.draw-indexed-indirect` sync** void | `run: async func() -> u32` | construct pass + buffer → draw-indexed-indirect(offset 0; L2 host-fixed draw(3)) → drop own; harness returns 1 |
| `webgpu_method_render_pass_push_debug_group.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.push-debug-group` sync** void | `run: async func() -> u32` | construct pass → push-debug-group(""); harness returns 1 |
| `webgpu_method_render_pass_pop_debug_group.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.pop-debug-group` sync** void | `run: async func() -> u32` | construct pass → pop-debug-group; harness returns 1 |
| `webgpu_method_render_pass_insert_debug_marker.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.insert-debug-marker` sync** void | `run: async func() -> u32` | construct pass → insert-debug-marker(""); harness returns 1 |
| `webgpu_method_render_pass_begin_occlusion_query.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.begin-occlusion-query` sync** void | `run: async func() -> u32` | construct pass → begin-occlusion-query(0); harness returns 1 |
| `webgpu_method_render_pass_end_occlusion_query.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.end-occlusion-query` sync** void | `run: async func() -> u32` | construct pass → end-occlusion-query; harness returns 1 |
| `webgpu_method_render_pass_execute_bundles.wasm` | `get-pass` + `get-render-bundle` + **`[method]gpu-render-pass-encoder.execute-bundles` sync** void | `run: async func() -> u32` | construct pass + bundle → execute-bundles(one-element list) → drop own; harness returns 1 |
| `webgpu_method_render_pass_set_immediates.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.set-immediates` sync** void | `run: async func() -> u32` | construct pass → set-immediates(range 0, empty data, offset/size none); harness returns 1 |
| `webgpu_method_render_bundle_encoder_finish.wasm` | `get-render-bundle-encoder` + **`[method]gpu-render-bundle-encoder.finish` sync** `own<gpu-render-bundle>` | `run: async func() -> u32` | construct bundle-encoder → finish (descriptor=none) → drop own; harness returns 1 |
| `webgpu_method_render_bundle_encoder_set_pipeline.wasm` | `get-render-bundle-encoder` + `get-render-pipeline` + **`[method]gpu-render-bundle-encoder.set-pipeline` sync** void | `run: async func() -> u32` | construct bundle-encoder + pipeline → set-pipeline (borrow) → drop own; harness returns 1 |
| `webgpu_method_render_bundle_encoder_set_bind_group.wasm` | `get-render-bundle-encoder` + `get-bind-group` + **`[method]gpu-render-bundle-encoder.set-bind-group` sync** `result<_, set-bind-group-error>` | `run: async func() -> u32` | construct bundle-encoder + bind-group → set-bind-group (index 0, some, offsets none) → ok → drop own; harness returns 1 |
| `webgpu_method_render_bundle_encoder_draw.wasm` | `get-render-bundle-encoder` + **`[method]gpu-render-bundle-encoder.draw` sync** void | `run: async func() -> u32` | construct bundle-encoder → draw(vertex-count=3, other options none); harness returns 1 |
| `webgpu_method_buffer_map_async.wasm` | `get-buffer` + **`[method]gpu-buffer.map-async` async** `result<_, map-async-error>` | `run: async func() -> u32` | construct buffer → map-async(READ, offset/size none) → ok; harness returns 1 |
| `webgpu_method_buffer_unmap.wasm` | `get-buffer` + **`[method]gpu-buffer.unmap` sync** `result<_, unmap-error>` | `run: async func() -> u32` | construct buffer → unmap → ok; harness returns 1 |
| `webgpu_method_buffer_get_mapped_range.wasm` | `get-buffer` + **`[method]gpu-buffer.get-mapped-range-get-with-copy` sync** `result<list<u8>, get-mapped-range-error>` | `run: async func() -> u32` | construct buffer → get (offset/size none) → ok empty list; harness returns 1 |
| `webgpu_method_buffer_set_mapped_range.wasm` | `get-buffer` + **`[method]gpu-buffer.get-mapped-range-set-with-copy` sync** `result<_, get-mapped-range-error>` | `run: async func() -> u32` | construct buffer → set (empty data, offset/size none) → ok; harness returns 1 |

**Transitional:** host still registers flat names. **W2:** true CM async (`func_wrap_concurrent` + oneshot yield); pump via `run_concurrent` / `callRunConcurrent`. **W3:** `device-get-queue`, `device-create-command-encoder`, `command-encoder-finish`, `queue-submit1`, `command-encoder-begin-render-pass-clear`, and `render-pass-end` are **sync** on the same proposal instance (same L2 u32 as experimental; submit is single-buffer, not proposal `list`; begin-clear / end use stub view `23`, instrument substitutes Cpu offscreen TextureView). **W3 `[method]`:** `get-gpu` + `[method]gpu.request-adapter` (S2: `option<own<gpu-adapter>>`), `get-adapter` + `[method]gpu-adapter.request-device` (S3: `result<own<gpu-device>, request-device-error>`), and `get-device` + `[method]gpu-device.queue` (S1: `own<gpu-queue>`) and `[method]gpu-device.create-command-encoder` (S6: Guest descriptor=none → `own<gpu-command-encoder>`) and `[method]gpu-command-encoder.finish` (S7: Guest descriptor=none → `own<gpu-command-buffer>`) and `[method]gpu-device.create-buffer` (S4: Guest `gpu-buffer-descriptor` → `own<gpu-buffer>`) and `[method]gpu-queue.submit` (S5: Guest `list<borrow<gpu-command-buffer>>` → drop owns; harness 1) and `[method]gpu-device.create-texture` (S6+: Guest `gpu-texture-descriptor` → `own<gpu-texture>`) and `[method]gpu-device.create-sampler` (S8: Guest descriptor=none → `own<gpu-sampler>`) and `[method]gpu-device.create-shader-module` (S6+: Guest `gpu-shader-module-descriptor` → `own<gpu-shader-module>`; L2 host-fixed WGSL) and `[method]gpu-queue.write-buffer-with-copy` (S6+: Guest borrow buffer + empty list → `result<_, write-buffer-error>`; L2 host-fixed 4 bytes) and `get-texture` + `[method]gpu-texture.create-view` (S8: Guest descriptor=none → `own<gpu-texture-view>`) and `[method]gpu-device.create-bind-group-layout` (S6+: Guest `gpu-bind-group-layout-descriptor` → `own<gpu-bind-group-layout>`; L2 host-fixed empty entries) and `[method]gpu-device.create-pipeline-layout` (S6+: Guest `gpu-pipeline-layout-descriptor` → `own<gpu-pipeline-layout>`; L2 host-fixed empty bind-group-layouts) and `[method]gpu-device.create-bind-group` (S6+: Guest `gpu-bind-group-descriptor` → `own<gpu-bind-group>`; L2 host-fixed empty BGL + empty entries) and `[method]gpu-device.create-render-pipeline` (S6+: Guest `gpu-render-pipeline-descriptor` → `own<gpu-render-pipeline>`; L2 host-fixed stub shader + triangle) and `[method]gpu-device.create-render-pipeline-async` (S6+: same descriptor → `result<own<gpu-render-pipeline>, create-pipeline-error>`; true CM async) and `[method]gpu-device.create-compute-pipeline` (S6+: Guest `gpu-compute-pipeline-descriptor` → `own<gpu-compute-pipeline>`; L2 host-fixed stub shader + empty layout) and `[method]gpu-device.create-compute-pipeline-async` (S6+: same descriptor → `result<own<gpu-compute-pipeline>, create-pipeline-error>`; true CM async) and `[method]gpu-buffer.get-mapped-range-get-with-copy` (S6+: offset/size none → `result<list<u8>, get-mapped-range-error>`; L2 host-fixed empty list) and `[method]gpu-buffer.get-mapped-range-set-with-copy` (S6+: empty data + offset/size none → `result<_, get-mapped-range-error>`) and `[method]gpu-queue.write-texture-with-copy` (S6+: Guest texel copy info + empty list + size 1×1×1; L2 host-fixed 1×1) and `[method]gpu-command-encoder.begin-compute-pass` (S8: Guest descriptor=none → `own<gpu-compute-pass-encoder>`) and `get-compute-pass` + `[method]gpu-compute-pass-encoder.end` (S6+: void; harness 1) and `[method]gpu-command-encoder.copy-buffer-to-buffer` (S6+: Guest borrow src/dst + option offsets/size none → drop owns; harness 1; L2 host-fixed 4-byte copy) and `[method]gpu-command-encoder.copy-buffer-to-texture` / `copy-texture-to-buffer` / `copy-texture-to-texture` (S6+: texel-copy records + size 1×1×1; L2 host-fixed 4-byte buffer copy) and `[method]gpu-command-encoder.clear-buffer` (S6+: borrow buffer + offset/size none; L2 host-fixed 4-byte buffer copy) and `[method]gpu-command-encoder.resolve-query-set` (S6+: borrow query-set/buffer; L2 unused) and `[method]gpu-command-encoder.push-debug-group` / `pop-debug-group` / `insert-debug-marker` (S6+: empty string / void; L2 unused) and `[method]gpu-compute-pass-encoder.set-pipeline` (S6+: Guest `borrow<gpu-compute-pipeline>`; L2 host-fixed compute pipeline) and `[method]gpu-compute-pass-encoder.set-bind-group` (S6+: index + option bind-group + option offsets → `result<_, set-bind-group-error>`; L2 host-fixed empty bind-group) and `[method]gpu-compute-pass-encoder.dispatch-workgroups` (S6+: x + option y/z; L2 host-fixed 1×1×1 after set-pipeline + empty bind-group) and `[method]gpu-compute-pass-encoder.dispatch-workgroups-indirect` (S6+: borrow buffer + offset 0; L2 host-fixed 1×1×1) and `[method]gpu-compute-pass-encoder.set-immediates` (S6+: range 0 + empty list + offset/size none; L2 unused) and `[method]gpu-compute-pass-encoder.push-debug-group` / `pop-debug-group` / `insert-debug-marker` (S6+: empty string / void; L2 unused) and `[method]gpu-render-pass-encoder.set-pipeline` (S6+: Guest `borrow<gpu-render-pipeline>`; L2 host-fixed triangle pipeline) and `[method]gpu-render-pass-encoder.set-bind-group` (S6+: same result shape as compute; L2 host-fixed empty bind-group) and `[method]gpu-render-pass-encoder.set-vertex-buffer` (S6+: slot + option buffer + option offset/size; L2 host-fixed VERTEX slot 0) and `[method]gpu-render-pass-encoder.set-viewport` (S6+: six f32; L2 unused) and `[method]gpu-render-pass-encoder.set-scissor-rect` (S6+: four u32; L2 unused) and `[method]gpu-render-pass-encoder.set-blend-constant` (S6+: `gpu-color`; L2 unused) and `[method]gpu-render-pass-encoder.set-stencil-reference` (S6+: u32; L2 unused) and `[method]gpu-render-pass-encoder.set-index-buffer` (S6+: borrow buffer + `gpu-index-format` + option offset/size; L2 host-fixed VERTEX slot 0) and `[method]gpu-render-pass-encoder.draw` (S6+: vertex-count + three option<u32>; L2 host-fixed draw(3) after set-pipeline) and `[method]gpu-render-pass-encoder.draw-indexed` (S6+: index-count + options none; L2 host-fixed draw(3)) and `[method]gpu-render-pass-encoder.draw-indirect` / `draw-indexed-indirect` (S6+: borrow buffer + offset 0; L2 host-fixed draw(3)) and `[method]gpu-render-pass-encoder.push-debug-group` / `pop-debug-group` / `insert-debug-marker` (S6+: empty string / void; L2 unused) and `[method]gpu-render-pass-encoder.begin-occlusion-query` / `end-occlusion-query` (S6+: query-index 0 / void; L2 unused) and `[method]gpu-render-pass-encoder.execute-bundles` (S6+: one-element `list<borrow<gpu-render-bundle>>`; L2 unused) and `[method]gpu-render-pass-encoder.set-immediates` (S6+: range 0 + empty list + offset/size none; L2 unused) and `[method]gpu-render-bundle-encoder.finish` (S6+: descriptor=none → `own<gpu-render-bundle>`; L2 unused) and `[method]gpu-render-bundle-encoder.set-pipeline` (S6+: borrow pipeline; L2 unused) and `[method]gpu-render-bundle-encoder.set-bind-group` (S6+: same result shape as render-pass; L2 unused) and `[method]gpu-render-bundle-encoder.draw` (S6+: vertex-count=3 + options none; L2 unused). Experimental flat sync path unchanged. Not full option/resource compliance.

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
wasm-tools parse fixtures/w1/webgpu_method_create_render_pipeline_async.wat -o fixtures/w1/webgpu_method_create_render_pipeline_async.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_render_pipeline_async.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_compute_pipeline.wat -o fixtures/w1/webgpu_method_create_compute_pipeline.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_compute_pipeline.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_compute_pipeline_async.wat -o fixtures/w1/webgpu_method_create_compute_pipeline_async.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_compute_pipeline_async.wasm
wasm-tools parse fixtures/w1/webgpu_method_write_texture.wat -o fixtures/w1/webgpu_method_write_texture.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_write_texture.wasm
wasm-tools parse fixtures/w1/webgpu_method_begin_compute_pass.wat -o fixtures/w1/webgpu_method_begin_compute_pass.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_begin_compute_pass.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pass_end.wat -o fixtures/w1/webgpu_method_compute_pass_end.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pass_end.wasm
wasm-tools parse fixtures/w1/webgpu_method_copy_buffer_to_buffer.wat -o fixtures/w1/webgpu_method_copy_buffer_to_buffer.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_copy_buffer_to_buffer.wasm
wasm-tools parse fixtures/w1/webgpu_method_copy_buffer_to_texture.wat -o fixtures/w1/webgpu_method_copy_buffer_to_texture.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_copy_buffer_to_texture.wasm
wasm-tools parse fixtures/w1/webgpu_method_copy_texture_to_buffer.wat -o fixtures/w1/webgpu_method_copy_texture_to_buffer.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_copy_texture_to_buffer.wasm
wasm-tools parse fixtures/w1/webgpu_method_copy_texture_to_texture.wat -o fixtures/w1/webgpu_method_copy_texture_to_texture.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_copy_texture_to_texture.wasm
wasm-tools parse fixtures/w1/webgpu_method_clear_buffer.wat -o fixtures/w1/webgpu_method_clear_buffer.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_clear_buffer.wasm
wasm-tools parse fixtures/w1/webgpu_method_resolve_query_set.wat -o fixtures/w1/webgpu_method_resolve_query_set.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_resolve_query_set.wasm
wasm-tools parse fixtures/w1/webgpu_method_push_debug_group.wat -o fixtures/w1/webgpu_method_push_debug_group.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_push_debug_group.wasm
wasm-tools parse fixtures/w1/webgpu_method_pop_debug_group.wat -o fixtures/w1/webgpu_method_pop_debug_group.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_pop_debug_group.wasm
wasm-tools parse fixtures/w1/webgpu_method_insert_debug_marker.wat -o fixtures/w1/webgpu_method_insert_debug_marker.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_insert_debug_marker.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pass_set_pipeline.wat -o fixtures/w1/webgpu_method_compute_pass_set_pipeline.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pass_set_pipeline.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pass_set_bind_group.wat -o fixtures/w1/webgpu_method_compute_pass_set_bind_group.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pass_set_bind_group.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pass_dispatch_workgroups.wat -o fixtures/w1/webgpu_method_compute_pass_dispatch_workgroups.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pass_dispatch_workgroups.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pass_dispatch_workgroups_indirect.wat -o fixtures/w1/webgpu_method_compute_pass_dispatch_workgroups_indirect.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pass_dispatch_workgroups_indirect.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pass_set_immediates.wat -o fixtures/w1/webgpu_method_compute_pass_set_immediates.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pass_set_immediates.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pass_push_debug_group.wat -o fixtures/w1/webgpu_method_compute_pass_push_debug_group.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pass_push_debug_group.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pass_pop_debug_group.wat -o fixtures/w1/webgpu_method_compute_pass_pop_debug_group.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pass_pop_debug_group.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pass_insert_debug_marker.wat -o fixtures/w1/webgpu_method_compute_pass_insert_debug_marker.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pass_insert_debug_marker.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_set_pipeline.wat -o fixtures/w1/webgpu_method_render_pass_set_pipeline.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_set_pipeline.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_draw.wat -o fixtures/w1/webgpu_method_render_pass_draw.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_draw.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_set_bind_group.wat -o fixtures/w1/webgpu_method_render_pass_set_bind_group.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_set_bind_group.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_set_vertex_buffer.wat -o fixtures/w1/webgpu_method_render_pass_set_vertex_buffer.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_set_vertex_buffer.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_set_viewport.wat -o fixtures/w1/webgpu_method_render_pass_set_viewport.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_set_viewport.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_set_scissor_rect.wat -o fixtures/w1/webgpu_method_render_pass_set_scissor_rect.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_set_scissor_rect.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_set_blend_constant.wat -o fixtures/w1/webgpu_method_render_pass_set_blend_constant.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_set_blend_constant.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_set_stencil_reference.wat -o fixtures/w1/webgpu_method_render_pass_set_stencil_reference.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_set_stencil_reference.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_set_index_buffer.wat -o fixtures/w1/webgpu_method_render_pass_set_index_buffer.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_set_index_buffer.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_draw_indexed.wat -o fixtures/w1/webgpu_method_render_pass_draw_indexed.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_draw_indexed.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_draw_indirect.wat -o fixtures/w1/webgpu_method_render_pass_draw_indirect.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_draw_indirect.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_draw_indexed_indirect.wat -o fixtures/w1/webgpu_method_render_pass_draw_indexed_indirect.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_draw_indexed_indirect.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_push_debug_group.wat -o fixtures/w1/webgpu_method_render_pass_push_debug_group.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_push_debug_group.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_pop_debug_group.wat -o fixtures/w1/webgpu_method_render_pass_pop_debug_group.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_pop_debug_group.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_insert_debug_marker.wat -o fixtures/w1/webgpu_method_render_pass_insert_debug_marker.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_insert_debug_marker.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_begin_occlusion_query.wat -o fixtures/w1/webgpu_method_render_pass_begin_occlusion_query.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_begin_occlusion_query.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_end_occlusion_query.wat -o fixtures/w1/webgpu_method_render_pass_end_occlusion_query.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_end_occlusion_query.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_execute_bundles.wat -o fixtures/w1/webgpu_method_render_pass_execute_bundles.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_execute_bundles.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_set_immediates.wat -o fixtures/w1/webgpu_method_render_pass_set_immediates.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_set_immediates.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_encoder_finish.wat -o fixtures/w1/webgpu_method_render_bundle_encoder_finish.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_encoder_finish.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_encoder_set_pipeline.wat -o fixtures/w1/webgpu_method_render_bundle_encoder_set_pipeline.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_encoder_set_pipeline.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_encoder_set_bind_group.wat -o fixtures/w1/webgpu_method_render_bundle_encoder_set_bind_group.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_encoder_set_bind_group.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_encoder_draw.wat -o fixtures/w1/webgpu_method_render_bundle_encoder_draw.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_encoder_draw.wasm
wasm-tools parse fixtures/w1/webgpu_method_buffer_map_async.wat -o fixtures/w1/webgpu_method_buffer_map_async.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_buffer_map_async.wasm
wasm-tools parse fixtures/w1/webgpu_method_buffer_unmap.wat -o fixtures/w1/webgpu_method_buffer_unmap.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_buffer_unmap.wasm
wasm-tools parse fixtures/w1/webgpu_method_buffer_get_mapped_range.wat -o fixtures/w1/webgpu_method_buffer_get_mapped_range.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_buffer_get_mapped_range.wasm
wasm-tools parse fixtures/w1/webgpu_method_buffer_set_mapped_range.wat -o fixtures/w1/webgpu_method_buffer_set_mapped_range.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_buffer_set_mapped_range.wasm
```
