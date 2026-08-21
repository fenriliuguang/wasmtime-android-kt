### Code — leftover descriptor semantics Dawn consume pipeline constants (2026-08-21)

- Dawn `GPUComputeState` / `GPUVertexState` / `GPUFragmentState` now take snapshotted `descriptor.*.constants` as `GPUConstantEntry[]`
- Do not re-cut `record-gpu-pipeline-constant-value`; JNI + Kotlin records already hold the keyed f64 map
- androidx `1.0.0-alpha05` exposes the constants ctor slot (empty map → `arrayOf()`)
