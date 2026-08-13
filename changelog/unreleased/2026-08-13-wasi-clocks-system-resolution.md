### Code — WASI 0.3 wasi:clocks system-clock.resolution smoke (2026-08-13)

- Register `wasi:clocks/system-clock@0.3.0#resolution` (transitional `func() -> u64` ns; host returns 1)
- Fixture `fixtures/wasi/system_resolution`; native `wasi_system_resolution`; twin instrument `WasiSystemResolutionInstrumentedTest`
- timezone not in this PR
