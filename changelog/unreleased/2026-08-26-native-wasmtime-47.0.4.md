### Native — P2-EVAL Wasmtime pin 47.0.4 (2026-08-26)

- Refresh `docs/scheme/wasmtime-tracking.md` §2/§3: last-checked 2026-08-26; crates.io latest stable **48.0.1**; latest 47.x **47.0.4**
- Patch `47.0.2` (lockfile was `47.0.3`) → **`47.0.4`**: upstream 47.0.4 records [GHSA-x84v-gj2h-g759](https://github.com/bytecodealliance/wasmtime/security/advisories/GHSA-x84v-gj2h-g759) (WASIp3 stream host heap) and [GHSA-vqjp-4c8c-hfgg](https://github.com/bytecodealliance/wasmtime/security/advisories/GHSA-vqjp-4c8c-hfgg) (filesystem trailing-slash sandbox)
- Do **not** jump to 48.x (needs §4.1 RFC). Remove `gap: p2 pin eval pending`. No `gap: p2 patch pending` (patch landed here)
