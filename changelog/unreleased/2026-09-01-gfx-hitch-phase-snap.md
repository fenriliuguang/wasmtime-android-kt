### Code — present phase can snap out of lock, but guest content stays decoupled (2026-09-01)

Re-eye-check confirmed the pop **still shows**. Re-reading the P2 probe's full
`phase-crossing` sequence (not the short 95 s window that reported "clean")
revealed a real, previously-unseen signature: present phase can perform a sudden,
persistent snap out of lock.

Real device (Vivo V2458A, Android 16, 120 Hz lock, Dawn C host):

- **Slow-drift (dominant regime):** `phase-crossing` fires every 30–145 s
  (`sinceLast` stays tens of seconds), `margin` bounces near 0, `cross` crawls.
  Harmless. This is what the P2 "clean" window sampled.
- **Snap (rare, observed once):** at 23:44:03.5 `lat` jumped 5 → 9.6 ms and
  `margin` flipped +3 → −1.3 ms in one beat; afterwards `cross` accumulated
  several/second with no recovery.

Ruled out as the snap trigger: refresh rate stayed **120 Hz** (no mode switch);
thermal normal (CPU/GPU 40 °C vs 95 °C throttle); no `NativeGpu`/Dawn error at
the instant.

**Correction — the snap is NOT the pop, or at least not via the guest.** The P5
`take-skip` probe (`host.rs note_take_vsync`, logs `dt>12ms`) fired **0 times**
across ~12 min including phase crossings: the guest's `now`/`angle` stays strictly
linear per 8.33 ms beat even when present phase unlocks. So a present-phase snap
does **not** propagate into guest content — the cube does not fast-forward/rewind
from this. Combined with SF `--timestats` always clean (`droppedFrames=0`,
`jankyFrames=0`, `present2presentDelta 25ms=0`), all three observable layers
(guest content, SF counters, present submission) remain clean.

Net: the snap is real but rare (~30 min between the one observation and any
recurrence) and does not explain the pop through any measured channel. The pop is
still most consistent with an un-measured compositor/panel re-show (D13/D19), or
the snap's host-side present transient — neither yet captured at the exact pop
instant. Next needs event-triggered capture (screenrecord keyed on `sinceLast`
collapse) to bind the visible pop to a specific frame, or the real-fence lane
(`onSubmittedWorkDone`) to close the last D24 structural gap.
