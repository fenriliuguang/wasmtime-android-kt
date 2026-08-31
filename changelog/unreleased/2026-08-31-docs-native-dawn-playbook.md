# docs: native-dawn host playbook (full pin)

- Living auto queue after empty `0.1.0` gates: default `wasi:webgpu` consume moves to in-process Dawn C (same pin; Kotlin `Store`/`WebGpuBackend` stay the shell / BYO).
- Playbook [`docs/agent/native-dawn.md`](../../docs/agent/native-dawn.md); needles [`docs/scheme/native-dawn.md`](../../docs/scheme/native-dawn.md); `scripts/native-dawn-remaining.py`. First **Next:** `ND-DISP`.
- Acceptance: full `wasi_webgpu_method` suite on NativeGpu (`ND-REST`); cube / out-of-tree demo is **ND-DEVICE** evidence only. Reuse existing lowering, fixtures, hitch invariants; do not re-cut G1–G9.
