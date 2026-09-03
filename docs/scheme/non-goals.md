# Non-goals

**English** | [中文](non-goals.zh.md)

Hard boundaries until a new RFC ([`rfc.md`](rfc.md)).

| ID | Do not |
|----|--------|
| NG-2 | Depend on **wasmtime4j** as the runtime |
| NG-3 | Reimplement a full **Kotlin WebGPU client API** |
| NG-4 | Treat “all WASI 0.3 worlds” or “full wasi-testsuite P3” as the single KPI |
| NG-5 | Claim a **compliant wasi:webgpu product**, CTS pass, or WASI 1.0 distro |
| NG-6 | Publish **Maven Central** / GitHub Packages **outside** GitHub Environment `release`, or when secrets / arm64 `.so` are missing. `0.1.0` is pressed; later GAVs still go through that Environment |
| NG-7 | Implement a **second Dawn renderer**. Packaging / adapting **one** Dawn (C API default + `dawn-jni` leftover) is allowed. wgpu-native as default is a second renderer |
| NG-8 | Treat Latch / sync-compat as **true** CM async |
| NG-9 | Promote **wasi-gfx / multi-window** to a **P0** wasi:webgpu re-queue. Size/resize and pin input streams are product; multi-window is named-only |
| NG-11 | Replace “track upstream Wasmtime” with a non-official engine |
| NG-12 | Accept **host-fixed descriptor + transitional u32** as the DoD for **new** wasi:webgpu slices |

## Deferred (separate RFC)

| ID | Item |
|----|------|
| DG-1 | Panama desktop bindings |
| DG-2 | iOS / desktop as first-class |
| DG-3 | Full cloud/CLI WASI distro |
| DG-4 | Interpreter fallback (no Cranelift) |
| DG-6 | Multi-window / full desktop gfx (**non-urgent**) |

## Allowed

- Ratified WASI 0.3 **slices** that webgpu apps need
- Implementing the **wasi:webgpu proposal** (not a compliance claim)
- A **pluggable** GPU backend — default Dawn bundle; core without Dawn
