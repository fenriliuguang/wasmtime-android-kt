# wasmtime-android-kt

**experimental · Android-first 的 Java/Kotlin Component 运行时**

[English](README.md) | **中文**

基于 **官方 Wasmtime** 的 Android（JNI / ART）嵌入，托管 [Component Model](https://component-model.bytecodealliance.org/) Guest（含 **真 CM async**），首发提案世界为规范形状的 [`wasi:webgpu`](https://github.com/WebAssembly/wasi-webgpu)。

本仓目标是成为 Wasm 组件链上 **可引用的 Android Host**——不是 UI 框架、不是重写的 Dawn、也不是生产级 WASI 发行版。**默认产品/测试构件包含 Dawn**；核心 AAR 不含。见 [`rfc.md`](docs/scheme/rfc.md)。

状态：**experimental `0.x`**。坐标 **`0.1.0`**（已发布）。不宣称合规 wasi:webgpu / CTS。产品子集：[`claim-010.md`](docs/scheme/claim-010.md)。后续发包：`.github/workflows/publish.yml`（`main` 上的 `v*` 标签，或从 `main` 手动触发；GitHub Environment `release`）。

若与英文冲突，**以 [README.md](README.md) 为准**。禁止向上游开 GitHub issue。非紧急：`unconfigure`、带时间戳的 `frame-event`、Lost/Outdated `result`、多窗口。

## 使用 `0.1.0`

minSdk **24**。仓库：`mavenCentral()` + `google()`（`androidx.webgpu`）。混淆必须吃到 AAR 的 `consumer-rules.pro`。sockets / 出站 HTTP 需要 Android **INTERNET**。

推荐（0.x 默认包 — runtime + Dawn host）：

```kotlin
dependencies {
    implementation("io.github.fenriliuguang.wasmtime.android:android-webgpu:0.1.0")
}
```

无 GPU / 自带后端：`…:runtime:0.1.0`。只要 Dawn host：`…:host-dawn:0.1.0`。不要直接依赖 `runtime-api` / `runtime-jni`（`runtime` 的传递依赖）。不要依赖 `:smoke-app`。

不走 Maven 时，源码检出 / `includeBuild` 仍可用。

### Host

公共 SPI：`Engine` / `Store` / `Linker` / `Component` / `Instance`，`GpuBackends.dawn()`，`Store.setWebGpuBackend`。编译、实例化、`callRunConcurrent` 放在专用 **GpuThread**，不要占 ART 主线程（[线程约定](docs/mapping/threading-android.md)）。

```kotlin
Engine.create().use { engine ->
    Component.compile(engine, wasmBytes).use { component ->
        Linker.create(engine).use { linker ->
            Store.create(engine).use { store ->
                store.setWebGpuBackend(GpuBackends.dawn())
                store.bindCanvasNativeWindow(
                    NativeBridge.nativeWindowFromSurface(surface),
                    width,
                    height,
                )
                linker.instantiate(store, component).use { instance ->
                    // Choreographer / GpuThread: store.postGfxVsync(frameTimeNanos)
                    val frames = instance.callRunConcurrent(store)
                    store.closeGfxOnFrame()
                }
            }
        }
    }
}
```

- 未接线后端 → guest `gpu.request-adapter` 返回 **`none`**。`Store.createWithDiscoveredBackend` 是 ServiceLoader 便利路径；**`setWebGpuBackend` 始终优先**。
- 产品 `Linker.create` 不含 fixture 构造（`get-device`、HTTP request/response ctor）。钉 `get-gpu` 保留。
- 上屏循环：guest **拉** `wasi-gfx:surface@0.2.0` 的 `on-frame`；host 用 `Store.postGfxVsync` 送 vsync。`surfaceDestroyed` → `closeGfxOnFrame`。指针 / 按键：`postGfxPointer` / `postGfxKey`。
- `Engine` / `Store` / `Linker` / `Component` / `Instance` 都是 `AutoCloseable`，用完关闭。

### Guest

交付 **Component** wasm（不是仅 core module）。钉 **`wasi:webgpu@0.3.0-rc.2`**：`get-gpu` → `request-adapter` → `request-device`。连续上屏走 **`wasi-gfx:surface@0.2.0`**。WIT 规则：[`guest-shape.md`](docs/scheme/guest-shape.md)。`0.1.0` 实际覆盖面：[`claim-010.md`](docs/scheme/claim-010.md)。

端到端示例（打包 guest、加载、上屏）：[wasmtime-android-kt-examples](https://github.com/fenriliuguang/wasmtime-android-kt-examples)。本仓不内嵌该 app。仓外门禁：`.\scripts\verify-examples-gate.ps1`（includeBuild，不用 mavenLocal）。

## 从源码构建

```powershell
.\scripts\build-native-android.ps1
.\gradlew.bat :smoke-app:assembleDebug
.\gradlew.bat :smoke-app:connectedDebugAndroidTest
```

版本钉：[`tech-stack.md`](docs/scheme/tech-stack.md)。协作：[`CONTRIBUTING.md`](CONTRIBUTING.md)。

## 文档

英文为正文；中文为副文档（`.zh.md`）。
