### Fix — Dawn `onSubmittedWorkDone` JNI global-ref leak crash (2026-08-27)

- `GPUQueue.onSubmittedWorkDone` (androidx.webgpu 1.0.0-alpha05) leaks a JNI global ref for **both** `callback` and `executor` on every call (never `DeleteGlobalRef`): 2 refs/frame → `global reference table overflow (max=51200)` `SIGABRT` on GpuThread ~3.5 min (25,320 frames)
- Fence **2** frames per `onSubmittedWorkDone` (batch) instead of every frame, halving the leak rate. Ring peak stays at keep+1 = 4 < the 5-deep BLAST pool (no acquire block). Verified on V2458A: overflow moved 25,320 → ~50,640 frames (3.5 → 7.0 min @120 Hz)
- Root fix is upstream / a self-hosted wgpu FFI — this AAR’s `libwebgpu_c_bundled.so` exports only `Java_androidx_webgpu_*`, no `wgpuQueueOnSubmittedWorkDone`

- Checklist: [`docs/mapping/gfx-hitch-checklist.md`](../../docs/mapping/gfx-hitch-checklist.md) C7
