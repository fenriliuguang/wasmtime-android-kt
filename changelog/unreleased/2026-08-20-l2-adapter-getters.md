### Code — L2 gpu-adapter getters guest fields to host (2026-08-20)

- Deepen `[method]gpu-adapter.features` / `limits` / `info` from lift-only stubs to described JNI: the guest adapter handle is validated by the host before the local resource lift
- Guest `get-adapter` still uses rep 0; the wrap stub-requests an adapter when needed; returned features/limits/info resources stay local lifts (their getters are omit-lane)
- New host API `adapterValidate`; fixtures unchanged (comment rows only)
