# Agent playbook: remaining close-out

**English** | [中文](remaining.zh.md)

Living **auto** queue. Tracking: [`../scheme/remaining.md`](../scheme/remaining.md). Policy: [`../scheme/rfc.md`](../scheme/rfc.md).

Do **not** re-cut closed P0 / P1 / `0.1.0` knives. P2 Wasmtime pin is **named-only** ([`wasmtime-p2.md`](wasmtime-p2.md)).

## Select the cut

```text
python3 ./scripts/remaining.py
```

Do the printed **Next:** only — one commit. Guest WIT names/args stay. Reuse `cm.rs` lowering; grep then Read ~80 lines. Do not reimplement `exp_*` JNI. Do not add `wasmtime-wasi` without a size + Android thread note. Never file upstream GitHub issues.

## Lanes (auto)

| Commit | Needle | DoD |
|--------|--------|-----|
| **BIND** | `gap: remaining bind pending` | Remaining pin `[method]`s that Dawn C has a slot **call `webgpu.h`** when `libwebgpu_dawn.so` is loaded (texture / sampler / compute / copies / map / query / bundle / error / depth-blend / indexed-indirect / viewport / adapter features / `write-texture` / work-done / …). Record holes stay Record. Cloud / missing `.so` may stay table-backed. Reuse `wasi_webgpu_method`. Remove the needle. |
| **GFX-SIZE** | `gap: remaining gfx-size pending` | `wasi-gfx:surface` `height` / `width` / `request-set-size` / `on-resize` registered and implemented against the bound window. Remove the needle. |
| **GFX-PIN** | `gap: remaining gfx-pin pending` | Remaining pin surface streams a complete gfx guest imports: `on-pointer-up/down/move`, `on-key-up/down`. Empty events are fine until input is wired; missing imports are not. Remove the needle. |

Cube / out-of-tree demo is evidence only.

## Named-only (never `Next:`)

| Item | Why |
|------|-----|
| `context.unconfigure` | Non-urgent; teardown today is `surfaceDestroyed` + host unconfigure |
| Timestamped `frame-event` | Pin is `{ nothing: bool }`; guest uses `wasi:clocks` |
| Lost/Outdated as `result` | Pin still returns a bare `gpu-texture` |
| Multi-window | DG-6 |
| P2 / G-cmd / G-fs-full / listen/UDP / testsuite / `wasmtime-wasi` / CTS / 1.0 | Existing named queues |

## File whitelist

- `native/src/dawn_c.rs`, `native_gpu.rs`, windowed `cm.rs`
- `docs/scheme/remaining.md` (remove this lane’s needle)
- `docs/mapping/gap-webgpu-native-dawn.md` (BIND leftover rows)
- `changelog/unreleased/<yyyy-mm-dd>-<slug>.md`

Hub freeze on a lane commit: no root `README.md` / `CHANGELOG.md` / `ci.yml` / `CONTRIBUTING.md`. rustfmt only `.rs` this slice changed.

## Tests

```text
cd native && cargo check --locked --lib
```

Plus the existing `cargo test --locked --test <family>` the lane flips. Cloud has no device and may have no Dawn `.so`; do not fail the lane solely for that.

## Commit messages

- BIND: `feat(webgpu): bind remaining pin methods to Dawn C`
- GFX-SIZE: `feat(gfx): surface size and resize`
- GFX-PIN: `feat(gfx): remaining wasi-gfx pin streams`
