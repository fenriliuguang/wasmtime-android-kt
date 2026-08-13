### Fix — callRunConcurrent 8MiB pump thread (W3 device-get-queue) (2026-08-13)

- `Instance.callRunConcurrent` hops to a dedicated 8MiB-stack thread so W3 `device-get-queue` instrument does not `StackOverflowError` on ~1MiB ART instrument threads (Vivo)
