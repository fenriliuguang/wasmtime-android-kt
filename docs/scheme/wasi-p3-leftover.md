# WASI 0.3 leftover queue (tracking)

**English** | [中文](wasi-p3-leftover.zh.md)

Living **auto** queue after `0.1.2`: fill the **named leftovers** in [`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md) on the in-tree **thin host**. Draft policy: [`rfc-wasi-p3.md`](rfc-wasi-p3.md) (sibling PR; strawman B). Guest concurrency / stackful CM async is **not** this queue ([`rfc-threads.md`](rfc-threads.md)).

NG-4 **stays**: wasi-testsuite / “all WASI 0.3 worlds” is **not** the KPI. Do **not** add `wasmtime-wasi` (size + Android thread is a later RFC). Do **not** claim complete WASI 0.3 or CTS.

Branch: **`cursor/wasi-p3-leftover-b677`**. Remaining: `python3 ./scripts/wasi-p3-leftover-remaining.py` (next **commit**, not next PR). A lane drops when its **`gap: l-… pending`** needle leaves **this file**. Do **not** remove a needle without landing that lane’s DoD. Open **one** PR to `main` only when this table has no pending needles. Named exception to CONTRIBUTING “no long-lived lines.”

## How to cut

1. Stay on **`cursor/wasi-p3-leftover-b677`**. Do not fork a short `feat/` per knife.
2. Run `python3 ./scripts/wasi-p3-leftover-remaining.py`. Do the printed **Next:** only — **one commit**.
3. Do **not** open a per-lane PR. Push the commit to this branch.
4. Hub freeze on lane commits: no root `README.md` / `README.zh.md`, `CHANGELOG.md`, `.github/workflows/ci.yml`, `CONTRIBUTING.md`. Exception: this playbook / gate-amendment commit.
5. New tests only as `native/tests/*.rs` + `fixtures/wasi/*`. Do not edit `ci.yml`.
6. Grep then Read ~80 lines of `native/src/cm.rs`. Do not crate-`cargo fmt`.
7. Never file GitHub issues on Wasmtime / WASI / any upstream. No `gh issue create`.

`rustc` **1.97.1**. Keep `func_wrap_concurrent` + yield for WIT `async func`. Reuse sandbox / helper-thread rules in [`../mapping/threading-android.md`](../mapping/threading-android.md).

## Needles (auto order)

<!-- remaining.py greps these exact strings. Keep one per unfinished lane. -->

| Lane | Needle (delete when landed) |
|------|------------------------------|
| L-RFC | landed 2026-09-05 (this playbook / remaining script / gate amendments) |
| L-ERR-CLI | landed 2026-09-05 (official `wasi:cli` `error-code`; NUL → illegal-byte-sequence; invalid UTF-8 → io) |
| L-ERR-FS | landed 2026-09-05 (official `wasi:filesystem` `error-code` variant; `..` → access; table miss → bad-descriptor; r/w IO → io / is-directory) |
| L-ERR-SOCK | landed 2026-09-05 (official sockets `error-code` variant; IPv6 create → not-supported; connect IO mapped off unknown) |
| L-ERR-HTTP | gap: l-err-http pending |
| L-CMD-ENV | gap: l-cmd-env pending |
| L-CMD-EXIT | gap: l-cmd-exit pending |
| L-CMD-TERM | gap: l-cmd-term pending |
| L-FS-STAT | gap: l-fs-stat pending |
| L-FS-DIR | gap: l-fs-dir pending |
| L-FS-APPEND | gap: l-fs-append pending |
| L-FS-SYNC | gap: l-fs-sync pending |
| L-FS-TIMES | gap: l-fs-times pending |
| L-SOCK-LISTEN | gap: l-sock-listen pending |
| L-SOCK-UDP | gap: l-sock-udp pending |
| L-SOCK-DNS | gap: l-sock-dns pending |
| L-HTTP-FIELDS | gap: l-http-fields pending |
| L-HTTP-TRAIL | gap: l-http-trail pending |
| L-HTTP-TLS | gap: l-http-tls pending |
| L-HTTP-SVC | gap: l-http-svc pending |

## Lanes (auto)

| Commit | Needle | DoD |
|--------|--------|-----|
| **L-RFC** | *(this commit)* | Playbook + remaining script + gap/claim/CONTRIBUTING amendments. `wasi-p3-leftover-remaining.py` prints **`Next: L-ERR-CLI`**. |
| **L-ERR-CLI** | *(landed)* | Official `wasi:cli` `error-code` enum (`io` / `illegal-byte-sequence` / `pipe`). NUL → `illegal-byte-sequence`; invalid UTF-8 → `io`; cancelled oneshot → `pipe`. Fixture `cli_stdout_io`. |
| **L-ERR-FS** | *(landed)* | Official `wasi:filesystem` `error-code` variant. `open-at("..")` stays `access`; missing descriptor → `bad-descriptor`; r/w IO maps off `unknown` (`io` / `is-directory`). |
| **L-ERR-SOCK** | *(landed)* | Official sockets `error-code` variant. IPv6 create → `not-supported`; failed connect maps `connection-refused` / `remote-unreachable` / `other(none)` (not `unknown`). Fixture `sockets_tcp_ipv6`. |
| **L-ERR-HTTP** | `gap: l-err-http pending` | `HttpErrorCode` matches official `wasi:http` `error-code` used by product `send` / body. Empty authority / https-without-TLS stay guest-visible codes, not a crate. Remove the needle. |
| **L-CMD-ENV** | `gap: l-cmd-env pending` | `wasi:cli/environment@0.3.0` `get-environment` / `get-arguments` (host-supplied; Android: empty or documented `TMPDIR`). Fixture. Remove the needle. |
| **L-CMD-EXIT** | `gap: l-cmd-exit pending` | `wasi:cli/exit@0.3.0`. Guest `exit` completes `run` with official `result`. Do not kill the ART process. Remove the needle. |
| **L-CMD-TERM** | `gap: l-cmd-term pending` | `terminal-stdin` / `terminal-stdout` / `terminal-stderr` (Android: `none` is allowed). Not a fake TTY. Remove the needle. |
| **L-FS-STAT** | `gap: l-fs-stat pending` | `stat` / `stat-at` on the sandbox descriptor. Remove the needle. |
| **L-FS-DIR** | `gap: l-fs-dir pending` | `read-directory` as a CM stream of directory entries. Remove the needle. |
| **L-FS-APPEND** | `gap: l-fs-append pending` | `append-via-stream`. Remove the needle. |
| **L-FS-SYNC** | `gap: l-fs-sync pending` | `sync` / `sync-data`. Remove the needle. |
| **L-FS-TIMES** | `gap: l-fs-times pending` | `set-times` / `set-times-at` (sandbox files only). Remove the needle. |
| **L-SOCK-LISTEN** | `gap: l-sock-listen pending` | TCP bind / listen / accept. **Default sandbox: loopback only.** Changelog: Android INTERNET + no bind on ART main. Remove the needle. |
| **L-SOCK-UDP** | `gap: l-sock-udp pending` | `udp-create-socket` + send/receive subset. Same sandbox. Remove the needle. |
| **L-SOCK-DNS** | `gap: l-sock-dns pending` | `ip-name-lookup` (helper thread). Remove the needle. |
| **L-HTTP-FIELDS** | `gap: l-http-fields pending` | `wasi:http` fields / headers on request/response. Product linker still omits fixture-only constructors unless this lane documents otherwise. Remove the needle. |
| **L-HTTP-TRAIL** | `gap: l-http-trail pending` | Trailers on consume-body `option`. Remove the needle. |
| **L-HTTP-TLS** | `gap: l-http-tls pending` | https on `client.send`. Changelog **must** record `.so` size + which thread does TLS. No `wasmtime-wasi`. Remove the needle. |
| **L-HTTP-SVC** | `gap: l-http-svc pending` | Remaining `incoming-handler` / types shape for a guest `handle` (not a listen HTTP server). Remove the needle. Then remaining.py is empty → **one** PR to `main`. |

Path policy, INTERNET, and helper-thread IO stay as in [`../mapping/threading-android.md`](../mapping/threading-android.md). Do not put listen / large FS / TLS on the ART main thread.

## Named-only (never `Next:`)

| Item | Why |
|------|-----|
| wasi-testsuite P3 | NG-4 |
| `wasmtime-wasi` | Size + Android thread RFC (option C) |
| WASI 0.2 pollable | Out |
| `wasi:clocks` timezone | Not in the named leftover table |
| Guest wasm threads / stackful CM async | [`rfc-threads.md`](rfc-threads.md) |
| Benchmarks | Deferred |
| gfx `unconfigure` / timestamped `frame-event` / Lost/Outdated / multi-window | gfx named-only |
| This-repo 1.0.0 / CTS | [`rfc.md`](rfc.md) §6 / NG-5 |
| P0/P1 / native-dawn re-cuts | Closed |

## File whitelist (typical leftover lane)

- `native/src/cm.rs` — this family’s `linker.instance` only (windowed)
- `native/tests/wasi_*.rs` + `fixtures/wasi/*` — reuse; add one fixture per new import
- `docs/scheme/wasi-p3-leftover.md` — **remove this lane’s needle**
- `docs/mapping/gap-wasi-p3-wit.md` — one row
- `docs/mapping/threading-android.md` — only if sandbox / listen / TLS thread rules change
- `changelog/unreleased/<yyyy-mm-dd>-l-<slug>.md`

Do not vendor out-of-tree examples. Do not edit hub files on a lane commit.

## Narrow tests

```text
cd native && rustup default 1.97.1
cargo test --locked --test <this_family>
```

Cloud has **no** device. Add or reuse a native test; do not fail the **final** PR solely because `connectedAndroidTest` could not run here.

This playbook amendment (docs-only): `python3 ./scripts/wasi-p3-leftover-remaining.py` must print **`Next: L-ERR-CLI`** and name branch `cursor/wasi-p3-leftover-b677`.
