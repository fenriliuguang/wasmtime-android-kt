---
name: webgpu-semantic-l2
description: >-
  Deepens one hung wasi:webgpu [method] from host-fixed or lift-only stub to
  described L2 (guest fields through JNI into WasiWebGpuHost). Use when the
  user says 语义加深, 真实编组, L2, host-fixed, described JNI, or follow
  docs/agent/webgpu-semantic-l2.md.
---

# WebGPU semantic L2

Read and follow [`docs/agent/webgpu-semantic-l2.md`](docs/agent/webgpu-semantic-l2.md) before exploring.

1. If the user listed one `[method]` name, that is the cut. Else run `.\scripts\webgpu-semantic-l2-remaining.ps1` and take the first **host-fixed** name that is not S1/S2/S3 (`queue` / `request-adapter` / `request-device`). Default: `gpu-device.create-sampler`.
2. Do not WebFetch WIT. Grep `third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`.
3. Do not read `cm.rs` / `jvm.rs` / callback/bridge files whole; Grep then windowed Read.
4. Copy the **create-buffer described** stack only. One `[method]` per PR. `HostArg` stays Int/Long.
5. Wire `ExperimentalHostCallbacks` + `ForwardingHostCallbacks` + existing attach. Guest fixture must pass a non-host-fixed scalar.
6. Tests: `cargo check --locked --lib` and filtered `wasi_webgpu_method` with `--test-threads=1`.
7. PR title `feat(webgpu): L2 <method> guest fields to host`, label `enhancement`.
