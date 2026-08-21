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
| `webgpu_method_request_adapter.wasm` | `get-gpu` + **`[method]gpu.request-adapter` async** `option<own<gpu-adapter>>` | `run: async func() -> u32` | construct gpu → method request-adapter (L2 described; options none, power-preference + force-fallback forwarded when some; feature-level unused) → drop own; harness returns 1 |
| `webgpu_method_request_device.wasm` | `get-adapter` + **`[method]gpu-adapter.request-device` async** `result<own<gpu-device>, request-device-error>` | `run: async func() -> u32` | construct adapter → method request-device (L2 described; descriptor none, first required-feature forwarded when some) → drop own on ok; harness returns 1 |
| `webgpu_method_device_queue.wasm` | `get-device` + **`[method]gpu-device.queue` sync** `own<gpu-queue>` | `run: async func() -> u32` | construct device → method queue (L2 described device handle; 0 → stub-create) → drop own; harness returns 1 |
| `webgpu_method_create_command_encoder.wasm` | `get-device` + **`[method]gpu-device.create-command-encoder` sync** `own<gpu-command-encoder>` | `run: async func() -> u32` | construct device → create-encoder (label="l2") → drop own; harness returns 1 |
| `webgpu_method_begin_render_pass.wasm` | `get-encoder` + `get-texture-view` + **`[method]gpu-command-encoder.begin-render-pass` sync** `own<gpu-render-pass-encoder>` | `run: async func() -> u32` | construct encoder+view → begin (one color-attachment, load-op=clear, store-op=store) → drop owns; harness returns 1 |
| `webgpu_method_render_pass_end.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.end` sync** void | `run: async func() -> u32` | construct pass → end (described JNI uses pass rep); harness returns 1 |
| `webgpu_method_command_encoder_finish.wasm` | `get-encoder` + **`[method]gpu-command-encoder.finish` sync** `own<gpu-command-buffer>` | `run: async func() -> u32` | construct encoder → finish (label="l2") → drop own; harness returns 1 |
| `webgpu_method_queue_submit.wasm` | `get-queue` + `get-command-buffer` + **`[method]gpu-queue.submit` sync** void (`list<borrow<gpu-command-buffer>>`) | `run: async func() -> u32` | construct queue + command-buffer → submit(one-element list, L2 described handles) → drop owns; harness returns 1 |
| `webgpu_method_dawn_compute_slice.wasm` | `get-device` → **`[method]gpu-device.create-buffer`** + **`[method]gpu-device.create-command-encoder`** + **`[method]gpu-device.queue`** + **`[method]gpu-command-encoder.begin-compute-pass`** + **`[method]gpu-compute-pass-encoder.end`** + **`[method]gpu-command-encoder.finish`** + **`[method]gpu-queue.submit`** | `run: async func() -> u32` | D1 cite: buffer + encoder + one compute pass + submit (descriptor none); harness returns 1 |
| `webgpu_method_create_buffer.wasm` | `get-device` + **`[method]gpu-device.create-buffer` sync** `own<gpu-buffer>` | `run: async func() -> u32` | construct device → create-buffer (`gpu-buffer-descriptor` size=4 COPY_DST\|VERTEX) → drop own; harness returns 1 |
| `webgpu_method_create_texture.wasm` | `get-device` + **`[method]gpu-device.create-texture` sync** `own<gpu-texture>` | `run: async func() -> u32` | construct device → create-texture (`gpu-texture-descriptor` 1×1×1 rgba8unorm RENDER_ATTACHMENT) → drop own; harness returns 1 |
| `webgpu_method_create_sampler.wasm` | `get-device` + **`[method]gpu-device.create-sampler` sync** `own<gpu-sampler>` | `run: async func() -> u32` | construct device → create-sampler (`option<gpu-sampler-descriptor>` some: address-mode-u=repeat, mag/min-filter=linear) → drop own; harness returns 1 |
| `webgpu_method_canvas_context_configure.wasm` | `get-canvas-context` + `get-device` + **`[method]gpu-canvas-context.configure` sync** void | `run: async func() -> u32` | construct context+device → configure (format=rgba8unorm, options none, L2 described device/format/usage); harness returns 1 |
| `webgpu_method_canvas_context_unconfigure.wasm` | `get-canvas-context` + **`[method]gpu-canvas-context.unconfigure` sync** void | `run: async func() -> u32` | construct context → unconfigure (L2 described handle; 0 is a no-op); harness returns 1 |
| `webgpu_method_canvas_context_get_configuration.wasm` | `get-canvas-context` + **`[method]gpu-canvas-context.get-configuration` sync** `option<gpu-canvas-configuration-owned>` | `run: async func() -> u32` | construct context → get-configuration (L2 described; unconfigured → none); harness returns 1 |
| `webgpu_method_canvas_context_get_current_texture.wasm` | `get-canvas-context` + **`[method]gpu-canvas-context.get-current-texture` sync** `own<gpu-texture>` | `run: async func() -> u32` | construct context → get-current-texture (L2 described handle; 0 → 1×1 texture) → drop own; harness returns 1 |
| `webgpu_method_sampler_label.wasm` | `get-sampler` + **`[method]gpu-sampler.label` sync** `string` | `run: async func() -> u32` | construct sampler → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_sampler_set_label.wasm` | `get-sampler` + **`[method]gpu-sampler.set-label` sync** void | `run: async func() -> u32` | construct sampler → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_create_shader_module.wasm` | `get-device` + **`[method]gpu-device.create-shader-module` sync** `own<gpu-shader-module>` | `run: async func() -> u32` | construct device → create-shader-module (WGSL `fn l2`) → drop own; harness returns 1 |
| `webgpu_method_shader_module_get_compilation_info.wasm` | `get-shader-module` + **`[method]gpu-shader-module.get-compilation-info` async** `own<gpu-compilation-info>` | `run: async func() -> u32` | construct shader → get-compilation-info (L2 described handle validate) → drop own info; harness returns 1 |
| `webgpu_method_shader_module_label.wasm` | `get-shader-module` + **`[method]gpu-shader-module.label` sync** `string` | `run: async func() -> u32` | construct shader-module → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_shader_module_set_label.wasm` | `get-shader-module` + **`[method]gpu-shader-module.set-label` sync** void | `run: async func() -> u32` | construct shader-module → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_write_buffer.wasm` | `get-queue` + `get-buffer` + **`[method]gpu-queue.write-buffer-with-copy` sync** `result<_, write-buffer-error>` | `run: async func() -> u32` | construct queue + buffer → write (offset 0, 4-byte data `l2`, offset/size none) → ok → drop buffer; harness returns 1 |
| `webgpu_method_texture_create_view.wasm` | `get-texture` + **`[method]gpu-texture.create-view` sync** `own<gpu-texture-view>` | `run: async func() -> u32` | construct texture → create-view (`option<gpu-texture-view-descriptor>` some: dimension=d2, aspect=all) → drop own; harness returns 1 |
| `webgpu_method_texture_view_label.wasm` | `get-texture-view` + **`[method]gpu-texture-view.label` sync** `string` | `run: async func() -> u32` | construct texture-view → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_texture_view_set_label.wasm` | `get-texture-view` + **`[method]gpu-texture-view.set-label` sync** void | `run: async func() -> u32` | construct texture-view → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_create_bind_group_layout.wasm` | `get-device` + **`[method]gpu-device.create-bind-group-layout` sync** `own<gpu-bind-group-layout>` | `run: async func() -> u32` | construct device → create-bind-group-layout (two buffer entries: binding=0 uniform, binding=1 storage) → drop own; harness returns 1 |
| `webgpu_method_create_pipeline_layout.wasm` | `get-device` + **`[method]gpu-device.create-pipeline-layout` sync** `own<gpu-pipeline-layout>` | `run: async func() -> u32` | construct device → create-pipeline-layout (empty bind-group-layouts, label="l2") → drop own; harness returns 1 |
| `webgpu_method_create_bind_group.wasm` | `get-device` + `get-bind-group-layout` + `get-buffer` + **`[method]gpu-device.create-bind-group` sync** `own<gpu-bind-group>` | `run: async func() -> u32` | construct device + layout + buffer → create-bind-group (one gpu-buffer entry binding=0, label="l2") → drop owns; harness returns 1 |
| `webgpu_method_create_render_pipeline.wasm` | `get-device` + `get-shader-module` + **`[method]gpu-device.create-render-pipeline` sync** `own<gpu-render-pipeline>` | `run: async func() -> u32` | construct device + shader → create-render-pipeline (shader borrow, vertex entry-point="vs_main", layout auto, label="l2") → drop own; harness returns 1 |
| `webgpu_method_create_render_pipeline_async.wasm` | `get-device` + `get-shader-module` + **`[method]gpu-device.create-render-pipeline-async` async** `result<own<gpu-render-pipeline>, create-pipeline-error>` | `run: async func() -> u32` | same descriptor as sync create; drop own on ok; harness returns 1 |
| `webgpu_method_create_compute_pipeline.wasm` | `get-device` + `get-shader-module` + **`[method]gpu-device.create-compute-pipeline` sync** `own<gpu-compute-pipeline>` | `run: async func() -> u32` | construct device + shader → create-compute-pipeline (shader borrow, entry-point="main", layout auto, label="l2") → drop own; harness returns 1 |
| `webgpu_method_create_compute_pipeline_async.wasm` | `get-device` + `get-shader-module` + **`[method]gpu-device.create-compute-pipeline-async` async** `result<own<gpu-compute-pipeline>, create-pipeline-error>` | `run: async func() -> u32` | same descriptor as sync create; drop own on ok; harness returns 1 |
| `webgpu_method_write_texture.wasm` | `get-queue` + `get-texture` + **`[method]gpu-queue.write-texture-with-copy` sync** void | `run: async func() -> u32` | construct queue + texture → write (4-byte data `l2`, bytes-per-row=4, size 1×1×1) → drop texture; harness returns 1 |
| `webgpu_method_begin_compute_pass.wasm` | `get-encoder` + `get-query-set` + **`[method]gpu-command-encoder.begin-compute-pass` sync** `own<gpu-compute-pass-encoder>` | `run: async func() -> u32` | construct encoder+query-set → begin-compute-pass (timestamp-writes beginning=0, end=1) → drop owns; harness returns 1 |
| `webgpu_method_compute_pass_end.wasm` | `get-compute-pass` + **`[method]gpu-compute-pass-encoder.end` sync** void | `run: async func() -> u32` | construct compute-pass → end (L2 described pass rep); harness returns 1 |
| `webgpu_method_copy_buffer_to_buffer.wasm` | `get-encoder` + `get-buffer` + **`[method]gpu-command-encoder.copy-buffer-to-buffer` sync** void | `run: async func() -> u32` | construct encoder + two buffers → copy (offsets some(0), size some(4)) → drop owns; harness returns 1 |
| `webgpu_method_copy_buffer_to_texture.wasm` | `get-encoder` + `get-buffer` + `get-texture` + **`[method]gpu-command-encoder.copy-buffer-to-texture` sync** void | `run: async func() -> u32` | construct encoder + buffer + texture → copy (layout/mip/origin/aspect none, size 1×1×1) → drop owns; harness returns 1 |
| `webgpu_method_copy_texture_to_buffer.wasm` | `get-encoder` + `get-texture` + `get-buffer` + **`[method]gpu-command-encoder.copy-texture-to-buffer` sync** void | `run: async func() -> u32` | construct encoder + texture + buffer → copy (layout/mip/origin/aspect none, size 1×1×1) → drop owns; harness returns 1 |
| `webgpu_method_copy_texture_to_texture.wasm` | `get-encoder` + `get-texture` + **`[method]gpu-command-encoder.copy-texture-to-texture` sync** void | `run: async func() -> u32` | construct encoder + two textures → copy (mip/origin/aspect none, size 1×1×1) → drop owns; harness returns 1 |
| `webgpu_method_clear_buffer.wasm` | `get-encoder` + `get-buffer` + **`[method]gpu-command-encoder.clear-buffer` sync** void | `run: async func() -> u32` | construct encoder + buffer → clear (offset some(0), size some(4)) → drop own; harness returns 1 |
| `webgpu_method_resolve_query_set.wasm` | `get-encoder` + `get-query-set` + `get-buffer` + **`[method]gpu-command-encoder.resolve-query-set` sync** void | `run: async func() -> u32` | construct encoder + query-set + buffer → resolve (first 0, count 1, offset 0; L2 described handles, 0 → stub); harness returns 1 |
| `webgpu_method_push_debug_group.wasm` | `get-encoder` + **`[method]gpu-command-encoder.push-debug-group` sync** void | `run: async func() -> u32` | construct encoder → push-debug-group("") (L2 described handle + label); harness returns 1 |
| `webgpu_method_pop_debug_group.wasm` | `get-encoder` + **`[method]gpu-command-encoder.pop-debug-group` sync** void | `run: async func() -> u32` | construct encoder → pop-debug-group (L2 described handle); harness returns 1 |
| `webgpu_method_insert_debug_marker.wasm` | `get-encoder` + **`[method]gpu-command-encoder.insert-debug-marker` sync** void | `run: async func() -> u32` | construct encoder → insert-debug-marker("") (L2 described handle + label); harness returns 1 |
| `webgpu_method_compute_pass_set_pipeline.wasm` | `get-compute-pass` + `get-compute-pipeline` + **`[method]gpu-compute-pass-encoder.set-pipeline` sync** void | `run: async func() -> u32` | construct compute-pass + pipeline → set-pipeline (borrow; L2 described pass+pipeline reps) → drop own; harness returns 1 |
| `webgpu_method_compute_pass_set_bind_group.wasm` | `get-compute-pass` + `get-bind-group` + **`[method]gpu-compute-pass-encoder.set-bind-group` sync** `result<_, set-bind-group-error>` | `run: async func() -> u32` | construct compute-pass + bind-group → set-bind-group (index 0, some, offsets none; L2 described JNI) → ok → drop own; harness returns 1 |
| `webgpu_method_compute_pass_dispatch_workgroups.wasm` | `get-compute-pass` + **`[method]gpu-compute-pass-encoder.dispatch-workgroups` sync** void | `run: async func() -> u32` | construct compute-pass → dispatch(x=1, y/z=some(1); L2 described JNI); harness returns 1 |
| `webgpu_method_compute_pass_dispatch_workgroups_indirect.wasm` | `get-compute-pass` + `get-buffer` + **`[method]gpu-compute-pass-encoder.dispatch-workgroups-indirect` sync** void | `run: async func() -> u32` | construct compute-pass + buffer → dispatch-indirect(offset 0; L2 described JNI) → drop own; harness returns 1 |
| `webgpu_method_compute_pass_set_immediates.wasm` | `get-compute-pass` + **`[method]gpu-compute-pass-encoder.set-immediates` sync** void | `run: async func() -> u32` | construct compute-pass → set-immediates(range 0, empty data, offset/size none; L2 described bytes); harness returns 1 |
| `webgpu_method_compute_pass_push_debug_group.wasm` | `get-compute-pass` + **`[method]gpu-compute-pass-encoder.push-debug-group` sync** void | `run: async func() -> u32` | construct compute-pass → push-debug-group("") (L2 described pass + label); harness returns 1 |
| `webgpu_method_compute_pass_pop_debug_group.wasm` | `get-compute-pass` + **`[method]gpu-compute-pass-encoder.pop-debug-group` sync** void | `run: async func() -> u32` | construct compute-pass → pop-debug-group (L2 described pass); harness returns 1 |
| `webgpu_method_compute_pass_insert_debug_marker.wasm` | `get-compute-pass` + **`[method]gpu-compute-pass-encoder.insert-debug-marker` sync** void | `run: async func() -> u32` | construct compute-pass → insert-debug-marker("") (L2 described pass + label); harness returns 1 |
| `webgpu_method_render_pass_set_pipeline.wasm` | `get-pass` + `get-render-pipeline` + **`[method]gpu-render-pass-encoder.set-pipeline` sync** void | `run: async func() -> u32` | construct pass + pipeline → set-pipeline (borrow; L2 described pass+pipeline reps) → drop own; harness returns 1 |
| `webgpu_method_render_pass_draw.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.draw` sync** void | `run: async func() -> u32` | construct pass → draw(vertex-count=3, other options none); harness returns 1 |
| `webgpu_method_render_pass_set_bind_group.wasm` | `get-pass` + `get-bind-group` + **`[method]gpu-render-pass-encoder.set-bind-group` sync** `result<_, set-bind-group-error>` | `run: async func() -> u32` | construct pass + bind-group → set-bind-group (index 0, some, offsets none; L2 described JNI) → ok → drop own; harness returns 1 |
| `webgpu_method_render_pass_set_vertex_buffer.wasm` | `get-pass` + `get-buffer` + **`[method]gpu-render-pass-encoder.set-vertex-buffer` sync** void | `run: async func() -> u32` | construct pass + buffer → set-vertex-buffer (slot 0, some, offset/size none; L2 described JNI) → drop own; harness returns 1 |
| `webgpu_method_render_pass_set_viewport.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.set-viewport` sync** void | `run: async func() -> u32` | construct pass → set-viewport(0,0,1,1,0,1) (L2 described floats); harness returns 1 |
| `webgpu_method_render_pass_set_scissor_rect.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.set-scissor-rect` sync** void | `run: async func() -> u32` | construct pass → set-scissor-rect(0,0,1,1) (L2 described ints); harness returns 1 |
| `webgpu_method_render_pass_set_blend_constant.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.set-blend-constant` sync** void | `run: async func() -> u32` | construct pass → set-blend-constant(0,0,0,1) (L2 described color); harness returns 1 |
| `webgpu_method_render_pass_set_stencil_reference.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.set-stencil-reference` sync** void | `run: async func() -> u32` | construct pass → set-stencil-reference(0) (L2 described u32); harness returns 1 |
| `webgpu_method_render_pass_set_index_buffer.wasm` | `get-pass` + `get-buffer` + **`[method]gpu-render-pass-encoder.set-index-buffer` sync** void | `run: async func() -> u32` | construct pass + buffer → set-index-buffer (uint16, offset/size none; L2 described JNI) → drop own; harness returns 1 |
| `webgpu_method_render_pass_draw_indexed.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.draw-indexed` sync** void | `run: async func() -> u32` | construct pass → draw-indexed(index-count=3, other options none); harness returns 1 |
| `webgpu_method_render_pass_draw_indirect.wasm` | `get-pass` + `get-buffer` + **`[method]gpu-render-pass-encoder.draw-indirect` sync** void | `run: async func() -> u32` | construct pass + buffer → draw-indirect(offset 0; L2 described JNI) → drop own; harness returns 1 |
| `webgpu_method_render_pass_draw_indexed_indirect.wasm` | `get-pass` + `get-buffer` + **`[method]gpu-render-pass-encoder.draw-indexed-indirect` sync** void | `run: async func() -> u32` | construct pass + buffer → draw-indexed-indirect(offset 0; L2 described JNI) → drop own; harness returns 1 |
| `webgpu_method_render_pass_push_debug_group.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.push-debug-group` sync** void | `run: async func() -> u32` | construct pass → push-debug-group("") (L2 described pass + label); harness returns 1 |
| `webgpu_method_render_pass_pop_debug_group.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.pop-debug-group` sync** void | `run: async func() -> u32` | construct pass → pop-debug-group (L2 described pass); harness returns 1 |
| `webgpu_method_render_pass_insert_debug_marker.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.insert-debug-marker` sync** void | `run: async func() -> u32` | construct pass → insert-debug-marker("") (L2 described pass + label); harness returns 1 |
| `webgpu_method_render_pass_begin_occlusion_query.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.begin-occlusion-query` sync** void | `run: async func() -> u32` | construct pass → begin-occlusion-query(0) (L2 described index); harness returns 1 |
| `webgpu_method_render_pass_end_occlusion_query.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.end-occlusion-query` sync** void | `run: async func() -> u32` | construct pass → end-occlusion-query (L2 described pass); harness returns 1 |
| `webgpu_method_render_pass_execute_bundles.wasm` | `get-pass` + `get-render-bundle` + **`[method]gpu-render-pass-encoder.execute-bundles` sync** void | `run: async func() -> u32` | construct pass + bundle → execute-bundles(one-element list; L2 described reps, 0 skipped) → drop own; harness returns 1 |
| `webgpu_method_render_pass_set_immediates.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.set-immediates` sync** void | `run: async func() -> u32` | construct pass → set-immediates(range 0, empty data, offset/size none; L2 described bytes); harness returns 1 |
| `webgpu_method_render_bundle_encoder_finish.wasm` | `get-render-bundle-encoder` + **`[method]gpu-render-bundle-encoder.finish` sync** `own<gpu-render-bundle>` | `run: async func() -> u32` | construct bundle-encoder → finish (descriptor=none; L2 described handle → bundle rep) → drop own; harness returns 1 |
| `webgpu_method_render_bundle_encoder_set_pipeline.wasm` | `get-render-bundle-encoder` + `get-render-pipeline` + **`[method]gpu-render-bundle-encoder.set-pipeline` sync** void | `run: async func() -> u32` | construct bundle-encoder + pipeline → set-pipeline (borrow; L2 described reps, 0 → stub) → drop own; harness returns 1 |
| `webgpu_method_render_bundle_encoder_set_bind_group.wasm` | `get-render-bundle-encoder` + `get-bind-group` + **`[method]gpu-render-bundle-encoder.set-bind-group` sync** `result<_, set-bind-group-error>` | `run: async func() -> u32` | construct bundle-encoder + bind-group → set-bind-group (index 0, some, offsets none; L2 described reps, 0 → stub) → ok → drop own; harness returns 1 |
| `webgpu_method_render_bundle_encoder_draw.wasm` | `get-render-bundle-encoder` + **`[method]gpu-render-bundle-encoder.draw` sync** void | `run: async func() -> u32` | construct bundle-encoder → draw(vertex-count=3, other options none; L2 described counts); harness returns 1 |
| `webgpu_method_render_bundle_encoder_set_index_buffer.wasm` | `get-render-bundle-encoder` + `get-buffer` + **`[method]gpu-render-bundle-encoder.set-index-buffer` sync** void | `run: async func() -> u32` | construct bundle-encoder + buffer → set-index-buffer (uint16, offset/size none; L2 described reps) → drop own; harness returns 1 |
| `webgpu_method_render_bundle_encoder_set_vertex_buffer.wasm` | `get-render-bundle-encoder` + `get-buffer` + **`[method]gpu-render-bundle-encoder.set-vertex-buffer` sync** void | `run: async func() -> u32` | construct bundle-encoder + buffer → set-vertex-buffer (slot 0, some, offset/size none; L2 described reps) → drop own; harness returns 1 |
| `webgpu_method_render_bundle_encoder_draw_indexed.wasm` | `get-render-bundle-encoder` + **`[method]gpu-render-bundle-encoder.draw-indexed` sync** void | `run: async func() -> u32` | construct bundle-encoder → draw-indexed(index-count=3, other options none; L2 described counts); harness returns 1 |
| `webgpu_method_render_bundle_encoder_draw_indirect.wasm` | `get-render-bundle-encoder` + `get-buffer` + **`[method]gpu-render-bundle-encoder.draw-indirect` sync** void | `run: async func() -> u32` | construct bundle-encoder + buffer → draw-indirect(offset 0; L2 described reps) → drop own; harness returns 1 |
| `webgpu_method_render_bundle_encoder_draw_indexed_indirect.wasm` | `get-render-bundle-encoder` + `get-buffer` + **`[method]gpu-render-bundle-encoder.draw-indexed-indirect` sync** void | `run: async func() -> u32` | construct bundle-encoder + buffer → draw-indexed-indirect(offset 0; L2 described reps) → drop own; harness returns 1 |
| `webgpu_method_render_bundle_encoder_push_debug_group.wasm` | `get-render-bundle-encoder` + **`[method]gpu-render-bundle-encoder.push-debug-group` sync** void | `run: async func() -> u32` | construct bundle-encoder → push-debug-group("") (L2 described handle + label); harness returns 1 |
| `webgpu_method_render_bundle_encoder_pop_debug_group.wasm` | `get-render-bundle-encoder` + **`[method]gpu-render-bundle-encoder.pop-debug-group` sync** void | `run: async func() -> u32` | construct bundle-encoder → pop-debug-group (L2 described handle); harness returns 1 |
| `webgpu_method_render_bundle_encoder_insert_debug_marker.wasm` | `get-render-bundle-encoder` + **`[method]gpu-render-bundle-encoder.insert-debug-marker` sync** void | `run: async func() -> u32` | construct bundle-encoder → insert-debug-marker("") (L2 described handle + label); harness returns 1 |
| `webgpu_method_render_bundle_encoder_set_immediates.wasm` | `get-render-bundle-encoder` + **`[method]gpu-render-bundle-encoder.set-immediates` sync** void | `run: async func() -> u32` | construct bundle-encoder → set-immediates(range 0, empty data, offset/size none; L2 described bytes); harness returns 1 |
| `webgpu_method_buffer_map_async.wasm` | `get-buffer` + **`[method]gpu-buffer.map-async` async** `result<_, map-async-error>` | `run: async func() -> u32` | construct buffer → map-async(READ, offset/size none) → ok; harness returns 1 |
| `webgpu_method_buffer_unmap.wasm` | `get-buffer` + **`[method]gpu-buffer.unmap` sync** `result<_, unmap-error>` | `run: async func() -> u32` | construct buffer → unmap → ok (L2 described buffer rep); harness returns 1 |
| `webgpu_method_buffer_get_mapped_range.wasm` | `get-buffer` + **`[method]gpu-buffer.get-mapped-range-get-with-copy` sync** `result<list<u8>, get-mapped-range-error>` | `run: async func() -> u32` | construct buffer → get (offset/size none; L2 described handle, stub maps first) → ok bytes; harness returns 1 |
| `webgpu_method_buffer_set_mapped_range.wasm` | `get-buffer` + **`[method]gpu-buffer.get-mapped-range-set-with-copy` sync** `result<_, get-mapped-range-error>` | `run: async func() -> u32` | construct buffer → set (empty data, offset/size none; L2 described handle, stub maps first) → ok; harness returns 1 |
| `webgpu_method_create_render_bundle_encoder.wasm` | `get-device` + **`[method]gpu-device.create-render-bundle-encoder` sync** `own<gpu-render-bundle-encoder>` | `run: async func() -> u32` | construct device → create-render-bundle-encoder (L2 described format/sample-count; empty color-formats → RGBA8) → drop own; harness returns 1 |
| `webgpu_method_create_query_set.wasm` | `get-device` + **`[method]gpu-device.create-query-set` sync** `result<own<gpu-query-set>, create-query-set-error>` | `run: async func() -> u32` | construct device → create-query-set (type=occlusion, count=1; L2 described fields) → drop own on ok; harness returns 1 |
| `webgpu_method_device_destroy.wasm` | `get-device` + **`[method]gpu-device.destroy` sync** void | `run: async func() -> u32` | construct device → destroy (L2 described handle); harness returns 1 |
| `webgpu_method_buffer_destroy.wasm` | `get-buffer` + **`[method]gpu-buffer.destroy` sync** void | `run: async func() -> u32` | construct buffer → destroy (L2 described handle; stub 4 bytes); harness returns 1 |
| `webgpu_method_texture_destroy.wasm` | `get-texture` + **`[method]gpu-texture.destroy` sync** void | `run: async func() -> u32` | construct texture → destroy (L2 described handle; stub 1×1); harness returns 1 |
| `webgpu_method_texture_width.wasm` | `get-texture` + **`[method]gpu-texture.width` sync** `u32` | `run: async func() -> u32` | construct texture → width (L2 described handle; stub 1×1) ; harness returns 1 |
| `webgpu_method_texture_height.wasm` | `get-texture` + **`[method]gpu-texture.height` sync** `u32` | `run: async func() -> u32` | construct texture → height (L2 described handle; stub 1×1); harness returns 1 |
| `webgpu_method_texture_depth_or_array_layers.wasm` | `get-texture` + **`[method]gpu-texture.depth-or-array-layers` sync** `u32` | `run: async func() -> u32` | construct texture → depth-or-array-layers (L2 described handle; stub 1×1); harness returns 1 |
| `webgpu_method_texture_mip_level_count.wasm` | `get-texture` + **`[method]gpu-texture.mip-level-count` sync** `u32` | `run: async func() -> u32` | construct texture → mip-level-count (L2 described handle; stub 1×1); harness returns 1 |
| `webgpu_method_texture_sample_count.wasm` | `get-texture` + **`[method]gpu-texture.sample-count` sync** `u32` | `run: async func() -> u32` | construct texture → sample-count (L2 described handle; stub 1×1); harness returns 1 |
| `webgpu_method_texture_dimension.wasm` | `get-texture` + **`[method]gpu-texture.dimension` sync** `gpu-texture-dimension` | `run: async func() -> u32` | construct texture → dimension (L2 described handle; stub 1×1 d2); harness returns 1 |
| `webgpu_method_texture_format.wasm` | `get-texture` + **`[method]gpu-texture.format` sync** `gpu-texture-format` | `run: async func() -> u32` | construct texture → format (L2 described handle; stub rgba8unorm); harness returns 1 |
| `webgpu_method_texture_usage.wasm` | `get-texture` + **`[method]gpu-texture.usage` sync** `gpu-texture-usage` | `run: async func() -> u32` | construct texture → usage (L2 described handle; stub RENDER_ATTACHMENT); harness returns 1 |
| `webgpu_method_texture_texture_binding_view_dimension.wasm` | `get-texture` + **`[method]gpu-texture.texture-binding-view-dimension` sync** `option<gpu-texture-view-dimension>` | `run: async func() -> u32` | construct texture → texture-binding-view-dimension (L2 described handle; stub none); harness returns 1 |
| `webgpu_method_texture_label.wasm` | `get-texture` + **`[method]gpu-texture.label` sync** `string` | `run: async func() -> u32` | construct texture → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_texture_set_label.wasm` | `get-texture` + **`[method]gpu-texture.set-label` sync** void | `run: async func() -> u32` | construct texture → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_query_set_destroy.wasm` | `get-query-set` + **`[method]gpu-query-set.destroy` sync** void | `run: async func() -> u32` | construct query-set → destroy (L2 described handle; stub occlusion count 1); harness returns 1 |
| `webgpu_method_query_set_type.wasm` | `get-query-set` + **`[method]gpu-query-set.type` sync** `gpu-query-type` | `run: async func() -> u32` | construct query-set → type (L2 described handle; stub occlusion); harness returns 1 |
| `webgpu_method_query_set_count.wasm` | `get-query-set` + **`[method]gpu-query-set.count` sync** `u32` | `run: async func() -> u32` | construct query-set → count (L2 described handle; stub 1); harness returns 1 |
| `webgpu_method_adapter_features.wasm` | `get-adapter` + **`[method]gpu-adapter.features` sync** `own<gpu-supported-features>` | `run: async func() -> u32` | construct adapter → features (L2 described handle validate) → drop own; harness returns 1 |
| `webgpu_method_adapter_limits.wasm` | `get-adapter` + **`[method]gpu-adapter.limits` sync** `own<gpu-supported-limits>` | `run: async func() -> u32` | construct adapter → limits (L2 described handle validate) → drop own; harness returns 1 |
| `webgpu_method_adapter_info.wasm` | `get-adapter` + **`[method]gpu-adapter.info` sync** `own<gpu-adapter-info>` | `run: async func() -> u32` | construct adapter → info (L2 described handle validate) → drop own; harness returns 1 |
| `webgpu_method_adapter_info_vendor.wasm` | `get-adapter-info` + **`[method]gpu-adapter-info.vendor` sync** `string` | `run: async func() -> u32` | construct adapter-info → vendor (L2 described adapter handle; Cpu stub `cpu-vendor`); harness returns 1 |
| `webgpu_method_adapter_info_architecture.wasm` | `get-adapter-info` + **`[method]gpu-adapter-info.architecture` sync** `string` | `run: async func() -> u32` | construct adapter-info → architecture (L2 described adapter handle; Cpu stub `cpu-arch`); harness returns 1 |
| `webgpu_method_adapter_info_device.wasm` | `get-adapter-info` + **`[method]gpu-adapter-info.device` sync** `string` | `run: async func() -> u32` | construct adapter-info → device (L2 described adapter handle; Cpu stub `cpu-device`); harness returns 1 |
| `webgpu_method_adapter_info_description.wasm` | `get-adapter-info` + **`[method]gpu-adapter-info.description` sync** `string` | `run: async func() -> u32` | construct adapter-info → description (L2 described adapter handle; Cpu stub `cpu-desc`); harness returns 1 |
| `webgpu_method_adapter_info_subgroup_min_size.wasm` | `get-adapter-info` + **`[method]gpu-adapter-info.subgroup-min-size` sync** `u32` | `run: async func() -> u32` | construct adapter-info → subgroup-min-size (L2 described adapter handle; Cpu stub 4); harness returns 1 |
| `webgpu_method_adapter_info_subgroup_max_size.wasm` | `get-adapter-info` + **`[method]gpu-adapter-info.subgroup-max-size` sync** `u32` | `run: async func() -> u32` | construct adapter-info → subgroup-max-size (L2 described adapter handle; Cpu stub 128); harness returns 1 |
| `webgpu_method_adapter_info_is_fallback_adapter.wasm` | `get-adapter-info` + **`[method]gpu-adapter-info.is-fallback-adapter` sync** `bool` | `run: async func() -> u32` | construct adapter-info → is-fallback-adapter (L2 described adapter handle; Cpu stub false); harness returns 1 |
| `webgpu_method_bind_group_label.wasm` | `get-bind-group` + **`[method]gpu-bind-group.label` sync** `string` | `run: async func() -> u32` | construct bind-group → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_bind_group_set_label.wasm` | `get-bind-group` + **`[method]gpu-bind-group.set-label` sync** void | `run: async func() -> u32` | construct bind-group → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_bind_group_layout_label.wasm` | `get-bind-group-layout` + **`[method]gpu-bind-group-layout.label` sync** `string` | `run: async func() -> u32` | construct bind-group-layout → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_bind_group_layout_set_label.wasm` | `get-bind-group-layout` + **`[method]gpu-bind-group-layout.set-label` sync** void | `run: async func() -> u32` | construct bind-group-layout → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_buffer_label.wasm` | `get-buffer` + **`[method]gpu-buffer.label` sync** `string` | `run: async func() -> u32` | construct buffer → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_buffer_set_label.wasm` | `get-buffer` + **`[method]gpu-buffer.set-label` sync** void | `run: async func() -> u32` | construct buffer → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_buffer_size.wasm` | `get-buffer` + **`[method]gpu-buffer.size` sync** `gpu-size64-out` | `run: async func() -> u32` | construct buffer → size (L2 described handle; stub 4 bytes); harness returns 1 |
| `webgpu_method_buffer_usage.wasm` | `get-buffer` + **`[method]gpu-buffer.usage` sync** `gpu-buffer-usage` | `run: async func() -> u32` | construct buffer → usage (L2 described handle; stub MAP_READ\|COPY_DST); harness returns 1 |
| `webgpu_method_buffer_map_state.wasm` | `get-buffer` + **`[method]gpu-buffer.map-state` sync** `gpu-buffer-map-state` | `run: async func() -> u32` | construct buffer → map-state (L2 described handle; stub unmapped); harness returns 1 |
| `webgpu_method_command_buffer_label.wasm` | `get-command-buffer` + **`[method]gpu-command-buffer.label` sync** `string` | `run: async func() -> u32` | construct command-buffer → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_command_buffer_set_label.wasm` | `get-command-buffer` + **`[method]gpu-command-buffer.set-label` sync** void | `run: async func() -> u32` | construct command-buffer → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_command_encoder_label.wasm` | `get-encoder` + **`[method]gpu-command-encoder.label` sync** `string` | `run: async func() -> u32` | construct command-encoder → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_command_encoder_set_label.wasm` | `get-encoder` + **`[method]gpu-command-encoder.set-label` sync** void | `run: async func() -> u32` | construct command-encoder → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_compilation_info_messages.wasm` | `get-compilation-info` + **`[method]gpu-compilation-info.messages` sync** `list` | `run: async func() -> u32` | construct compilation-info → messages (L2 described; Cpu stub count=1); harness returns 1 |
| `webgpu_method_compilation_message_message.wasm` | `get-compilation-message` + **`[method]gpu-compilation-message.message` sync** `string` | `run: async func() -> u32` | construct compilation-message → message (L2 described; Cpu stub `cpu-compilation-message`); harness returns 1 |
| `webgpu_method_compilation_message_type.wasm` | `get-compilation-message` + **`[method]gpu-compilation-message.type` sync** enum | `run: async func() -> u32` | construct compilation-message → type (L2 described shader-module handle; Cpu stub error); harness returns 1 |
| `webgpu_method_compilation_message_length.wasm` | `get-compilation-message` + **`[method]gpu-compilation-message.length` sync** `u64` | `run: async func() -> u32` | construct compilation-message → length (L2 described; Cpu stub 256); harness returns 1 |
| `webgpu_method_compilation_message_line_num.wasm` | `get-compilation-message` + **`[method]gpu-compilation-message.line-num` sync** `u64` | `run: async func() -> u32` | construct compilation-message → line-num (L2 described; Cpu stub 42); harness returns 1 |
| `webgpu_method_compilation_message_line_pos.wasm` | `get-compilation-message` + **`[method]gpu-compilation-message.line-pos` sync** `u64` | `run: async func() -> u32` | construct compilation-message → line-pos (L2 described; Cpu stub 7); harness returns 1 |
| `webgpu_method_compilation_message_offset.wasm` | `get-compilation-message` + **`[method]gpu-compilation-message.offset` sync** `u64` | `run: async func() -> u32` | construct compilation-message → offset (L2 described; Cpu stub 100); harness returns 1 |
| `webgpu_method_compute_pass_label.wasm` | `get-compute-pass` + **`[method]gpu-compute-pass-encoder.label` sync** `string` | `run: async func() -> u32` | construct compute-pass-encoder → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_compute_pass_set_label.wasm` | `get-compute-pass` + **`[method]gpu-compute-pass-encoder.set-label` sync** void | `run: async func() -> u32` | construct compute-pass-encoder → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_compute_pipeline_label.wasm` | `get-compute-pipeline` + **`[method]gpu-compute-pipeline.label` sync** `string` | `run: async func() -> u32` | construct compute-pipeline → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_compute_pipeline_set_label.wasm` | `get-compute-pipeline` + **`[method]gpu-compute-pipeline.set-label` sync** void | `run: async func() -> u32` | construct compute-pipeline → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_compute_pipeline_get_bind_group_layout.wasm` | `get-compute-pipeline` + **`[method]gpu-compute-pipeline.get-bind-group-layout` sync** `own<gpu-bind-group-layout>` | `run: async func() -> u32` | construct compute-pipeline → get-bind-group-layout (index 0; L2 described handle, 0 → stub) → drop own; harness returns 1 |
| `webgpu_method_device_adapter_info.wasm` | `get-device` + **`[method]gpu-device.adapter-info` sync** `own<gpu-adapter-info>` | `run: async func() -> u32` | construct device → adapter-info (L2 described handle validate) → drop own; harness returns 1 |
| `webgpu_method_device_features.wasm` | `get-device` + **`[method]gpu-device.features` sync** `own<gpu-supported-features>` | `run: async func() -> u32` | construct device → features (L2 described handle validate) → drop own; harness returns 1 |
| `webgpu_method_device_limits.wasm` | `get-device` + **`[method]gpu-device.limits` sync** `own<gpu-supported-limits>` | `run: async func() -> u32` | construct device → limits (L2 described handle validate) → drop own; harness returns 1 |
| `webgpu_method_device_label.wasm` | `get-device` + **`[method]gpu-device.label` sync** `string` | `run: async func() -> u32` | construct device → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_device_set_label.wasm` | `get-device` + **`[method]gpu-device.set-label` sync** void | `run: async func() -> u32` | construct device → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_device_lost.wasm` | `get-device` + **`[method]gpu-device.lost` sync** `future<gpu-device-lost-info>` | `run: async func() -> u32` | construct device → lost (L2 described handle validate; future local) → future.read → drop own; harness returns 1 |
| `webgpu_method_device_on_uncaptured_error.wasm` | `get-device` + **`[method]gpu-device.on-uncaptured-error` sync** `stream<gpu-error>` | `run: async func() -> u32` | construct device → on-uncaptured-error (L2 described handle validate; stream local) → drop readable; harness returns 1 |
| `webgpu_method_device_push_error_scope.wasm` | `get-device` + **`[method]gpu-device.push-error-scope` sync** void | `run: async func() -> u32` | construct device → push-error-scope (validation; L2 described filter); harness returns 1 |
| `webgpu_method_device_pop_error_scope.wasm` | `get-device` + **`[method]gpu-device.pop-error-scope` async** `result<option<gpu-error>, pop-error-scope-error>` | `run: async func() -> u32` | construct device → pop-error-scope (L2 described handle; host ok/none); harness returns 1 |
| `webgpu_method_device_lost_info_message.wasm` | `get-device-lost-info` + **`[method]gpu-device-lost-info.message` sync** `string` | `run: async func() -> u32` | construct lost-info → message (L2 described device handle; Cpu stub `cpu-device-lost`); harness returns 1 |
| `webgpu_method_device_lost_info_reason.wasm` | `get-device-lost-info` + **`[method]gpu-device-lost-info.reason` sync** `gpu-device-lost-reason` | `run: async func() -> u32` | construct lost-info → reason (L2 described device handle; Cpu stub unknown); harness returns 1 |
| `webgpu_method_render_bundle_label.wasm` | `get-render-bundle` + **`[method]gpu-render-bundle.label` sync** `string` | `run: async func() -> u32` | construct render-bundle → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_render_bundle_set_label.wasm` | `get-render-bundle` + **`[method]gpu-render-bundle.set-label` sync** void | `run: async func() -> u32` | construct render-bundle → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_render_bundle_encoder_label.wasm` | `get-render-bundle-encoder` + **`[method]gpu-render-bundle-encoder.label` sync** `string` | `run: async func() -> u32` | construct render-bundle-encoder → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_render_bundle_encoder_set_label.wasm` | `get-render-bundle-encoder` + **`[method]gpu-render-bundle-encoder.set-label` sync** void | `run: async func() -> u32` | construct render-bundle-encoder → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_render_pass_label.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.label` sync** `string` | `run: async func() -> u32` | construct render-pass-encoder → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_render_pass_set_label.wasm` | `get-pass` + **`[method]gpu-render-pass-encoder.set-label` sync** void | `run: async func() -> u32` | construct render-pass-encoder → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_render_pipeline_label.wasm` | `get-render-pipeline` + **`[method]gpu-render-pipeline.label` sync** `string` | `run: async func() -> u32` | construct render-pipeline → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_render_pipeline_set_label.wasm` | `get-render-pipeline` + **`[method]gpu-render-pipeline.set-label` sync** void | `run: async func() -> u32` | construct render-pipeline → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_render_pipeline_get_bind_group_layout.wasm` | `get-render-pipeline` + **`[method]gpu-render-pipeline.get-bind-group-layout` sync** `own<gpu-bind-group-layout>` | `run: async func() -> u32` | construct render-pipeline → get-bind-group-layout (index 0; L2 described handle, 0 → stub) → drop own; harness returns 1 |
| `webgpu_method_record_gpu_pipeline_constant_value_add.wasm` | **`[constructor]record-gpu-pipeline-constant-value`** + **`[method]record-gpu-pipeline-constant-value.add` sync** void | `run: async func() -> u32` | construct record → add (L2 described handle + empty key + 0.0); harness returns 1 |
| `webgpu_method_record_gpu_pipeline_constant_value_get.wasm` | **`[constructor]record-gpu-pipeline-constant-value`** + **`[method]record-gpu-pipeline-constant-value.get` sync** `option<f64>` | `run: async func() -> u32` | construct record → get (L2 described handle + empty key; host none); harness returns 1 |
| `webgpu_method_record_gpu_pipeline_constant_value_has.wasm` | **`[constructor]record-gpu-pipeline-constant-value`** + **`[method]record-gpu-pipeline-constant-value.has` sync** `bool` | `run: async func() -> u32` | construct record → has (L2 described handle + empty key; host false); harness returns 1 |
| `webgpu_method_record_gpu_pipeline_constant_value_remove.wasm` | **`[constructor]record-gpu-pipeline-constant-value`** + **`[method]record-gpu-pipeline-constant-value.remove` sync** void | `run: async func() -> u32` | construct record → remove (L2 described handle + empty key); harness returns 1 |
| `webgpu_method_record_gpu_pipeline_constant_value_keys.wasm` | **`[constructor]record-gpu-pipeline-constant-value`** + **`[method]record-gpu-pipeline-constant-value.keys` sync** `list<string>` | `run: async func() -> u32` | construct record → keys (L2 described handle; host empty); harness returns 1 |
| `webgpu_method_record_gpu_pipeline_constant_value_values.wasm` | **`[constructor]record-gpu-pipeline-constant-value`** + **`[method]record-gpu-pipeline-constant-value.values` sync** `list<f64>` | `run: async func() -> u32` | construct record → values (L2 described handle; host empty); harness returns 1 |
| `webgpu_method_record_gpu_pipeline_constant_value_entries.wasm` | **`[constructor]record-gpu-pipeline-constant-value`** + **`[method]record-gpu-pipeline-constant-value.entries` sync** `list<tuple<string, f64>>` | `run: async func() -> u32` | construct record → entries (L2 described handle; host empty); harness returns 1 |
| `webgpu_method_record_option_gpu_size64_add.wasm` | **`[constructor]record-option-gpu-size64`** + **`[method]record-option-gpu-size64.add` sync** void | `run: async func() -> u32` | construct record → add (L2 described handle + empty key + none); harness returns 1 |
| `webgpu_method_record_option_gpu_size64_get.wasm` | **`[constructor]record-option-gpu-size64`** + **`[method]record-option-gpu-size64.get` sync** `option<option<u64>>` | `run: async func() -> u32` | construct record → get (L2 described handle + empty key; host none); harness returns 1 |
| `webgpu_method_record_option_gpu_size64_has.wasm` | **`[constructor]record-option-gpu-size64`** + **`[method]record-option-gpu-size64.has` sync** `bool` | `run: async func() -> u32` | construct record → has (L2 described handle + empty key; host false); harness returns 1 |
| `webgpu_method_record_option_gpu_size64_remove.wasm` | **`[constructor]record-option-gpu-size64`** + **`[method]record-option-gpu-size64.remove` sync** void | `run: async func() -> u32` | construct record → remove (L2 described handle + empty key); harness returns 1 |
| `webgpu_method_record_option_gpu_size64_keys.wasm` | **`[constructor]record-option-gpu-size64`** + **`[method]record-option-gpu-size64.keys` sync** `list<string>` | `run: async func() -> u32` | construct record → keys (L2 described handle; host empty); harness returns 1 |
| `webgpu_method_record_option_gpu_size64_values.wasm` | **`[constructor]record-option-gpu-size64`** + **`[method]record-option-gpu-size64.values` sync** `list<option<u64>>` | `run: async func() -> u32` | construct record → values (L2 described handle; host empty); harness returns 1 |
| `webgpu_method_record_option_gpu_size64_entries.wasm` | **`[constructor]record-option-gpu-size64`** + **`[method]record-option-gpu-size64.entries` sync** `list<tuple<string, option<u64>>>` | `run: async func() -> u32` | construct record → entries (L2 described handle; host empty); harness returns 1 |
| `webgpu_method_supported_limits_max_bind_groups.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-bind-groups` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxBindGroups` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_bind_groups_plus_vertex_buffers.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-bind-groups-plus-vertex-buffers` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxBindGroupsPlusVertexBuffers` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_bindings_per_bind_group.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-bindings-per-bind-group` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxBindingsPerBindGroup` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_buffer_size.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-buffer-size` sync** `u64` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxBufferSize` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_color_attachment_bytes_per_sample.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-color-attachment-bytes-per-sample` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxColorAttachmentBytesPerSample` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_color_attachments.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-color-attachments` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxColorAttachments` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_compute_invocations_per_workgroup.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-compute-invocations-per-workgroup` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxComputeInvocationsPerWorkgroup` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_compute_workgroup_size_x.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-compute-workgroup-size-x` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxComputeWorkgroupSizeX` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_compute_workgroup_size_y.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-compute-workgroup-size-y` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxComputeWorkgroupSizeY` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_compute_workgroup_size_z.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-compute-workgroup-size-z` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxComputeWorkgroupSizeZ` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_compute_workgroups_per_dimension.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-compute-workgroups-per-dimension` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxComputeWorkgroupsPerDimension` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_compute_workgroup_storage_size.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-compute-workgroup-storage-size` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxComputeWorkgroupStorageSize` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_dynamic_storage_buffers_per_pipeline_layout.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-dynamic-storage-buffers-per-pipeline-layout` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxDynamicStorageBuffersPerPipelineLayout` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_dynamic_uniform_buffers_per_pipeline_layout.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-dynamic-uniform-buffers-per-pipeline-layout` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxDynamicUniformBuffersPerPipelineLayout` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_immediate_size.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-immediate-size` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxImmediateSize` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_inter_stage_shader_variables.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-inter-stage-shader-variables` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxInterStageShaderVariables` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_sampled_textures_per_shader_stage.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-sampled-textures-per-shader-stage` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxSampledTexturesPerShaderStage` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_samplers_per_shader_stage.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-samplers-per-shader-stage` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxSamplersPerShaderStage` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_storage_buffer_binding_size.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-storage-buffer-binding-size` sync** `u64` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxStorageBufferBindingSize` (Cpu 1L); harness returns 1 |
| `webgpu_method_supported_limits_max_storage_buffers_in_fragment_stage.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-storage-buffers-in-fragment-stage` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxStorageBuffersInFragmentStage` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_storage_buffers_in_vertex_stage.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-storage-buffers-in-vertex-stage` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxStorageBuffersInVertexStage` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_storage_buffers_per_shader_stage.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-storage-buffers-per-shader-stage` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxStorageBuffersPerShaderStage` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_storage_textures_in_fragment_stage.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-storage-textures-in-fragment-stage` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxStorageTexturesInFragmentStage` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_storage_textures_in_vertex_stage.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-storage-textures-in-vertex-stage` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxStorageTexturesInVertexStage` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_storage_textures_per_shader_stage.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-storage-textures-per-shader-stage` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxStorageTexturesPerShaderStage` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_texture_array_layers.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-texture-array-layers` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxTextureArrayLayers` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_texture_dimension1_d.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-texture-dimension1-d` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxTextureDimension1D` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_texture_dimension2_d.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-texture-dimension2-d` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxTextureDimension2D` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_texture_dimension3_d.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-texture-dimension3-d` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxTextureDimension3D` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_features_has.wasm` | `get-adapter` + **`[method]gpu-adapter.features`** + **`[method]gpu-supported-features.has` sync** `bool` | `run: async func() -> u32` | L2: adapter rep + feature name → host `supportedFeaturesHas` (empty value; Cpu false); harness returns 1 |
| `webgpu_method_supported_limits_max_uniform_buffer_binding_size.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-uniform-buffer-binding-size` sync** `u64` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxUniformBufferBindingSize` (Cpu 1L); harness returns 1 |
| `webgpu_method_supported_limits_max_uniform_buffers_per_shader_stage.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-uniform-buffers-per-shader-stage` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxUniformBuffersPerShaderStage` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_vertex_attributes.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-vertex-attributes` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxVertexAttributes` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_vertex_buffer_array_stride.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-vertex-buffer-array-stride` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxVertexBufferArrayStride` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_max_vertex_buffers.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.max-vertex-buffers` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMaxVertexBuffers` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_min_storage_buffer_offset_alignment.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.min-storage-buffer-offset-alignment` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMinStorageBufferOffsetAlignment` (Cpu 1); harness returns 1 |
| `webgpu_method_supported_limits_min_uniform_buffer_offset_alignment.wasm` | `get-supported-limits` + **`[method]gpu-supported-limits.min-uniform-buffer-offset-alignment` sync** `u32` | `run: async func() -> u32` | L2: adapter/device reps → host `supportedLimitsMinUniformBufferOffsetAlignment` (Cpu 1); harness returns 1 |
| `webgpu_method_gpu_get_preferred_canvas_format.wasm` | `get-gpu` + **`[method]gpu.get-preferred-canvas-format` sync** `gpu-texture-format` | `run: async func() -> u32` | L2: host `gpuGetPreferredCanvasFormat` (Cpu rgba8unorm); harness returns 1 |
| `webgpu_method_gpu_wgsl_language_features.wasm` | `get-gpu` + **`[method]gpu.wgsl-language-features` sync** `own<wgsl-language-features>` | `run: async func() -> u32` | L2: host `gpuWgslLanguageFeatures` validate → drop own; harness returns 1 |
| `webgpu_method_wgsl_language_features_has.wasm` | `get-gpu` + **`[method]gpu.wgsl-language-features`** + **`[method]wgsl-language-features.has` sync** `bool` | `run: async func() -> u32` | L2: feature name → host `wgslLanguageFeaturesHas` (empty value; Cpu false); harness returns 1 |
| `webgpu_method_gpu_error_kind.wasm` | `get-gpu-error` + **`[method]gpu-error.kind` sync** `gpu-error-kind` | `run: async func() -> u32` | construct error → kind (L2 described device handle; Cpu stub validation-error); harness returns 1 |
| `webgpu_method_gpu_error_message.wasm` | `get-gpu-error` + **`[method]gpu-error.message` sync** `string` | `run: async func() -> u32` | construct error → message (L2 described device handle; Cpu stub `cpu-gpu-error`); harness returns 1 |
| `webgpu_method_uncaptured_error_event_error.wasm` | `get-uncaptured-error-event` + **`[method]gpu-uncaptured-error-event.error` sync** `own<gpu-error>` | `run: async func() -> u32` | construct event → error (L2 described device handle → own gpu-error with device rep) → drop own error; harness returns 1 |
| `webgpu_method_pipeline_layout_label.wasm` | `get-pipeline-layout` + **`[method]gpu-pipeline-layout.label` sync** `string` | `run: async func() -> u32` | construct pipeline-layout → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_pipeline_layout_set_label.wasm` | `get-pipeline-layout` + **`[method]gpu-pipeline-layout.set-label` sync** void | `run: async func() -> u32` | construct pipeline-layout → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_query_set_label.wasm` | `get-query-set` + **`[method]gpu-query-set.label` sync** `string` | `run: async func() -> u32` | construct query-set → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_query_set_set_label.wasm` | `get-query-set` + **`[method]gpu-query-set.set-label` sync** void | `run: async func() -> u32` | construct query-set → set-label (L2 described handle + guest empty string); harness returns 1 |
| `webgpu_method_queue_label.wasm` | `get-queue` + **`[method]gpu-queue.label` sync** `string` | `run: async func() -> u32` | construct queue → label (L2 described handle; Cpu stub empty); harness returns 1 |
| `webgpu_method_queue_on_submitted_work_done.wasm` | `get-queue` + **`[method]gpu-queue.on-submitted-work-done` async** void | `run: async func() -> u32` | construct queue → on-submitted-work-done (L2 described handle validate); harness returns 1 |
| `webgpu_method_queue_set_label.wasm` | `get-queue` + **`[method]gpu-queue.set-label` sync** void | `run: async func() -> u32` | construct queue → set-label (L2 described handle + guest empty string); harness returns 1 |

**Transitional:** host still registers flat names. **W2:** true CM async (`func_wrap_concurrent` + oneshot yield); pump via `run_concurrent` / `callRunConcurrent`. **W3:** `device-get-queue`, `device-create-command-encoder`, `command-encoder-finish`, `queue-submit1`, `command-encoder-begin-render-pass-clear`, and `render-pass-end` are **sync** on the same proposal instance (same L2 u32 as experimental; submit is single-buffer, not proposal `list`; begin-clear / end use stub view `23`, instrument substitutes Cpu offscreen TextureView). **W3 `[method]`:** `get-gpu` + `[method]gpu.request-adapter` (S2: `option<own<gpu-adapter>>`), `get-adapter` + `[method]gpu-adapter.request-device` (S3: `result<own<gpu-device>, request-device-error>`), and `get-device` + `[method]gpu-device.queue` (S1: `own<gpu-queue>`) and `[method]gpu-device.create-command-encoder` (S6: Guest descriptor=none → `own<gpu-command-encoder>`) and `[method]gpu-command-encoder.finish` (S7: Guest descriptor=none → `own<gpu-command-buffer>`) and `[method]gpu-device.create-buffer` (S4: Guest `gpu-buffer-descriptor` → `own<gpu-buffer>`) and `[method]gpu-queue.submit` (S5: Guest `list<borrow<gpu-command-buffer>>` → drop owns; harness 1) and `[method]gpu-device.create-texture` (S6+: Guest `gpu-texture-descriptor` → `own<gpu-texture>`) and `[method]gpu-device.create-sampler` (S8: Guest descriptor=none → `own<gpu-sampler>`) and `[method]gpu-device.create-shader-module` (S6+: Guest `gpu-shader-module-descriptor` → `own<gpu-shader-module>`; L2 host-fixed WGSL) and `[method]gpu-queue.write-buffer-with-copy` (S6+: Guest borrow buffer + empty list → `result<_, write-buffer-error>`; L2 host-fixed 4 bytes) and `get-texture` + `[method]gpu-texture.create-view` (S8: Guest descriptor=none → `own<gpu-texture-view>`) and `[method]gpu-device.create-bind-group-layout` (S6+: Guest `gpu-bind-group-layout-descriptor` → `own<gpu-bind-group-layout>`; L2 host-fixed empty entries) and `[method]gpu-device.create-pipeline-layout` (S6+: Guest `gpu-pipeline-layout-descriptor` → `own<gpu-pipeline-layout>`; L2 host-fixed empty bind-group-layouts) and `[method]gpu-device.create-bind-group` (S6+: Guest `gpu-bind-group-descriptor` → `own<gpu-bind-group>`; L2 host-fixed empty BGL + empty entries) and `[method]gpu-device.create-render-pipeline` (S6+: Guest `gpu-render-pipeline-descriptor` → `own<gpu-render-pipeline>`; L2 host-fixed stub shader + triangle) and `[method]gpu-device.create-render-pipeline-async` (S6+: same descriptor → `result<own<gpu-render-pipeline>, create-pipeline-error>`; true CM async) and `[method]gpu-device.create-compute-pipeline` (S6+: Guest `gpu-compute-pipeline-descriptor` → `own<gpu-compute-pipeline>`; L2 host-fixed stub shader + empty layout) and `[method]gpu-device.create-compute-pipeline-async` (S6+: same descriptor → `result<own<gpu-compute-pipeline>, create-pipeline-error>`; true CM async) and `[method]gpu-buffer.get-mapped-range-get-with-copy` (S6+: offset/size none → `result<list<u8>, get-mapped-range-error>`; L2 host-fixed empty list) and `[method]gpu-buffer.get-mapped-range-set-with-copy` (S6+: empty data + offset/size none → `result<_, get-mapped-range-error>`) and `[method]gpu-queue.write-texture-with-copy` (S6+: Guest texel copy info + empty list + size 1×1×1; L2 host-fixed 1×1) and `[method]gpu-command-encoder.begin-compute-pass` (S8: Guest descriptor=none → `own<gpu-compute-pass-encoder>`) and `get-compute-pass` + `[method]gpu-compute-pass-encoder.end` (S6+: void; harness 1) and `[method]gpu-command-encoder.copy-buffer-to-buffer` (S6+: Guest borrow src/dst + option offsets/size none → drop owns; harness 1; L2 host-fixed 4-byte copy) and `[method]gpu-command-encoder.copy-buffer-to-texture` / `copy-texture-to-buffer` / `copy-texture-to-texture` (S6+: texel-copy records + size 1×1×1; L2 host-fixed 4-byte buffer copy) and `[method]gpu-command-encoder.clear-buffer` (S6+: borrow buffer + offset/size none; L2 host-fixed 4-byte buffer copy) and `[method]gpu-command-encoder.resolve-query-set` (S6+: borrow query-set/buffer; L2 unused) and `[method]gpu-command-encoder.push-debug-group` / `pop-debug-group` / `insert-debug-marker` (S6+: empty string / void; L2 unused) and `[method]gpu-compute-pass-encoder.set-pipeline` (S6+: Guest `borrow<gpu-compute-pipeline>`; L2 host-fixed compute pipeline) and `[method]gpu-compute-pass-encoder.set-bind-group` (S6+: index + option bind-group + option offsets → `result<_, set-bind-group-error>`; L2 host-fixed empty bind-group) and `[method]gpu-compute-pass-encoder.dispatch-workgroups` (S6+: x + option y/z; L2 host-fixed 1×1×1 after set-pipeline + empty bind-group) and `[method]gpu-compute-pass-encoder.dispatch-workgroups-indirect` (S6+: borrow buffer + offset 0; L2 host-fixed 1×1×1) and `[method]gpu-compute-pass-encoder.set-immediates` (S6+: range 0 + empty list + offset/size none; L2 unused) and `[method]gpu-compute-pass-encoder.push-debug-group` / `pop-debug-group` / `insert-debug-marker` (S6+: empty string / void; L2 unused) and `[method]gpu-render-pass-encoder.set-pipeline` (S6+: Guest `borrow<gpu-render-pipeline>`; L2 host-fixed triangle pipeline) and `[method]gpu-render-pass-encoder.set-bind-group` (S6+: same result shape as compute; L2 host-fixed empty bind-group) and `[method]gpu-render-pass-encoder.set-vertex-buffer` (S6+: slot + option buffer + option offset/size; L2 host-fixed VERTEX slot 0) and `[method]gpu-render-pass-encoder.set-viewport` (S6+: six f32; L2 unused) and `[method]gpu-render-pass-encoder.set-scissor-rect` (S6+: four u32; L2 unused) and `[method]gpu-render-pass-encoder.set-blend-constant` (S6+: `gpu-color`; L2 unused) and `[method]gpu-render-pass-encoder.set-stencil-reference` (S6+: u32; L2 unused) and `[method]gpu-render-pass-encoder.set-index-buffer` (S6+: borrow buffer + `gpu-index-format` + option offset/size; L2 host-fixed VERTEX slot 0) and `[method]gpu-render-pass-encoder.draw` (S6+: vertex-count + three option<u32>; L2 host-fixed draw(3) after set-pipeline) and `[method]gpu-render-pass-encoder.draw-indexed` (S6+: index-count + options none; L2 host-fixed draw(3)) and `[method]gpu-render-pass-encoder.draw-indirect` / `draw-indexed-indirect` (S6+: borrow buffer + offset 0; L2 host-fixed draw(3)) and `[method]gpu-render-pass-encoder.push-debug-group` / `pop-debug-group` / `insert-debug-marker` (S6+: empty string / void; L2 unused) and `[method]gpu-render-pass-encoder.begin-occlusion-query` / `end-occlusion-query` (S6+: query-index 0 / void; L2 unused) and `[method]gpu-render-pass-encoder.execute-bundles` (S6+: one-element `list<borrow<gpu-render-bundle>>`; L2 unused) and `[method]gpu-render-pass-encoder.set-immediates` (S6+: range 0 + empty list + offset/size none; L2 unused) and `[method]gpu-render-bundle-encoder.finish` (S6+: descriptor=none → `own<gpu-render-bundle>`; L2 unused) and `[method]gpu-render-bundle-encoder.set-pipeline` (S6+: borrow pipeline; L2 unused) and `[method]gpu-render-bundle-encoder.set-bind-group` (S6+: same result shape as render-pass; L2 unused) and `[method]gpu-render-bundle-encoder.draw` (S6+: vertex-count=3 + options none; L2 unused) and `[method]gpu-render-bundle-encoder.set-index-buffer` (S6+: borrow buffer + `gpu-index-format` + option offset/size; L2 unused) and `[method]gpu-render-bundle-encoder.set-vertex-buffer` (S6+: slot + option buffer + option offset/size; L2 unused) and `[method]gpu-render-bundle-encoder.draw-indexed` (S6+: index-count=3 + options none; L2 unused) and `[method]gpu-render-bundle-encoder.draw-indirect` / `draw-indexed-indirect` (S6+: borrow buffer + offset 0; L2 unused) and `[method]gpu-render-bundle-encoder.push-debug-group` / `pop-debug-group` / `insert-debug-marker` (S6+: empty string / void; L2 unused) and `[method]gpu-render-bundle-encoder.set-immediates` (S6+: range 0 + empty list + offset/size none; L2 unused) and `[method]gpu-device.create-render-bundle-encoder` (S6+: Guest `gpu-render-bundle-encoder-descriptor` empty color-formats → `own<gpu-render-bundle-encoder>`; L2 unused) and `[method]gpu-device.create-query-set` (S6+: Guest `gpu-query-set-descriptor` → `result<own<gpu-query-set>, create-query-set-error>`; L2 unused) and `[method]gpu-device.destroy` / `[method]gpu-buffer.destroy` / `[method]gpu-texture.destroy` / `[method]gpu-query-set.destroy` (S6+: void; L2 unused) and `[method]gpu-query-set.type` (S6+: `gpu-query-type`; host-fixed occlusion) and `[method]gpu-query-set.count` (S6+: `u32`; host-fixed 1). Experimental flat sync path unchanged. Not full option/resource compliance.

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
wasm-tools parse fixtures/w1/webgpu_method_dawn_compute_slice.wat -o fixtures/w1/webgpu_method_dawn_compute_slice.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_dawn_compute_slice.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_buffer.wat -o fixtures/w1/webgpu_method_create_buffer.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_buffer.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_texture.wat -o fixtures/w1/webgpu_method_create_texture.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_texture.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_sampler.wat -o fixtures/w1/webgpu_method_create_sampler.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_sampler.wasm
wasm-tools parse fixtures/w1/webgpu_method_sampler_label.wat -o fixtures/w1/webgpu_method_sampler_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_sampler_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_sampler_set_label.wat -o fixtures/w1/webgpu_method_sampler_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_sampler_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_shader_module.wat -o fixtures/w1/webgpu_method_create_shader_module.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_shader_module.wasm
wasm-tools parse fixtures/w1/webgpu_method_shader_module_get_compilation_info.wat -o fixtures/w1/webgpu_method_shader_module_get_compilation_info.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_shader_module_get_compilation_info.wasm
wasm-tools parse fixtures/w1/webgpu_method_shader_module_label.wat -o fixtures/w1/webgpu_method_shader_module_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_shader_module_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_shader_module_set_label.wat -o fixtures/w1/webgpu_method_shader_module_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_shader_module_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_write_buffer.wat -o fixtures/w1/webgpu_method_write_buffer.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_write_buffer.wasm
wasm-tools parse fixtures/w1/webgpu_method_texture_create_view.wat -o fixtures/w1/webgpu_method_texture_create_view.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_texture_create_view.wasm
wasm-tools parse fixtures/w1/webgpu_method_texture_view_label.wat -o fixtures/w1/webgpu_method_texture_view_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_texture_view_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_texture_view_set_label.wat -o fixtures/w1/webgpu_method_texture_view_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_texture_view_set_label.wasm
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
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_encoder_set_index_buffer.wat -o fixtures/w1/webgpu_method_render_bundle_encoder_set_index_buffer.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_encoder_set_index_buffer.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_encoder_set_vertex_buffer.wat -o fixtures/w1/webgpu_method_render_bundle_encoder_set_vertex_buffer.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_encoder_set_vertex_buffer.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_encoder_draw_indexed.wat -o fixtures/w1/webgpu_method_render_bundle_encoder_draw_indexed.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_encoder_draw_indexed.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_encoder_draw_indirect.wat -o fixtures/w1/webgpu_method_render_bundle_encoder_draw_indirect.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_encoder_draw_indirect.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_encoder_draw_indexed_indirect.wat -o fixtures/w1/webgpu_method_render_bundle_encoder_draw_indexed_indirect.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_encoder_draw_indexed_indirect.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_encoder_push_debug_group.wat -o fixtures/w1/webgpu_method_render_bundle_encoder_push_debug_group.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_encoder_push_debug_group.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_encoder_pop_debug_group.wat -o fixtures/w1/webgpu_method_render_bundle_encoder_pop_debug_group.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_encoder_pop_debug_group.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_encoder_insert_debug_marker.wat -o fixtures/w1/webgpu_method_render_bundle_encoder_insert_debug_marker.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_encoder_insert_debug_marker.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_encoder_set_immediates.wat -o fixtures/w1/webgpu_method_render_bundle_encoder_set_immediates.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_encoder_set_immediates.wasm
wasm-tools parse fixtures/w1/webgpu_method_buffer_map_async.wat -o fixtures/w1/webgpu_method_buffer_map_async.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_buffer_map_async.wasm
wasm-tools parse fixtures/w1/webgpu_method_buffer_unmap.wat -o fixtures/w1/webgpu_method_buffer_unmap.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_buffer_unmap.wasm
wasm-tools parse fixtures/w1/webgpu_method_buffer_get_mapped_range.wat -o fixtures/w1/webgpu_method_buffer_get_mapped_range.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_buffer_get_mapped_range.wasm
wasm-tools parse fixtures/w1/webgpu_method_buffer_set_mapped_range.wat -o fixtures/w1/webgpu_method_buffer_set_mapped_range.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_buffer_set_mapped_range.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_render_bundle_encoder.wat -o fixtures/w1/webgpu_method_create_render_bundle_encoder.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_render_bundle_encoder.wasm
wasm-tools parse fixtures/w1/webgpu_method_create_query_set.wat -o fixtures/w1/webgpu_method_create_query_set.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_create_query_set.wasm
wasm-tools parse fixtures/w1/webgpu_method_device_destroy.wat -o fixtures/w1/webgpu_method_device_destroy.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_device_destroy.wasm
wasm-tools parse fixtures/w1/webgpu_method_buffer_destroy.wat -o fixtures/w1/webgpu_method_buffer_destroy.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_buffer_destroy.wasm
wasm-tools parse fixtures/w1/webgpu_method_texture_destroy.wat -o fixtures/w1/webgpu_method_texture_destroy.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_texture_destroy.wasm
wasm-tools parse fixtures/w1/webgpu_method_texture_width.wat -o fixtures/w1/webgpu_method_texture_width.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_texture_width.wasm
wasm-tools parse fixtures/w1/webgpu_method_texture_height.wat -o fixtures/w1/webgpu_method_texture_height.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_texture_height.wasm
wasm-tools parse fixtures/w1/webgpu_method_texture_depth_or_array_layers.wat -o fixtures/w1/webgpu_method_texture_depth_or_array_layers.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_texture_depth_or_array_layers.wasm
wasm-tools parse fixtures/w1/webgpu_method_texture_mip_level_count.wat -o fixtures/w1/webgpu_method_texture_mip_level_count.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_texture_mip_level_count.wasm
wasm-tools parse fixtures/w1/webgpu_method_texture_sample_count.wat -o fixtures/w1/webgpu_method_texture_sample_count.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_texture_sample_count.wasm
wasm-tools parse fixtures/w1/webgpu_method_texture_dimension.wat -o fixtures/w1/webgpu_method_texture_dimension.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_texture_dimension.wasm
wasm-tools parse fixtures/w1/webgpu_method_texture_format.wat -o fixtures/w1/webgpu_method_texture_format.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_texture_format.wasm
wasm-tools parse fixtures/w1/webgpu_method_texture_usage.wat -o fixtures/w1/webgpu_method_texture_usage.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_texture_usage.wasm
wasm-tools parse fixtures/w1/webgpu_method_texture_texture_binding_view_dimension.wat -o fixtures/w1/webgpu_method_texture_texture_binding_view_dimension.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_texture_texture_binding_view_dimension.wasm
wasm-tools parse fixtures/w1/webgpu_method_texture_label.wat -o fixtures/w1/webgpu_method_texture_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_texture_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_texture_set_label.wat -o fixtures/w1/webgpu_method_texture_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_texture_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_query_set_destroy.wat -o fixtures/w1/webgpu_method_query_set_destroy.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_query_set_destroy.wasm
wasm-tools parse fixtures/w1/webgpu_method_query_set_type.wat -o fixtures/w1/webgpu_method_query_set_type.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_query_set_type.wasm
wasm-tools parse fixtures/w1/webgpu_method_query_set_count.wat -o fixtures/w1/webgpu_method_query_set_count.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_query_set_count.wasm
wasm-tools parse fixtures/w1/webgpu_method_adapter_features.wat -o fixtures/w1/webgpu_method_adapter_features.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_adapter_features.wasm
wasm-tools parse fixtures/w1/webgpu_method_adapter_limits.wat -o fixtures/w1/webgpu_method_adapter_limits.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_adapter_limits.wasm
wasm-tools parse fixtures/w1/webgpu_method_adapter_info.wat -o fixtures/w1/webgpu_method_adapter_info.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_adapter_info.wasm
wasm-tools parse fixtures/w1/webgpu_method_adapter_info_vendor.wat -o fixtures/w1/webgpu_method_adapter_info_vendor.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_adapter_info_vendor.wasm
wasm-tools parse fixtures/w1/webgpu_method_adapter_info_architecture.wat -o fixtures/w1/webgpu_method_adapter_info_architecture.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_adapter_info_architecture.wasm
wasm-tools parse fixtures/w1/webgpu_method_adapter_info_device.wat -o fixtures/w1/webgpu_method_adapter_info_device.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_adapter_info_device.wasm
wasm-tools parse fixtures/w1/webgpu_method_adapter_info_description.wat -o fixtures/w1/webgpu_method_adapter_info_description.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_adapter_info_description.wasm
wasm-tools parse fixtures/w1/webgpu_method_adapter_info_subgroup_min_size.wat -o fixtures/w1/webgpu_method_adapter_info_subgroup_min_size.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_adapter_info_subgroup_min_size.wasm
wasm-tools parse fixtures/w1/webgpu_method_adapter_info_subgroup_max_size.wat -o fixtures/w1/webgpu_method_adapter_info_subgroup_max_size.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_adapter_info_subgroup_max_size.wasm
wasm-tools parse fixtures/w1/webgpu_method_adapter_info_is_fallback_adapter.wat -o fixtures/w1/webgpu_method_adapter_info_is_fallback_adapter.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_adapter_info_is_fallback_adapter.wasm
wasm-tools parse fixtures/w1/webgpu_method_bind_group_label.wat -o fixtures/w1/webgpu_method_bind_group_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_bind_group_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_bind_group_set_label.wat -o fixtures/w1/webgpu_method_bind_group_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_bind_group_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_bind_group_layout_label.wat -o fixtures/w1/webgpu_method_bind_group_layout_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_bind_group_layout_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_bind_group_layout_set_label.wat -o fixtures/w1/webgpu_method_bind_group_layout_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_bind_group_layout_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_buffer_label.wat -o fixtures/w1/webgpu_method_buffer_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_buffer_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_buffer_set_label.wat -o fixtures/w1/webgpu_method_buffer_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_buffer_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_buffer_size.wat -o fixtures/w1/webgpu_method_buffer_size.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_buffer_size.wasm
wasm-tools parse fixtures/w1/webgpu_method_buffer_usage.wat -o fixtures/w1/webgpu_method_buffer_usage.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_buffer_usage.wasm
wasm-tools parse fixtures/w1/webgpu_method_buffer_map_state.wat -o fixtures/w1/webgpu_method_buffer_map_state.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_buffer_map_state.wasm
wasm-tools parse fixtures/w1/webgpu_method_command_buffer_label.wat -o fixtures/w1/webgpu_method_command_buffer_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_command_buffer_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_command_buffer_set_label.wat -o fixtures/w1/webgpu_method_command_buffer_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_command_buffer_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_command_encoder_label.wat -o fixtures/w1/webgpu_method_command_encoder_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_command_encoder_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_command_encoder_set_label.wat -o fixtures/w1/webgpu_method_command_encoder_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_command_encoder_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_compilation_info_messages.wat -o fixtures/w1/webgpu_method_compilation_info_messages.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compilation_info_messages.wasm
wasm-tools parse fixtures/w1/webgpu_method_compilation_message_message.wat -o fixtures/w1/webgpu_method_compilation_message_message.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compilation_message_message.wasm
wasm-tools parse fixtures/w1/webgpu_method_compilation_message_type.wat -o fixtures/w1/webgpu_method_compilation_message_type.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compilation_message_type.wasm
wasm-tools parse fixtures/w1/webgpu_method_compilation_message_length.wat -o fixtures/w1/webgpu_method_compilation_message_length.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compilation_message_length.wasm
wasm-tools parse fixtures/w1/webgpu_method_compilation_message_line_num.wat -o fixtures/w1/webgpu_method_compilation_message_line_num.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compilation_message_line_num.wasm
wasm-tools parse fixtures/w1/webgpu_method_compilation_message_line_pos.wat -o fixtures/w1/webgpu_method_compilation_message_line_pos.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compilation_message_line_pos.wasm
wasm-tools parse fixtures/w1/webgpu_method_compilation_message_offset.wat -o fixtures/w1/webgpu_method_compilation_message_offset.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compilation_message_offset.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pass_label.wat -o fixtures/w1/webgpu_method_compute_pass_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pass_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pass_set_label.wat -o fixtures/w1/webgpu_method_compute_pass_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pass_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pipeline_label.wat -o fixtures/w1/webgpu_method_compute_pipeline_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pipeline_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pipeline_set_label.wat -o fixtures/w1/webgpu_method_compute_pipeline_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pipeline_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_compute_pipeline_get_bind_group_layout.wat -o fixtures/w1/webgpu_method_compute_pipeline_get_bind_group_layout.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_compute_pipeline_get_bind_group_layout.wasm
wasm-tools parse fixtures/w1/webgpu_method_device_adapter_info.wat -o fixtures/w1/webgpu_method_device_adapter_info.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_device_adapter_info.wasm
wasm-tools parse fixtures/w1/webgpu_method_device_features.wat -o fixtures/w1/webgpu_method_device_features.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_device_features.wasm
wasm-tools parse fixtures/w1/webgpu_method_device_limits.wat -o fixtures/w1/webgpu_method_device_limits.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_device_limits.wasm
wasm-tools parse fixtures/w1/webgpu_method_device_label.wat -o fixtures/w1/webgpu_method_device_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_device_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_device_set_label.wat -o fixtures/w1/webgpu_method_device_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_device_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_device_lost.wat -o fixtures/w1/webgpu_method_device_lost.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_device_lost.wasm
wasm-tools parse fixtures/w1/webgpu_method_device_on_uncaptured_error.wat -o fixtures/w1/webgpu_method_device_on_uncaptured_error.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_device_on_uncaptured_error.wasm
wasm-tools parse fixtures/w1/webgpu_method_device_push_error_scope.wat -o fixtures/w1/webgpu_method_device_push_error_scope.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_device_push_error_scope.wasm
wasm-tools parse fixtures/w1/webgpu_method_device_pop_error_scope.wat -o fixtures/w1/webgpu_method_device_pop_error_scope.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_device_pop_error_scope.wasm
wasm-tools parse fixtures/w1/webgpu_method_device_lost_info_message.wat -o fixtures/w1/webgpu_method_device_lost_info_message.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_device_lost_info_message.wasm
wasm-tools parse fixtures/w1/webgpu_method_device_lost_info_reason.wat -o fixtures/w1/webgpu_method_device_lost_info_reason.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_device_lost_info_reason.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_label.wat -o fixtures/w1/webgpu_method_render_bundle_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_set_label.wat -o fixtures/w1/webgpu_method_render_bundle_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_encoder_label.wat -o fixtures/w1/webgpu_method_render_bundle_encoder_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_encoder_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_bundle_encoder_set_label.wat -o fixtures/w1/webgpu_method_render_bundle_encoder_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_bundle_encoder_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_label.wat -o fixtures/w1/webgpu_method_render_pass_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pass_set_label.wat -o fixtures/w1/webgpu_method_render_pass_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pass_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pipeline_label.wat -o fixtures/w1/webgpu_method_render_pipeline_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pipeline_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pipeline_set_label.wat -o fixtures/w1/webgpu_method_render_pipeline_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pipeline_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_render_pipeline_get_bind_group_layout.wat -o fixtures/w1/webgpu_method_render_pipeline_get_bind_group_layout.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_render_pipeline_get_bind_group_layout.wasm
wasm-tools parse fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_add.wat -o fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_add.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_add.wasm
wasm-tools parse fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_get.wat -o fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_get.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_get.wasm
wasm-tools parse fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_has.wat -o fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_has.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_has.wasm
wasm-tools parse fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_remove.wat -o fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_remove.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_remove.wasm
wasm-tools parse fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_keys.wat -o fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_keys.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_keys.wasm
wasm-tools parse fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_values.wat -o fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_values.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_values.wasm
wasm-tools parse fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_entries.wat -o fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_entries.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_record_gpu_pipeline_constant_value_entries.wasm
wasm-tools parse fixtures/w1/webgpu_method_record_option_gpu_size64_add.wat -o fixtures/w1/webgpu_method_record_option_gpu_size64_add.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_record_option_gpu_size64_add.wasm
wasm-tools parse fixtures/w1/webgpu_method_record_option_gpu_size64_get.wat -o fixtures/w1/webgpu_method_record_option_gpu_size64_get.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_record_option_gpu_size64_get.wasm
wasm-tools parse fixtures/w1/webgpu_method_record_option_gpu_size64_has.wat -o fixtures/w1/webgpu_method_record_option_gpu_size64_has.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_record_option_gpu_size64_has.wasm
wasm-tools parse fixtures/w1/webgpu_method_record_option_gpu_size64_remove.wat -o fixtures/w1/webgpu_method_record_option_gpu_size64_remove.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_record_option_gpu_size64_remove.wasm
wasm-tools parse fixtures/w1/webgpu_method_record_option_gpu_size64_keys.wat -o fixtures/w1/webgpu_method_record_option_gpu_size64_keys.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_record_option_gpu_size64_keys.wasm
wasm-tools parse fixtures/w1/webgpu_method_record_option_gpu_size64_values.wat -o fixtures/w1/webgpu_method_record_option_gpu_size64_values.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_record_option_gpu_size64_values.wasm
wasm-tools parse fixtures/w1/webgpu_method_record_option_gpu_size64_entries.wat -o fixtures/w1/webgpu_method_record_option_gpu_size64_entries.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_record_option_gpu_size64_entries.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_bind_groups.wat -o fixtures/w1/webgpu_method_supported_limits_max_bind_groups.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_bind_groups.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_bind_groups_plus_vertex_buffers.wat -o fixtures/w1/webgpu_method_supported_limits_max_bind_groups_plus_vertex_buffers.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_bind_groups_plus_vertex_buffers.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_bindings_per_bind_group.wat -o fixtures/w1/webgpu_method_supported_limits_max_bindings_per_bind_group.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_bindings_per_bind_group.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_buffer_size.wat -o fixtures/w1/webgpu_method_supported_limits_max_buffer_size.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_buffer_size.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_color_attachment_bytes_per_sample.wat -o fixtures/w1/webgpu_method_supported_limits_max_color_attachment_bytes_per_sample.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_color_attachment_bytes_per_sample.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_color_attachments.wat -o fixtures/w1/webgpu_method_supported_limits_max_color_attachments.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_color_attachments.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_compute_invocations_per_workgroup.wat -o fixtures/w1/webgpu_method_supported_limits_max_compute_invocations_per_workgroup.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_compute_invocations_per_workgroup.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_compute_workgroup_size_x.wat -o fixtures/w1/webgpu_method_supported_limits_max_compute_workgroup_size_x.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_compute_workgroup_size_x.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_compute_workgroup_size_y.wat -o fixtures/w1/webgpu_method_supported_limits_max_compute_workgroup_size_y.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_compute_workgroup_size_y.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_compute_workgroup_size_z.wat -o fixtures/w1/webgpu_method_supported_limits_max_compute_workgroup_size_z.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_compute_workgroup_size_z.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_compute_workgroups_per_dimension.wat -o fixtures/w1/webgpu_method_supported_limits_max_compute_workgroups_per_dimension.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_compute_workgroups_per_dimension.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_compute_workgroup_storage_size.wat -o fixtures/w1/webgpu_method_supported_limits_max_compute_workgroup_storage_size.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_compute_workgroup_storage_size.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_dynamic_storage_buffers_per_pipeline_layout.wat -o fixtures/w1/webgpu_method_supported_limits_max_dynamic_storage_buffers_per_pipeline_layout.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_dynamic_storage_buffers_per_pipeline_layout.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_dynamic_uniform_buffers_per_pipeline_layout.wat -o fixtures/w1/webgpu_method_supported_limits_max_dynamic_uniform_buffers_per_pipeline_layout.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_dynamic_uniform_buffers_per_pipeline_layout.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_immediate_size.wat -o fixtures/w1/webgpu_method_supported_limits_max_immediate_size.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_immediate_size.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_inter_stage_shader_variables.wat -o fixtures/w1/webgpu_method_supported_limits_max_inter_stage_shader_variables.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_inter_stage_shader_variables.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_sampled_textures_per_shader_stage.wat -o fixtures/w1/webgpu_method_supported_limits_max_sampled_textures_per_shader_stage.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_sampled_textures_per_shader_stage.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_samplers_per_shader_stage.wat -o fixtures/w1/webgpu_method_supported_limits_max_samplers_per_shader_stage.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_samplers_per_shader_stage.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_storage_buffer_binding_size.wat -o fixtures/w1/webgpu_method_supported_limits_max_storage_buffer_binding_size.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_storage_buffer_binding_size.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_storage_buffers_in_fragment_stage.wat -o fixtures/w1/webgpu_method_supported_limits_max_storage_buffers_in_fragment_stage.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_storage_buffers_in_fragment_stage.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_storage_buffers_in_vertex_stage.wat -o fixtures/w1/webgpu_method_supported_limits_max_storage_buffers_in_vertex_stage.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_storage_buffers_in_vertex_stage.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_storage_buffers_per_shader_stage.wat -o fixtures/w1/webgpu_method_supported_limits_max_storage_buffers_per_shader_stage.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_storage_buffers_per_shader_stage.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_storage_textures_in_fragment_stage.wat -o fixtures/w1/webgpu_method_supported_limits_max_storage_textures_in_fragment_stage.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_storage_textures_in_fragment_stage.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_storage_textures_in_vertex_stage.wat -o fixtures/w1/webgpu_method_supported_limits_max_storage_textures_in_vertex_stage.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_storage_textures_in_vertex_stage.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_storage_textures_per_shader_stage.wat -o fixtures/w1/webgpu_method_supported_limits_max_storage_textures_per_shader_stage.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_storage_textures_per_shader_stage.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_texture_array_layers.wat -o fixtures/w1/webgpu_method_supported_limits_max_texture_array_layers.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_texture_array_layers.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_texture_dimension1_d.wat -o fixtures/w1/webgpu_method_supported_limits_max_texture_dimension1_d.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_texture_dimension1_d.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_texture_dimension2_d.wat -o fixtures/w1/webgpu_method_supported_limits_max_texture_dimension2_d.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_texture_dimension2_d.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_texture_dimension3_d.wat -o fixtures/w1/webgpu_method_supported_limits_max_texture_dimension3_d.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_texture_dimension3_d.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_features_has.wat -o fixtures/w1/webgpu_method_supported_features_has.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_features_has.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_uniform_buffer_binding_size.wat -o fixtures/w1/webgpu_method_supported_limits_max_uniform_buffer_binding_size.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_uniform_buffer_binding_size.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_uniform_buffers_per_shader_stage.wat -o fixtures/w1/webgpu_method_supported_limits_max_uniform_buffers_per_shader_stage.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_uniform_buffers_per_shader_stage.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_vertex_attributes.wat -o fixtures/w1/webgpu_method_supported_limits_max_vertex_attributes.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_vertex_attributes.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_vertex_buffer_array_stride.wat -o fixtures/w1/webgpu_method_supported_limits_max_vertex_buffer_array_stride.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_vertex_buffer_array_stride.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_max_vertex_buffers.wat -o fixtures/w1/webgpu_method_supported_limits_max_vertex_buffers.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_max_vertex_buffers.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_min_storage_buffer_offset_alignment.wat -o fixtures/w1/webgpu_method_supported_limits_min_storage_buffer_offset_alignment.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_min_storage_buffer_offset_alignment.wasm
wasm-tools parse fixtures/w1/webgpu_method_supported_limits_min_uniform_buffer_offset_alignment.wat -o fixtures/w1/webgpu_method_supported_limits_min_uniform_buffer_offset_alignment.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_supported_limits_min_uniform_buffer_offset_alignment.wasm
wasm-tools parse fixtures/w1/webgpu_method_gpu_get_preferred_canvas_format.wat -o fixtures/w1/webgpu_method_gpu_get_preferred_canvas_format.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_gpu_get_preferred_canvas_format.wasm
wasm-tools parse fixtures/w1/webgpu_method_gpu_wgsl_language_features.wat -o fixtures/w1/webgpu_method_gpu_wgsl_language_features.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_gpu_wgsl_language_features.wasm
wasm-tools parse fixtures/w1/webgpu_method_wgsl_language_features_has.wat -o fixtures/w1/webgpu_method_wgsl_language_features_has.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_wgsl_language_features_has.wasm
wasm-tools parse fixtures/w1/webgpu_method_gpu_error_kind.wat -o fixtures/w1/webgpu_method_gpu_error_kind.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_gpu_error_kind.wasm
wasm-tools parse fixtures/w1/webgpu_method_gpu_error_message.wat -o fixtures/w1/webgpu_method_gpu_error_message.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_gpu_error_message.wasm
wasm-tools parse fixtures/w1/webgpu_method_uncaptured_error_event_error.wat -o fixtures/w1/webgpu_method_uncaptured_error_event_error.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_uncaptured_error_event_error.wasm
wasm-tools parse fixtures/w1/webgpu_method_pipeline_layout_label.wat -o fixtures/w1/webgpu_method_pipeline_layout_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_pipeline_layout_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_pipeline_layout_set_label.wat -o fixtures/w1/webgpu_method_pipeline_layout_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_pipeline_layout_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_query_set_label.wat -o fixtures/w1/webgpu_method_query_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_query_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_query_set_set_label.wat -o fixtures/w1/webgpu_method_query_set_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_query_set_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_queue_label.wat -o fixtures/w1/webgpu_method_queue_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_queue_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_queue_on_submitted_work_done.wat -o fixtures/w1/webgpu_method_queue_on_submitted_work_done.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_queue_on_submitted_work_done.wasm
wasm-tools parse fixtures/w1/webgpu_method_queue_set_label.wat -o fixtures/w1/webgpu_method_queue_set_label.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_queue_set_label.wasm
wasm-tools parse fixtures/w1/webgpu_method_canvas_context_configure.wat -o fixtures/w1/webgpu_method_canvas_context_configure.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_canvas_context_configure.wasm
wasm-tools parse fixtures/w1/webgpu_method_canvas_context_unconfigure.wat -o fixtures/w1/webgpu_method_canvas_context_unconfigure.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_canvas_context_unconfigure.wasm
wasm-tools parse fixtures/w1/webgpu_method_canvas_context_get_configuration.wat -o fixtures/w1/webgpu_method_canvas_context_get_configuration.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_canvas_context_get_configuration.wasm
wasm-tools parse fixtures/w1/webgpu_method_canvas_context_get_current_texture.wat -o fixtures/w1/webgpu_method_canvas_context_get_current_texture.wasm
wasm-tools validate --features=cm-async,component-model fixtures/w1/webgpu_method_canvas_context_get_current_texture.wasm
```
