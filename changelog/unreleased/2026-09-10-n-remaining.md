### Docs / NativeGpu — Remaining Table BIND leftover (2026-09-10)

- Playbook [`docs/scheme/nativegpu-remaining.md`](../../docs/scheme/nativegpu-remaining.md); `python3 ./scripts/nativegpu-remaining.py` now prints **Next: (NativeGpu Remaining Table empty)**.
- Catch-up: **N-LIMITS** (`GetLimits`; Cloud still `1`), **N-COPY** (origin / mip / aspect / layout), **N-DYNOFF** (`set-bind-group` offsets).
- Remaining knives on this PR: **N-COMPINFO** (`GetCompilationInfo`), **N-POPERR** (pop-error-scope type/message), **N-UNCAPTURED** (uncaptured + lost callbacks), **N-WGSLFEAT** (`HasWGSLLanguageFeature`), **N-LABELS** (`wgpu*SetLabel` + host get), **N-IMMEDIATE** (set-immediates / debug group·marker), **N-FENCE** (`queue.submit` waits `OnSubmittedWorkDone` before canvas recycle). Cloud / missing `.so` stays table-backed.
