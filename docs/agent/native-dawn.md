# Agent playbook: native Dawn host (full pin)

**English** | [中文](native-dawn.zh.md)

P0 `wasi:webgpu` **shape** is **closed**. P1 WASI 0.3 official-shape is **closed**. `0.1.0` product gates are **empty**. Do **not** re-cut W1–W8, P1-FS1–FS4, P1-SK1–SK2, P1-HT1, G-dev, or wasi:webgpu G1–G9 / F1–F9 / guest-pipeline / WG-6 **as those queues**. This queue **replaces the JNI/androidx consume path** behind the same pin.

Living **auto** queue after `0.1.0`. Tracking needles: [`../scheme/native-dawn.md`](../scheme/native-dawn.md).

**Integration (this queue only):** one long-lived branch, **one lane = one commit**, **one PR when the queue is empty**. Do **not** open a PR per lane. Branch: **`cursor/native-dawn-rewrite-1355`**. Stay on it; do not fork a short `feat/` / `cursor/` cut per knife. Push commits to that branch. Open **one** PR to `main` after `ND-DEVICE` (remaining script empty). Keep the lane commits on that PR (do not squash them into a single commit). Exception to [`vcs-workflow.md`](../scheme/vcs-workflow.md) “no long-lived lines.”

P2 Wasmtime pin stays **named-only** ([`wasmtime-p2.md`](wasmtime-p2.md)).

Consume leftover is **empty** (`ND-DEVICE` landed; `#299` on `main`). Cube hitch is **not** a consume needle. Living leftover: [`gfx-hitch.md`](gfx-hitch.md) on **`fix/300-gfx-cube-pop`** — forget inherited Closed/Likely; restart from hot-path stages.

## Goal

Default product `wasi:webgpu@0.3.0-rc.2` consume is **in-process Dawn C** on the Wasmtime pump thread: guest WIT unchanged; Kotlin `Store` / `Linker` / `WebGpuBackend` unchanged as the **shell and BYO SPI**; hot-path methods do **not** bounce through `ExperimentalHostCallbacks` → androidx JNI.

**Full pin capability** is the product goal (all 224 resource `[method]` names reach Dawn C on the default backend, not merely instantiate). The out-of-tree rotating cube is **demo / on-screen evidence only** — never a consume-lane DoD, never a reason to skip `ND-REST`.

Not CTS (NG-5). Not a second Dawn **renderer** (NG-7): **one** Dawn binary, C API adapter. Not a Kotlin WebGPU client (NG-3). Not JS-style `start(callback)`.

## Why (conversation, 2026-08-31)

Device hitch after H12 is **millisecond phase jitter amplified by frequent ART crossings**, not “WIT is slow.” `wasi:webgpu` only requires a Component Model import (ns–μs if in-process). This repo’s extra tax is:

```text
guest → cm.rs lowering → 8MiB wasmtime-cm-pump
  → mpsc JNI bounce → ExperimentalHostCallbacks
  → DawnWasiWebGpuHost → androidx.webgpu JNI → Dawn
```

D24: pure androidx cube is smooth; Wasmtime/`host-dawn` is not. androidx is a Java façade over Dawn; present is `ANativeWindow` → SurfaceFlinger either way. Bypass **Kotlin on the GPU hot path**, not the Activity / `Store` API.

That ART-crossing Why motivated the **consume** rewrite (landed). For the remaining cube **eye pop**, do **not** treat it as a premise — hitch restart is [`gfx-hitch.md`](gfx-hitch.md).

**Equivalent rewrite** means Dawn **semantics** + hitch **invariants** + existing **tests**. Do **not** reimplement the 278 `exp_*` JNI table in Rust.

## Select the cut

If the user named a lane (`ND-DISP`, `native-dawn`, `下一刀`), keep **one** family. Otherwise:

```powershell
.\scripts\native-dawn-remaining.ps1
```

No `pwsh`: `python3 ./scripts/native-dawn-remaining.py` (same flags: `--all`).

Do the printed **Next:** line only — as **one commit** on `cursor/native-dawn-rewrite-1355`. Do **not** open a PR for that commit. `product-010-remaining` is empty; do not invent new `0.1.0` needles.

If consume leftover is empty, or the user named hitch / 抖动 / cube-pop / issue 300 / 真机, follow [`gfx-hitch.md`](gfx-hitch.md):

```powershell
.\scripts\gfx-hitch-remaining.ps1
```

## Hard bans

- Do **not** re-cut P0/P1 auto knives or change guest WIT names/args. Linker still always defines `wasi:webgpu`. Unwired → `request-adapter` **`none`**.
- Do **not** add a batched “draw frame” product import, JS rAF, or new `experimental:webgpu-cm` flats (NG-12).
- Do **not** grow `runtime-api` `WebGpuBackend` into 224 Kotlin GPU methods (NG-3).
- Do **not** ship **two** Dawn `.so` in the default APK (androidx bundled + self-built).
- Do **not** treat wgpu-native as the default adapter (second renderer).
- Do **not** add `wasmtime-wasi` without a size + Android thread note.
- Do **not** introduce wasmtime4j.
- Do **not** JS-style frame callbacks. Reuse [`threading-android.md`](../mapping/threading-android.md) + [`GfxOnFrameGate`](../../native/src/host.rs); do not rewrite gfx.
- Do **not** open a per-lane PR, draft PR, or “stack” PR. The only allowed PR is the **final** merge of `cursor/native-dawn-rewrite-1355` → `main` when remaining is empty.
- Do **not** edit hub files on a lane commit: root `README.md` / `README.zh.md`, `CHANGELOG.md`, `.github/workflows/ci.yml`, `CONTRIBUTING.md`. Exception: this playbook / gate-amendment **commit** on the long branch.
- Do **not** crate-`cargo fmt`. rustfmt **only** `.rs` this slice changed.
- Do **not** read `native/src/cm.rs` without an offset. Grep the symbol, then Read ~80 lines.
- Never file GitHub issues on Wasmtime, WASI, wasi-webgpu, Dawn, or androidx. No `gh issue create`.

Keep `func_wrap_concurrent` + yield for WIT `async func`. `rustc` **1.97.1**.

## Reuse first (do not rediscover)

| Asset | Use as |
|-------|--------|
| `native/src/cm.rs` pin `[method]` wraps (224) | Keep lowering; add dispatch only |
| `native/src/jvm.rs` `exp_*` (278) | **JniBackend** leftover / `dawn-jni` only |
| `fixtures/w1/*` + `native/tests/wasi_webgpu_method/` (238 files) | **Acceptance** for consume lanes |
| Existing `WasiWebGpu*` / `WasiGfx*` instruments | Device gates; do not replace with a cube-only test |
| `DawnWasiWebGpuHost.kt` + `gap-webgpu-wit-androidx.md` | **Mapping spec** (which Dawn call, which Record hole). Do not copy JNI packing |
| `gfx-hitch-checklist.md` C1–C2, H1, H8, keep-3 + fence, Fifo | **Invariants** on the C API. Do not copy C7 AAR leak batching |
| `host.rs` `GfxOnFrameGate`, `Store.postGfxVsync` | Vsync / stream; native surface **attaches** |
| `WebGpuBackend` + `WebGpuBackendHostAttach` | Kotlin BYO / discover; native default does not attach the 200-method table |
| `ResourceTable` / `GpuBuffer.rep` in `host.rs` | Pattern for native handle table |

## Lanes (auto)

Copy this table. Do not shrink a consume lane to “cube enough.”

| Commit | Needle in [`../scheme/native-dawn.md`](../scheme/native-dawn.md) | DoD |
|----|------------------------------------------------------------------|-----|
| **ND-RFC** | *(landed this playbook)* | Playbook + remaining script + skill/rule + gate amendments. `native-dawn-remaining.py` prints **`Next: ND-DISP`**. |
| **ND-DISP** | `gap: nd disp pending` | `cm.rs` webgpu imports dispatch `NativeGpu \| JniBackend`. Default remains JNI so all existing tests stay green. Native slot may be unset. Changelog. Reuse `exp_*` unchanged. Remove the needle. |
| **ND-SO** | `gap: nd so pending` | One Android Dawn **C API** `.so` (or equivalent `webgpu.h` export) built or fetched by a documented recipe. Changelog: **size + Android thread** (same bar as `wasmtime-wasi`). License in `THIRD_PARTY_NOTICES`. Do **not** git-add androidx `libwebgpu_c_bundled.so`. Do not enable as default. Remove the needle. |
| **ND-HOST** | `gap: nd host pending` | Rust `NativeGpu` trait + handle table. Kinds match `DawnWasiWebGpuHost` / `ResourceKind`. No product Kotlin GPU API. Unit smoke that a table insert/drop does not JNI. Remove the needle. |
| **ND-BOOT** | `gap: nd boot pending` | Native `gpu.request-adapter`, `adapter.request-device`, `device.queue`, plus adapter info/features/limits needed to boot. WIT `async` stays concurrent. **Reuse** `wasi_webgpu_request_*` / `device_get_queue` / method adapter-info tests **on NativeGpu**. JNI path still works. Remove the needle. |
| **ND-RES** | `gap: nd res pending` | Native create-buffer / texture / sampler / shader-module / texture-view; leftover descriptors per [`gap-webgpu-wit-androidx.md`](../mapping/gap-webgpu-wit-androidx.md) (Record holes stay Record if Dawn C has no slot). **Reuse** matching `wasi_webgpu_method` tests on NativeGpu. Remove the needle. |
| **ND-PIPE** | `gap: nd pipe pending` | Native bind-group-layout, pipeline-layout, bind-group, compute/render pipelines (including async ctors + constants). **Reuse** existing pipeline/layout/bind-group method tests on NativeGpu. Remove the needle. |
| **ND-ENC** | `gap: nd enc pending` | Native command-encoder, render/compute pass, draws, copies, debug, query-sets. **Reuse** encoder / begin-render-pass / render-pass-end / finish tests on NativeGpu. Remove the needle. |
| **ND-QUEUE** | `gap: nd queue pending` | Native `queue.submit`, `write-buffer-with-copy`, `write-texture-with-copy`, `on-submitted-work-done` via **C API** (not AAR JNI). Linear memory → Dawn is one copy. **Reuse** submit / write-buffer / write-texture / work-done tests on NativeGpu. Remove the needle. |
| **ND-REST** | `gap: nd rest pending` | **Sweep:** every remaining pin `[method]` that existing `wasi_webgpu_method/*` covers (labels, supported-limits getters, WGSL features, map-async, error scopes, lost, compilation-info, render bundles, canvas get-configuration, …). DoD: with NativeGpu selected, **`cargo test --locked --test wasi_webgpu_method` is green** (not a cube subset). JNI leftover may still implement the same names. Remove the needle. |
| **ND-SURF** | `gap: nd surf pending` | Native Dawn `Surface` from `ANativeWindow*` (`bindCanvasNativeWindow` / `Store` window handle). `gpu-canvas-context` configure / get-current-texture / present. Hitch invariants: no same-present `close()`, GPU fence + keep-3, no acquire-wait of previous fence, Fifo, intern one queue, submit/`present` idempotent (H8). **Reuse** canvas-context + frame-lifetime instruments. Cube is **not** this lane’s DoD. Remove the needle. |
| **ND-DEFAULT** | `gap: nd default pending` | Product `GpuBackends.dawn()` / `:android-webgpu` default is NativeGpu. androidx path is explicit `id = "dawn-jni"` (BYO). **One** Dawn `.so` in the default APK. `0.x` MINOR + changelog. Kotlin SPI still `setWebGpuBackend`. Remove the needle. |
| **ND-CLAIM** | `gap: nd claim pending` | [`claim-010.md`](../scheme/claim-010.md): default consume degree is **Dawn C**, not “instantiate via JNI.” Update [`gap-webgpu-wit-androidx.md`](../mapping/gap-webgpu-wit-androidx.md) (JNI leftover) + living native gap. Still **not** CTS. Changelog. Remove the needle. |
| **ND-DEVICE** | `gap: nd device pending` | Existing product instruments (`WasiGfxFrameLoopInstrumentedTest`, WG-6 guest compute/render/present, canvas lifetime) listed as running on **native default**. Out-of-tree **cube** is one **demo** on-screen row (not vendored; not a substitute for instruments). Cloud has **no** device: still **name** the instruments; do not fail the **final** PR solely for missing `connectedAndroidTest`. Remove the needle. Then open **one** PR `cursor/native-dawn-rewrite-1355` → `main`. |

## Named-only (never `Next:`)

| Lane | When | DoD |
|------|------|-----|
| Cube-only hot path | User named cube | Demo evidence only; does **not** close `ND-REST` |
| Cube hitch restart (issue 300) | User named hitch / 抖动 / 真机, or consume leftover empty | [`gfx-hitch.md`](gfx-hitch.md) — forget inherited Closed/Likely; hot-path stages. Not pin consume. |
| Present timestamp / skip-present (D3) | *(archive; hitch playbook owns follow-up)* | Landed 2026-09-01; do **not** recut as a consume lane |
| `AChoreographer` (no Java vsync JNI) | User named it | Optional; `postGfxVsync` stays legal |
| Core pin / `THREAD_PRIORITY_*` | User named it | Not a consume gate |
| P2 Wasmtime / `wasmtime-wasi` / CTS / 1.0.0 | User named those | Existing named queues |

## File whitelist (typical consume lane)

- `native/src/cm.rs` — this family’s `linker.instance` only (windowed)
- `native/src/` new or existing native-gpu module (not a second WIT)
- `native/tests/wasi_webgpu_*.rs` / `wasi_webgpu_method/` — **reuse**; add only if a pin method has no test
- `host-dawn/` — only `ND-DEFAULT` (factory id) or `dawn-jni` leftover
- `docs/scheme/native-dawn.md` — **remove this lane’s needle**
- `docs/mapping/gap-webgpu-native-dawn.md` — create on first consume (`ND-BOOT`); one row per leftover vs Dawn C
- `changelog/unreleased/<yyyy-mm-dd>-nd-<slug>.md`
- `ND-SO` may add build scripts + `THIRD_PARTY_NOTICES` / `blocked-gpu-host.md` pin

Do not add files under `docs/archive/`. Do not re-cut G1–G9 fixture names.

## Narrow tests

```powershell
cd native
cargo check --locked --lib
```

Plus the **existing** `cargo test --locked --test <this_family>` the lane flips to NativeGpu. Do not add a parallel fixture set.

Device (required where the table says Device / `ND-DEVICE`):

```powershell
.\gradlew.bat :smoke-app:connectedDebugAndroidTest -Pandroid.testInstrumentationRunnerArguments.class=<instrument>
```

Cloud has **no** device. Still **add or list** the instrument; do not fail the **final** PR solely because this checkout could not run it.

This playbook amendment (docs-only, on the long branch): `python3 ./scripts/native-dawn-remaining.py` must print **`Next: ND-DISP`** and name branch `cursor/native-dawn-rewrite-1355`. `python3 ./scripts/product-010-remaining.py` must print the empty `0.1.0` queue and point here.

## Acceptance gates (product)

| Gate | Pass | Fail |
|------|------|------|
| Guest shape | Pin `[method]` names, `own`/`borrow`, `async` wraps ([`guest-shape.md`](../scheme/guest-shape.md)) | New flats, host-fixed `u32`, JS callback |
| Full consume | Default backend: `wasi_webgpu_method` suite green on NativeGpu (`ND-REST`) | “Cube presents” as consume DoD |
| Unwired | `request-adapter` → `none` | Trap / missing import |
| BYO | `setWebGpuBackend` + `dawn-jni` still attaches JNI table | Delete JNI path before `ND-DEFAULT` without a leftover id |
| Present | `wasi-gfx` pull `on-frame`; hitch invariants on native surface | Rewrite gfx; Mailbox as default |
| Demo | Cube / README out-of-tree link is **evidence** (`ND-DEVICE`) | Treating cube as the product subset |
| Claim | `ND-CLAIM` upgrades degree to Dawn C; not CTS | Silent drop of pin methods |
| Binary | One Dawn `.so`; size + thread note | androidx + self-built both in default APK |

## Commit message (one per lane)

- Workflow / playbook on this branch: `docs: native-dawn long-lived rewrite branch`
- DISP / HOST: `refactor(webgpu): ND …`
- SO: `build(webgpu): ND Dawn C Android so`
- Consume: `feat(webgpu): ND …`
- DEFAULT: `feat(api): ND default native Dawn`
- CLAIM: `docs: ND native Dawn claim table`
- DEVICE: `test(webgpu): ND instruments on native default`

Final PR (only when remaining is empty): title `feat(webgpu): native Dawn C host (full pin)`; label `enhancement`. Keep lane commits.

## Copy source

Pin WIT, [`guest-shape.md`](../scheme/guest-shape.md), [`DawnWasiWebGpuHost.kt`](../../host-dawn/src/main/kotlin/io/github/fenriliuguang/wasi/webgpu/experimental/dawn/DawnWasiWebGpuHost.kt) as mapping, hitch checklist invariants, gfx RFC (loop shape unchanged). Do not invent a second guest ABI.
