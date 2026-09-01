### Code — GfxHitch P4 probe: guest transform is time-linear; D20 SF-throttle closed (2026-09-01)

Native Dawn host. `GfxHitchLog` gains a **P4 angle-clock linearity histogram** in
`note_consumed_vsync` — the per-frame delta of the Choreographer `frameTimeNanos`
is exactly the guest's `angle` step (`rad_per_sec * (now_ns - last_ns)`), because
`monotonic-clock#now` returns `in_frame_instant_ns` = the vsync-instant accumulator.

Real device (Vivo V2458A, Android 16, 120 Hz lock, 95 s / 12720 frames):

- `angleDt <8ms=2`, `8-9ms=全部`, `9-17ms=0`, `>17ms=0` — the transform matrix
  received back from the guest is **strictly linear in time**: every frame steps
  exactly one 8.33 ms beat, no skip, no rewind.
- No `angle-dt-jump` events fired (dt never exceeded 9 ms).

Conclusion: guest content (cube `angle`) is not stuttering or rewinding. The
~5 s visual pop is the compositor re-showing an old BLAST buffer (D13/D19), not
guest-side. Corroborates P2 (2 `phase-crossing`/95 s, <1.5 ms) and P3
(`retire<8.3ms=0`).

D20 (SF "hold-until-phase" + UID FPS divisor) is **Closed**: `dumpsys
SurfaceFlinger` shows `GameFrameRateOverrides=` empty, `setFrameRate UID=10504 →
120.00 Hz`, BLAST layer `requestedFrameRate 120.00 Hz ExactOrMultiple`,
`idleScreenConfig timeout:-1`, and a stable 120 fps `FPS ring buffer`. SF does not
throttle this UID to a divisor of 120.

Next is no longer guest/runtime/CM-pump or SF-throttle: the remaining rewind is
the Dawn C present path's BLAST re-show, specific to it (D24 pure androidx has no
pop).
