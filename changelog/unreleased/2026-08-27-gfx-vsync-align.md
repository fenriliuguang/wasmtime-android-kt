### Fix — vsync / swapchain beat alignment (2026-08-27)

- Waiting GpuThread on the **current** `GPUQueue.onSubmittedWorkDone` stacked on Choreographer vsync: beats dropped, cube `angle += const` spun fast on 120 Hz then hitched. `queue.submit` registers the fence asynchronously; the **next** `get-current-texture` waits that fence before acquire while pumping `processEvents`. Recycle still requires GPU done + 3 newer frames.
- Pin `frame-event` is `{ nothing: bool }` (not rAF). Do **not** cap `on-frame` at 60 Hz. Guest motion delta is `wasi:clocks/monotonic-clock#now`. Beats that arrive while guest is still in a frame are dropped so the next read waits a fresh vsync. Hitch still open: [`docs/mapping/gfx-hitch-checklist.md`](../../docs/mapping/gfx-hitch-checklist.md). Cloud has no device
