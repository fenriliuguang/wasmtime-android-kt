### Code — WASI 0.3 wasi:clocks system-clock official instant (2026-08-26)

- Promote `wasi:clocks/system-clock@0.3.0#now` and `#resolution` from transitional `u64` to official `instant` record `{seconds: s64, nanoseconds: u32}`
- Fixtures `system_now` / `system_resolution`; native `wasi_system_now` / `wasi_system_resolution`; device `WasiSystemClockInstrumentedTest` (guest still returns seconds) plus existing resolution instrument
- No timezone in the 0.3.0 pin (`system-clock` exports `now` + `resolution` only)
