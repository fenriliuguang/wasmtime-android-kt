### Named — cube hitch synthetic 120 Hz beat check (issue 300)

- Cloud cannot simulate Mali / BLAST / SurfaceFlinger. `native_gpu` unit test `hotpath_synthetic_120hz_beats_are_1_to_1` drives `GfxOnFrameGate` + `NativeGpuHost` with 12 synthetic 8.333 ms timestamps and asserts 1:1 take/present, H8 no-op, keep-3, desired-present cadence, `vsync_dt` 8–9 ms, and a skipped-beat jump. Hitch §6.5 records the Cloud vs device split.
