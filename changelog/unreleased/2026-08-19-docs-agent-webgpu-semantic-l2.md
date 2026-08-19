### Docs — agent playbook for wasi:webgpu semantic L2 (2026-08-19)

- Add `docs/agent/webgpu-semantic-l2.md`, Cursor rule/skill, and `.\scripts\webgpu-semantic-l2-remaining.ps1` (classify hung wraps: described vs host-fixed vs lift-only)
- Lane is one `[method]` per PR: guest scalars through `*_described` JNI into existing `WasiWebGpuHost`; gold stack is `create-buffer`
- Hub freeze unchanged; shape hangs stay on `webgpu-shape-slice.md`
