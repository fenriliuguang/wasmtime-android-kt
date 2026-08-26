# Agent playbook: WASI 0.3 (P1)

**English** | [中文](wasi-p3.zh.md)

P0 `wasi:webgpu` is **closed**. Close-out: [`../archive/p0-wasi-webgpu.md`](../archive/p0-wasi-webgpu.md). WebGPU holes: [`../mapping/gap-webgpu-wit-androidx.md`](../mapping/gap-webgpu-wit-androidx.md). Do **not** re-cut G1–G9, F1–F9, P1–P5 guest-pipeline, or WG-6.

This queue finishes **ratified WASI 0.3** on Android: official WIT (not transitional `u32` / `future<u32>` smokes) and a **device instrument** per lane. Spec surface: [`../scheme/wasi-p3-surface.md`](../scheme/wasi-p3-surface.md). Pin: WASI **0.3.0**. One lane, one PR.

## Goal

A third party can run the P1 fixture set on a real device (`:smoke-app:connectedDebugAndroidTest` for that lane) and see official `@0.3.0` package shapes. Not a full [wasi-testsuite](https://github.com/WebAssembly/wasi-testsuite) pass (NG-4). Not Maven Central (NG-6).

Already landed (do not re-cut): `async func` / oneshot `future` / stream read+write smokes; `wasi:random` u64 + bytes; monotonic now / wait-for / wait-until / resolution; transitional system-clock + cli stdio + command-shaped `run`.

## Select the cut

If the user named a lane, keep **one** family. Otherwise:

```powershell
.\scripts\wasi-p3-remaining.ps1
```

No `pwsh`: `python3 ./scripts/wasi-p3-remaining.py` (same flags: `--all`).

Do the printed **Next:** line. W1–W8 smokes are landed. Remaining auto order: **P1-FS1 → P1-FS2 → P1-FS3 → P1-FS4 → P1-SK1 → P1-SK2 → P1-HT1** ([`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md)).

## Hard bans

- Do **not** start wasi:webgpu / Dawn / androidx consume PRs. GPU gap page is documentation only.
- Do **not** add `wasmtime-wasi` as a Cargo dependency unless that lane’s changelog records a size + Android thread review. Default is the existing thin JNI/Kotlin host (like clocks).
- Do **not** WebFetch or clone WASI / Wasmtime to “discover” WIT mid-cut. Copy the **current fixture** for that package and promote its import to the official 0.3.0 signature. Record the official names in the changelog fragment.
- Do **not** read `native/src/cm.rs` without an offset. Grep the WASI instance string (`wasi:clocks/…`, `wasi:cli/…`), then Read ~80 lines.
- Do **not** edit hub files: root `README.md` / `README.zh.md`, `CHANGELOG.md`, `.github/workflows/ci.yml`, `CONTRIBUTING.md`.
- Do **not** crate-`cargo fmt`. rustfmt **only** `.rs` files this slice changed.
- Do **not** treat Latch / sync-compat as true CM async (NG-8).
- Do **not** re-implement `wasi:io@0.2` pollable as the 0.3 path.
- PowerShell: no bash `&&`, no bash HEREDOC. `git commit` / `gh pr create` use `@"..."@`.
- Never file GitHub issues on Wasmtime, WASI, or any other upstream.

`request-adapter` / webgpu behavior is frozen. Keep true `async` wraps (`func_wrap_concurrent` + yield) for WIT `async func`.

## Lanes (auto)

| PR | Sentinel (remaining drops the lane when this changes) | DoD |
|----|------------------------------------------------------|-----|
| W1 | missing `fixtures/p3/stream_chunks.wat` | Multi-chunk `stream<T>` + backpressure or error-complete path. Native test + **device** `Stream*InstrumentedTest` (extend existing or add). Not a second copy of the 4-byte `P3ST`/`P3WR` smoke |
| W2 | `fixtures/wasi/system_now.wat` still `not official instant` | `wasi:clocks/system-clock@0.3.0#now` (and resolution if still `u64`) uses the official `instant` / datetime record. Timezone: implement if the 0.3.0 package exports it this cut; otherwise changelog “no timezone in 0.3.0 pin” and drop the sentinel anyway. Device: `WasiSystemClockInstrumentedTest` |
| W3 | `fixtures/wasi/cli_stdout.wat` still `future<u32> byte count` | `stdout` **and** `stderr` `write-via-stream` → official `future<result<_, error-code>>` (ok path at least). Device: stdout + stderr instruments |
| W4 | `fixtures/wasi/cli_stdin.wat` still `func() -> stream<u8>` | `stdin.read-via-stream` official `tuple<stream<u8>, future<result<_, error-code>>>`. Device: `WasiCliStdinInstrumentedTest` |
| W5 | `fixtures/wasi/cli_command.wat` still `official empty result deferred` | Command-shaped guest `run` is official (empty `result` / 0.3.0 command export), still using existing stdio. Not a full world with fs/sockets — that is W6–W8. Device: `WasiCliCommandInstrumentedTest` |
| W6 | missing `fixtures/wasi/filesystem_preopen.wat` | `wasi:filesystem` Android sandbox: document the path policy on [`../mapping/threading-android.md`](../mapping/threading-android.md) or a short mapping note **and** one preopen + read/write smoke on device. No world-writable shared storage |
| W7 | missing `fixtures/wasi/sockets_tcp.wat` | `wasi:sockets` Android subset: loopback or documented permission; true async; device instrument. Changelog must mention INTERNET + thread policy |
| W8 | missing `fixtures/wasi/http_handler.wat` | `wasi:http` Android subset: one `incoming-handler` / proxy smoke on device. May stay loopback. Evaluate `wasmtime-wasi` only in that PR’s fragment |

W1–W8 smokes are **landed**. Do **not** re-cut them. Remaining auto knives are the official-shape gap ([`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md)):

| PR | Sentinel | DoD (summary; full table on the gap page) |
|----|----------|-------------------------------------------|
| P1-FS1 | *(landed)* fixture no longer has `gap: get-directories not list tuple` | `get-directories` → `list<tuple<descriptor, string>>` |
| P1-FS2 | *(landed)* fixture no longer has `gap: read/write no filesize offset` | r/w-via-stream take `offset: filesize` |
| P1-FS3 | *(landed)* fixture no longer has `gap: no open-at` | directory preopen + `open-at` happy path |
| P1-FS4 | same file still `gap: open-at access not guest-visible` | guest `..` → `access` |
| P1-SK1 | `sockets_tcp.wat` still `gap: create-tcp-socket no address-family` | create takes `ip-address-family` → `result` |
| P1-SK2 | same file still `gap: connect no ip-socket-address` | `connect: async func(ip-socket-address) -> result` |
| P1-HT1 | `http_handler.wat` still `gap: handle not result<response>` | `handle -> result<response, error-code>` |

Copy: existing `native/src/cm.rs` instance block for that package + `native/tests/wasi_*.rs` + `fixtures/wasi/` or `fixtures/p3/` + the matching `smoke-app/…/Wasi*InstrumentedTest.kt`. Do not add a second linker stack.

## Named-only (never `Next:`)

| Lane | When | DoD |
|------|------|-----|
| WASI 0.2 polyfill | User named 0.2 / pollable | Do not make pollable the 0.3 path |
| Full wasi-testsuite | User named testsuite | Optional subset only; no compliance claim |
| Enable `wasmtime-wasi` crate | User named wasmtime-wasi | Size + Android thread review in-repo; still one lane |

## File whitelist

- `native/src/cm.rs` — this package’s `linker.instance` only (windowed)
- `native/src/lib.rs` — only if a new JNI export is required
- `native/tests/wasi_*.rs` — new or extended smoke (not `wasi_webgpu_*`)
- `fixtures/wasi/*` or `fixtures/p3/*` — guest; re-parse/validate `cm-async,component-model` when async
- `fixtures/wasi/README.md` or `fixtures/p3/README.md` — that fixture’s section only
- `smoke-app/src/androidTest/java/…/Wasi*InstrumentedTest.kt` or `Stream*InstrumentedTest.kt` — **required**
- `runtime-api/` / `runtime-jni/` — only if the Kotlin Store/pump API must grow
- `docs/mapping/threading-android.md` — when the pump or FS/socket thread policy changes
- `docs/mapping/gap-wasi-p3-wit.md` — that knife’s **one row** (Goal → Smoke)
- `docs/scheme/wasi-p3-surface.md` — that package’s **one row**
- `changelog/unreleased/<yyyy-mm-dd>-wasi-<slug>.md` (three bullets)

Do not add files under `docs/archive/`. Do not edit webgpu fixtures.

## Narrow tests

```powershell
cd native
cargo check --locked --lib
cargo test --locked --test wasi_<module> -- --test-threads=1
```

Device (required for the lane):

```powershell
.\gradlew.bat :smoke-app:connectedDebugAndroidTest -Pandroid.testInstrumentationRunnerArguments.class=<instrument>
```

Cloud images often have no device — still **add** the instrument; do not fail the PR solely because this checkout could not run it. State that in the PR.

PR title: `feat(wasi): L2 <package> <family>` (W1: `feat(wasi): L1 stream multi-chunk`). Label `enhancement`.

User prompt that works: “follow `docs/agent/wasi-p3.md`” or name the lane (`P1-FS1`, `G-fs-shape`). Docs-only gap table: [`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md).
