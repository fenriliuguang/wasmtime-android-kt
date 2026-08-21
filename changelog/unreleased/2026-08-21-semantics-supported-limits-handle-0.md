### Code — leftover descriptor semantics supported-limits handle-0 (2026-08-21)

- `gpu-supported-limits` getters no longer construct `GpuHandle(0)` when the device rep is absent; JNI `0` maps to `device: GpuHandle? = null` (adapter-only query)
- Live adapter/device reps still go through `Handles`; `get-supported-limits` / `adapter.limits` keep requesting a live adapter when both reps are 0
- Do not re-cut the limits first-cut JNI signatures (`(II)I` / `(II)J`)
