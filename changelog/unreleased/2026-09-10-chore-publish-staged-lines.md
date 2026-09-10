### Chore — stage Publish into native / dawn / pack plus snapshot and release lines (2026-09-10)

- Split [`.github/workflows/publish.yml`](../../.github/workflows/publish.yml) so a failed upload can retry without re-running NDK or Dawn C: `gate` → `native` ∥ `dawn` → `pack`, then a visible fork.
- `*-SNAPSHOT` GAV takes **`snapshot / publish`**; a non-SNAPSHOT GAV takes **`release / publish`**. The unused line is skipped. GitHub Environment `release` stays on the upload job (NG-6).
