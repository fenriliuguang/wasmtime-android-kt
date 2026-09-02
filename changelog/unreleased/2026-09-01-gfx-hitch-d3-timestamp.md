### Named — cube hitch D3 present timestamp (2026-09-01)

- `ANativeWindow_setBuffersTimestamp` immediately before `wgpuSurfacePresent` (default 2 Choreographer beats; `debug.wasmtime.gfx.desired_present_beats` / `WASMTIME_GFX_DESIRED_PRESENT_BEATS`, `0` off). V2458A `rc=0`. ~59 s 1:1 with D22 120 Hz lock: `desired2present` 29 ms → **5 ms=7072** at FPS 125.062 (skip `n=6` was 104). Not a consume lane.
