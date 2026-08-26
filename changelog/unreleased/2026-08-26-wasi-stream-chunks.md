### Code — WASI 0.3 stream multi-chunk + backpressure (2026-08-26)

- Guest `fixtures/p3/stream_chunks`: three 4-byte `stream.write` of `P3C1P3C2P3C3` with retry on partial count; host `take-chunks` pipes a 2-byte-per-poll `StreamConsumer` (backpressure). Not a second copy of the 4-byte `P3ST` / `P3WR` smokes
- Native `p3_stream_chunks` (byte payload + ≥6 consume polls); device `StreamChunksInstrumentedTest` reuses `callStreamWrite` / guest `run` / 8MiB CM pump
- `CollectConsumer.max_per_poll` keeps existing `take` / cli stdio unlimited; W1 is the capped path only
