package io.github.fenriliuguang.wasmtime.android.smoke

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.github.fenriliuguang.wasi.webgpu.experimental.host.CpuWasiWebGpuHost
import io.github.fenriliuguang.wasmtime.android.Component
import io.github.fenriliuguang.wasmtime.android.Engine
import io.github.fenriliuguang.wasmtime.android.Linker
import io.github.fenriliuguang.wasmtime.android.Store
import io.github.fenriliuguang.wasmtime.android.webgpu.ExperimentalWebGpuBridge
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith

/**
 * L2 `[method]gpu-adapter.request-device`
 * (async; `(borrow gpu-adapter, option<gpu-device-descriptor>)
 * -> result<own gpu-device, request-device-error>`; descriptor=none;
 * drops own on ok; `run` returns 1)
 * via [ExperimentalWebGpuBridge.attachRequestDevice] + [callRunConcurrent].
 * Flat `adapter-request-device` remains registered. Not full wasi:webgpu
 * compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodRequestDeviceInstrumentedTest {
    @Test
    fun guestRequestDeviceViaMethodNameAsync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_request_device.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.createWithFixtureConstructors(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachRequestDevice(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val harness = instance.callRunConcurrent(store)
                                assertEquals(
                                    "guest must drop result<own<gpu-device>, …> ok and return harness 1",
                                    1,
                                    harness,
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}
