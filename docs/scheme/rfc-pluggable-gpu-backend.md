# RFC: Pluggable GPU backends (Dawn default, optional at product)

**Status: Accepted** · 2026-08-17  
**English** | [中文](rfc-pluggable-gpu-backend.zh.md)

> Amends [`charter.md`](charter.md), [`tech-stack.md`](tech-stack.md), [`non-goals.md`](non-goals.md), [`guest-shape.md`](guest-shape.md).  
> Guest probe: [`gpu.request-adapter`](guest-shape.md) → `option<own<gpu-adapter>>` (`none` = no usable adapter).  
> Does **not** change P0 WIT shape or true CM async.  
> **Amended 2026-08-26** by [`rfc-l5-productization.md`](rfc-l5-productization.md): dual-track attach; three coordinates; Central at `0.1.0`.  
> Host Kotlin is vendored in `:host-dawn`; Dawn `.so` is published `androidx.webgpu` ([`../blocked-gpu-host.md`](../blocked-gpu-host.md)).

## 1. Decision

| Question | Decision |
|----------|----------|
| Does this repo provide a Dawn-backed host? | **Yes.** The in-tree **test runtime** and the **default product artifact** include Dawn. |
| Must every consumer ship Dawn? | **No.** Core runtime AAR has **no** Dawn `.so`. Apps may omit the Dawn module and supply a spec-shaped host, or run with no GPU backend. |
| How does a guest see “no backend / no adapter”? | **`request-adapter` returns `none`** — same as WebGPU `GPU.requestAdapter()` → `null`. Not a missing import, not “resource not found”, not a trap. |
| Publish (L5)? | **Three consumer** coordinates (`runtime` / `host-dawn` / **`android-webgpu` default**). P010-PUB: `0.1.0` + [`.github/workflows/publish.yml`](../../.github/workflows/publish.yml). |
| “Dynamic backend” means? | **Dual track (L5):** explicit `setWebGpuBackend` is the stable contract; ServiceLoader is default-bundle convenience. Not Play Feature Delivery, not downloading `.so` at runtime. |

## 2. Why not one AAR

Dawn’s `.so` dominates APK size and NDK/license surface. L1 (Wasmtime JNI + WIT lowering) is useful without GPU. Forcing Dawn into `:android` makes “no WebGPU host” impossible and blocks a later split publish.

Android still **packages** native libraries at **app build** time. Backends are dynamic in the **process** (which `WebGpuBackend` is registered). They are not hot-plugged from the network.

## 3. Artifact graph

```text
                    ┌─────────────────────────┐
                    │  runtime-api            │  SPI only (no Dawn types)
                    └───────────┬─────────────┘
                                │
                    ┌───────────▼─────────────┐
                    │  runtime-jni            │  Wasmtime .so + WIT lowering
                    │  android (AAR)          │  ALWAYS registers wasi:webgpu
                    └───────────┬─────────────┘
                                │
              ┌─────────────────┼─────────────────┐
              │                 │                 │
              ▼                 ▼                 ▼
     (no extra AAR)     host-dawn AAR      third-party AAR
     request-adapter    implements SPI     implements SPI
     → none             + Dawn jniLibs     + their .so / CPU
                                │
                    ┌───────────▼─────────────┐
                    │  android-webgpu         │  PRODUCT DEFAULT bundle
                    │  api(android)           │  api(host-dawn)
                    └─────────────────────────┘
```

| Module (working names) | Ships | Maven (later) |
|------------------------|-------|----------------|
| `:runtime-api` + `:runtime-jni` + `:android` | L1, `libwasmtime_android_kt.so`, SPI | `…:runtime` (or keep `android`) |
| `:host-dawn` | Dawn adapter + Dawn `.so` | `…:host-dawn` |
| `:android-webgpu` | **Default product** = runtime + host-dawn | `…:android-webgpu` |

**Default for this repo:** `smoke-app` and any “recommended dependency” snippet depend on **`:android-webgpu`** (Dawn included).

**Opt-out:** depend on `:android` only; register your own backend, or register nothing.

Until Central exists, the same graph is Gradle `include` + `project()` deps. The Kotlin host mapping is **vendored in `:host-dawn`** ([`../blocked-gpu-host.md`](../blocked-gpu-host.md)); Dawn `.so` comes from published `androidx.webgpu`. Do not put Dawn types on `:runtime-jni`.

## 4. Runtime SPI

Owned by **this repo** (`runtime-api`). Must not mention unpublished foreign types (`WasiWebGpuHost`, `AbiCmHostBindings`) in the public SPI.

Sketch (names can move in the code PR):

```kotlin
fun interface WebGpuBackendFactory {
    fun create(): WebGpuBackend
}

interface WebGpuBackend : AutoCloseable {
    val id: String  // "dawn", "cpu", "none", or "custom:<name>"

    /**
     * WIT `gpu.request-adapter`. Return null → guest `none`.
     * Must not throw for “no adapter”. Throw only for broken JNI / violated thread rules.
     */
    fun requestAdapter(options: GpuRequestAdapterOptions?): GpuAdapterHandle?
    // Further methods grow with S-series; or the backend installs
    // ExperimentalHostCallbacks — see §4.1.
}
```

### 4.1 Two legal ways to attach (dual track; L5)

1. **Stable contract (always wins):** `store.setWebGpuBackend(backend)` or `GpuBackends.dawn()` before instantiate. BYO, tests, multi-backend **only** this path.  
2. **Default-bundle convenience:** `android-webgpu` may discover via `ServiceLoader`. Prefer a **new** factory (for example `Store.createWithDiscoveredBackend`) if changing today’s `discoverWebGpuBackend = false` default is too sharp. Zero factories → `request-adapter` **`none`**. Several factories → prefer `id == "dawn"`. R8 `consumer-rules.pro` on `:host-dawn` / `:android-webgpu` is part of the published contract.

Resolution order:

```text
1. Explicit setWebGpuBackend / attach
2. ServiceLoader (discover path only)
3. None → request-adapter returns none
```

Linker **always** defines `wasi:webgpu`. Missing backend is not a link error.

### 4.2 Guest vs Kotlin

| Layer | “No GPU” |
|-------|----------|
| Guest | `option` **none** only |
| Kotlin | `backendKind`: `None` / `Dawn` / `Custom(id)` so tests can tell “unwired” from “Dawn returned none on this device” |

Do not invent a guest error for “host not wired”.

### 4.3 Who owns the shape

| Surface | Owner | Third-party host |
|---------|--------|------------------|
| Guest WIT (`wasi:webgpu@0.3.0-rc.2`) | This repo’s L1 lowering | **Do not** re-export or re-lower |
| Kotlin `WebGpuBackend` SPI | This repo’s `runtime-api` | **Implement** this interface |

A third-party (or a future system WebGPU module) is “spec-shaped” iff it implements **this SPI**. Guest ABI is not their contract.

Today’s androidx.webgpu / bundled Dawn is `:host-dawn` (`id = "dawn"`). A later OS WebGPU would be another `host-*` AAR with a different `id`, attached the same way (`setWebGpuBackend`).

## 5. Test runtime

| Kind | Backend | Gate |
|------|---------|------|
| Unwired | none | `request-adapter` is `none`; **no** mavenLocal / Dawn `.so` required |
| Shape / marshalling (optional) | CPU stub or fake `some` | May live under `:host-dawn` test fixtures or a tiny `:host-cpu` later |
| **Default integration / on-screen** | **Dawn** | `smoke-app` depends on `:android-webgpu`; GpuThread rules apply |

Default **does not** mean every `androidTest` class instantiates Dawn. Default means: the **application under test** is built like a product that includes Dawn; tests that need a real adapter use that wiring; tests that prove `none` use a store with no backend.

## 6. Consumer recipes (later publish)

**Default (Dawn in the APK):**

```kotlin
dependencies {
    implementation("io.github.fenriliuguang.wasmtime.android:android-webgpu:<ver>")
}
```

**Bring your own host:**

```kotlin
dependencies {
    implementation("io.github.fenriliuguang.wasmtime.android:android:<ver>")
    implementation("com.example:my-webgpu-host:<ver>") // implements WebGpuBackend
}
```

```kotlin
store.setWebGpuBackend(MyBackend())
```

**No GPU:** depend on `:android` only; do not register a backend.

## 7. Non-goals this RFC does not lift

- A second Dawn **implementation** (NG-7). This repo **packages / adapts** Dawn, it does not rewrite it.  
- Full Kotlin WebGPU client API (NG-3).  
- Compliant wasi:webgpu product claim (NG-5).  
- Maven Central **before `0.1.0` gates** (NG-6 / L5). The **coordinate split** is what L5 publishes then.  
- Runtime download of Dawn `.so`.

## 8. Follow-up code

Landed: `:host-dawn` / `:android-webgpu`; `ExperimentalWebGpuBridge` moved out of `:runtime-jni`; unwired `request-adapter` → `none`; `smoke-app` depends on the Dawn bundle; Host Kotlin vendored; `androidx.webgpu` for Dawn `.so` (no mavenLocal `experimental:*`).

## 9. Revisions

- SPI method list grows with S-series; bump `0.x` MINOR.  
- Changing “default product includes Dawn” or “none vs trap” needs a new RFC.  
- 2026-08-26: L5 dual-track (explicit = stable; discover = bundle convenience); Central at `0.1.0`.
- 2026-08-31: **Native Dawn host** ([`../agent/native-dawn.md`](../agent/native-dawn.md)). Default product adapter **will** be in-process Dawn C (same pin; one `.so`). Kotlin `WebGpuBackend` remains BYO / discover. androidx JNI consume becomes leftover `id = "dawn-jni"`. Not a second renderer (NG-7). `ND-DEFAULT` is a `0.x` MINOR when it lands. Full pin method suite is the consume DoD; cube is demo only.
