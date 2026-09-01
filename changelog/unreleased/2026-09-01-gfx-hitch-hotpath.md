### Named — cube hitch hot-path stage probe (issue 300)

- Restart the ~5 s Dawn C cube pop from the **code hot path**, not inherited Closed/Likely rows. Plan: `docs/mapping/gfx-hitch-native-dawn.md` §6.
- `NativeGpuHost` logs logcat `GfxHitch` `hotpath` every 120 presents and `hotpath-spike` when a stage crosses the threshold: `processEvents`, acquire, write-buffer, encode-gap (acquire→submit), `wgpuQueueSubmit`, stamp+`wgpuSurfacePresent`, mark+retire. No behavior change on the present path.
