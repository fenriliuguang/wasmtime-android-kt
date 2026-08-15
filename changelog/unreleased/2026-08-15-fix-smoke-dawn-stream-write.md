### Fix — smoke-app Dawn resume + stream-write 8MiB pump (2026-08-15)

- `nativeCallStreamWrite` pumps `run_concurrent` on the 8MiB `wasmtime-cm-pump` pthread (same as M2); ART instrument threads are ~1MiB and overflow on guest `stream.write`
- `CollectConsumer` empty probe returns `Pending` without `wake_by_ref` (self-wake re-polls while `stream.write` is still on the stack); non-empty chunks still return `Completed`
- Dawn instrument: wake + dismiss keyguard, then privileged `am start -W` (not `targetContext.startActivity`, which Android 16 / Vivo treats as background and never `RESUMED`); `MainActivity` `setTurnScreenOn` / `showWhenLocked`
