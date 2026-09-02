### Named — cube hitch CM-pump probe (2026-09-01)

- P2/P3 instrumentation in `native_gpu.rs` `GfxHitchLog`: per-present `margin` to the next vsync boundary (logs `phase-crossing` on sign flip) + present→retire wall-age histogram (`retire<8.3ms` / `8.3-25ms` / `>25ms`). No behavior change; two new fields on `NativeCanvasFrame` (`presented_mono_ns`).
- V2458A 120 Hz lock, `fullscreen-surface` NativeGpu, **95 s / 12000 frames**:
  - Choreographer `>20ms=0` (vsync source perfect); acquire `>20ms=0`; present interval `>20ms=0`.
  - vsync→present latency `>8.3ms=3` / 12000 (0.025%); `phase-crossing` **2** (margin `-0.4ms`, `-2.0ms`), **55.5 s apart** — an ultra-slow beat drift (~1.25 ns/beat), **not** a ~5 s period.
  - retire age `<8.3ms=0` across all 12000 frames; retire lands 24–28 ms (~3 beats, keep-3). No buffer is recycled before SurfaceFlinger composites it.
- **Verdict:** the CM pump / `GfxOnFrameGate` phase lock and the host-side present/buffer lifecycle are clean. The hitch is not on the out-frame path (host present timing, phase, or recycle). It stays on the SurfaceFlinger BLAST re-show side (D13 / N5). This closes the "CM pump / Wasmtime runtime / guest impl" line for this hitch.
