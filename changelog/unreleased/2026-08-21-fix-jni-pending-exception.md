### Fix — JNI pending exception must clear before FindClass (2026-08-21)

- Host callback `UnsupportedOperationException` left a pending Java exception; the next `FindClass` (`throw_new` / Drop) aborted ART and killed the instrument process
- Clear + describe on `JavaException` in host `call_method` paths; `throw_kind` clears before throwing a typed `Wasmtime*Exception`
