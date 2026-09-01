### Code — Dawn C surface configure aligns to D24 (caps.alphaModes[0]); present path ruled clean (2026-09-01)

Native Dawn host. Dig the last open lane — Dawn C `wgpuSurfacePresent` vs androidx
`GPUSurface.present` BufferQueue submission — by (1) dumping the real surface caps,
(2) aligning `surface_configure`, and (3) re-measuring SF over a long window.

`dawn_c::surface_caps_detail` copies the full `wgpuSurfaceGetCapabilities` lists
(formats / present / alpha) before `caps_free`, and `NativeGpuHost` caches
`dawn_surface_alpha_mode = alphaModes[0]` so `surface_configure` stops hard-coding
`AlphaMode_Auto(0)` and instead matches D24's `caps.alphaModes[0]`. One boot-time
`surface-caps` log records the evidence.

Real device (Vivo V2458A, Android 16, 120 Hz lock):

- `surface-caps formats=[22,23,40,30] present=[4,1] alpha=[4]` →
  RGBA8Unorm first, `present` = `Mailbox(4)` + `Fifo(1)`, `alpha` = `Inherit(4)`.
  Per `webgpu.h`, `AlphaMode_Auto` **defaults to `Inherit` on native**, so the old
  hard-coded `Auto(0)` already resolved to `Inherit(4)`: aligning to `caps[0]` is a
  no-op behaviourally, confirming configure is **not** the divergence.
- `PRESENT_FIFO = 0x0000_0001` is correct against the (updated) `WGPUPresentMode`
  (`Fifo=1`, `Mailbox=4`); we are not accidentally on Mailbox.
- Dawn pin `DAWN_COMMIT=9d41fdf…` is the androidx 1.0.0-alpha05 AAR
  `dawn_build_metadata.json` SHA — same Dawn commit, not a version skew.
- 65 s / 7835 frames SF `--timestats`: `droppedFrames=0`, `lateAcquireFrames=0`,
  `badDesiredPresentFrames=0`, all `jankyFrames=0`, `averageFPS=125.088`,
  `desired2present 5ms=7811`, `present2presentDelta 0ms=7740` / `25ms=0`,
  `present2present 8ms=7786`. **No rewind, no drop, no jank** on the Dawn C path.

Conclusion: the BufferQueue submission details (configure params, Dawn version,
SF timing) are **identical and clean** between the Dawn C host and the pure-androidx
D24 control. The only remaining structural diffs are (a) D24 calls
`onSubmittedWorkDone` (a real fence) every frame while NativeGpu's
`on-submitted-work-done` is mark+retire only, and (b) NativeGpu auto-presents inside
`queue.submit` (H8). The visual pop therefore does not originate in the present
submission path itself — it is the compositor/panel re-show (D13/D19) outside SF's
counters, or has already been drained by D3+D22.
