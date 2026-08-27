### Fix — canvas hitch: no GPU wait on acquire, poller retire, writeBuffer lock (2026-08-27)

- `get-current-texture` no longer waits the previous canvas GPU fence (that delayed present vs scanout). Swapchain ring + `onSubmittedWorkDone` stay; `tryDrop` runs on the event poller, not on vsync→present
- `GPUSurface.getCurrentTexture` does not hold `gpuLock` (poller can `processEvents` if BLAST blocks). Logcat `GfxHitch`: acquire ns, status, 120-frame interval histogram
- `queueWriteBuffer` holds `gpuLock` and reuses an **exact-capacity** direct `ByteBuffer` per size (Dawn uses capacity, not remaining; no per-frame `slice()`). Fifo + `ANativeWindow_setBufferCount(4)` before Dawn configure (H9; 3 was EINVAL on BLAST). Intern one device queue handle
- JNI `HostArg::Bytes` reuses an exact-size `jbyteArray` (H16). Whole-buffer `write-buffer` does not extra-copy. Slow acquire logs ART `gc-count` / `gc-time`
- 120 Hz `on-frame`: consume **every** Choreographer beat (H2/H27). Latch `last_take_gen` to the current generation so a stall is one present, not a burst. Do **not** wait an extra beat after wait-start (that forced 16 ms acquire / 60 fps on 120 Hz Fifo)
- Examples: peak display mode + `setFrameRate` FIXED_SOURCE + `CHANGE_FRAME_RATE_ONLY_IF_SEAMLESS` + `setPreferMinimalPostProcessing` + `android:isGame`. Do **not** stack `setFrameRatePowerSavingsBalanced(false)` / GameState / layer `SurfaceControl.setFrameRate` (those raised hitch to ~3 s vs RMS)

- Checklist: [`docs/mapping/gfx-hitch-checklist.md`](../../docs/mapping/gfx-hitch-checklist.md) §6 far causes (wiring / guest / Wasmtime / compositor). V2458A: overall smooth after H12; leftover **~5 s** with clean 8.3 ms acquire. No `wasmtime-wasi`
