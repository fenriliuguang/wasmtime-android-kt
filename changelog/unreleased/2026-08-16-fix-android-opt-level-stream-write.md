### Fix — Android native default opt-level=2 (stream.write instrument) (2026-08-16)

- Stage instrument on Vivo: `StreamWriteInstrumentedTest` SIGSEGV (`Process crashed`, overflow on `roidJUnitRunner`) with Windows Android `opt-level=0` frames
- `scripts/build-native-android.ps1` now defaults `CARGO_PROFILE_RELEASE_OPT_LEVEL=2`; set `0` only if rustc ACCESS_VIOLATION
