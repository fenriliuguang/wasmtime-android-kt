### Docs — remaining close-out (2026-09-02)

- Delete outdated `docs/archive/` (historical playbooks and dual-product notes).
- Merge the four product RFCs into one [`docs/scheme/rfc.md`](../../docs/scheme/rfc.md). Slim scheme: drop closed knife lists and archive stubs.
- Living leftover is **BIND → GFX-SIZE → GFX-PIN** ([`docs/agent/remaining.md`](../../docs/agent/remaining.md); `python3 ./scripts/remaining.py`).
- Named-only (never auto): `context.unconfigure`, timestamped `frame-event`, Lost/Outdated `result`, multi-window.
- Revert the mistaken **`0.2.0`** GAV bump. Coordinate stays **`0.1.0`** until that release is pressed (`gradle.properties`, `native/Cargo.toml`).
