---
name: webgpu-guest-dawn
description: >-
  After F1–F9: copy already-snapshotted Kotlin descriptor fields into
  androidx.webgpu (blend/cull/MSAA, texture view-formats, shader hints,
  xr-compatible, default-queue), marshall write-mask / depth-stencil leftovers /
  canvas configuration extras, accept pipeline layout auto, then named WG-6
  guest-drawn compute/3D/present. Use when the user says 下一刀, Dawn consume,
  WG-6, blend on GPU, view-formats Dawn, xr-compatible, default-queue,
  write-mask, stencil, canvas configuration, auto layout, 真 compute, 真 3D,
  上屏 guest, or follow docs/agent/webgpu-guest-dawn.md.
---

# WebGPU Dawn consume + WG-6 leftovers

Read and follow [`docs/agent/webgpu-guest-dawn.md`](docs/agent/webgpu-guest-dawn.md) before exploring.

1. Run `.\scripts\webgpu-guest-dawn-remaining.ps1` (or `python3 ./scripts/webgpu-guest-dawn-remaining.py`) unless the user named one lane. Do **only** the printed **Next:** PR.
2. Order: G1 Dawn render-pipeline extras → G2 texture view-formats → G3 shader hints → G4 xr-compatible → G5 default-queue → G6 write-mask → G7 depth-stencil leftovers → G8 canvas configuration leftovers → G9 auto pipeline layout. WG-6 real compute / 3D / guest-drawn present and stage-only limits only if the user named them.
3. One lane per PR. Do not re-hang `[method]` names. Do not re-cut P1–P5, F1–F9, labels/limits/sampler/canvas first-cuts, the constants map resource, S1–S3 JNI, or Dawn cite slices. Never file GitHub issues on wasi-webgpu, Wasmtime, or any other upstream.
4. Do not WebFetch WIT. Grep `third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`. Windowed Read of `cm.rs` / `jvm.rs` (~80 lines). Copy the playbook stack. Forward into existing `WasiWebGpuHost` records — extend them if a field is missing. No new `HostArg` variants.
5. No product `surface-*`. No wasi-gfx. No CTS claim. `request-adapter` with no backend → guest `none`. Keep true async.
6. Tests: `cargo check --locked --lib` + filtered `wasi_webgpu_method -- --test-threads=1`. Device instruments only for named WG-6 PRs.
7. PR title from the playbook; label `enhancement`.
