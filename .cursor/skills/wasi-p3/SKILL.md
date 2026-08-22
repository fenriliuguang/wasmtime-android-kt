---
name: wasi-p3
description: >-
  After P0 wasi:webgpu close-out: finish ratified WASI 0.3 on Android
  (official WIT, not transitional u64/future<u32> smokes) with a device
  instrument per lane. Use when the user says 下一刀, P1, WASI 0.3, wasi:clocks
  instant, timezone, stream multi-chunk, cli result, stdin tuple, command run,
  filesystem, sockets, http, follow docs/agent/wasi-p3.md, or run
  wasi-p3-remaining.
---

# WASI 0.3 (P1)

Read and follow [`docs/agent/wasi-p3.md`](docs/agent/wasi-p3.md) before exploring.

1. Run `.\scripts\wasi-p3-remaining.ps1` (or `python3 ./scripts/wasi-p3-remaining.py`) unless the user named one lane. Do **only** the printed **Next:** PR.
2. Order: W1 stream multi-chunk → W2 clocks official instant → W3 cli stdout/stderr result → W4 stdin tuple → W5 command official run → W6 filesystem sandbox → W7 sockets → W8 http.
3. One lane per PR. Do **not** re-cut wasi:webgpu G1–G9 / F1–F9 / guest-pipeline / WG-6. Never file GitHub issues on Wasmtime, WASI, or any other upstream.
4. Do not add `wasmtime-wasi` without a size + Android thread note in that PR’s changelog. Windowed Read of `cm.rs` (~80 lines after Grep). Copy the existing package instance + fixture + instrument.
5. Every lane adds or extends a `smoke-app` instrument. Keep true CM async. No `wasi:io@0.2` pollable path.
6. Tests: `cargo check --locked --lib` + filtered `wasi_<module>`. Hub freeze: no root README / `CHANGELOG.md` / `ci.yml` / `CONTRIBUTING.md`.
7. PR title from the playbook; label `enhancement`.
