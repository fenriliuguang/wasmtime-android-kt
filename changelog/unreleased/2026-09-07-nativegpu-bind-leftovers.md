### Fix — NativeGpu BIND leftovers: map/readback and table stubs (#317) (2026-09-07)

- Map / readback: `get-mapped-range-get-with-copy` copies `wgpuBufferGetConstMappedRange`; `set-with-copy` writes `wgpuBufferGetMappedRange`. Guest offset/size honored. Table-backed path keeps a CPU shadow of `write-buffer` (no 4096 zero-fill).
- Wire idle Dawn C wrappers: render-bundle encoder recording, occlusion queries, adapter `GetInfo`, buffer size/usage/map-state, texture getters, pipeline `get-bind-group-layout` index, bundle color format.
- Gap / claim: remaining Table leftovers listed (limits=`1`, compilation-info, pop-error-scope payload, uncaptured/lost, most labels). Not silent.
