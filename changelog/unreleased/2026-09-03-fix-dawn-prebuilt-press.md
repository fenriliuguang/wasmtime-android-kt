### Fix — press NativeGpu `.so` matches on-device green (`0.1.2-SNAPSHOT`) (2026-09-03)

- GAV **`0.1.2-SNAPSHOT`** (Central publishing limits: snapshot does not consume a release quota). Maven `0.1.1` packed CI `--build` Dawn (`9d41fdf`) plus Linux opt-level 3 `libwasmtime`; the device-green pair was `--prebuilt` (`bddf1a04`) plus opt-level 2. Press now runs `--prebuilt` and pins `CARGO_PROFILE_RELEASE_OPT_LEVEL=2` on Linux CI as well.
- In-tree press gate `scripts/verify-press-aar.py`: release AAR `.so` SHA256 must match the recipe files (and `--prebuilt` ORIGIN). Does not clone examples. includeBuild cube stays local-dev.
- Policy: **`SNAPSHOT` GAV is allowed.** CONTRIBUTING / `ci.yml` / `publish.yml` no longer reject it.
