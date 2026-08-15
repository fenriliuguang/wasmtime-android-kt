package io.github.fenriliuguang.wasmtime.android.smoke

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.github.fenriliuguang.wasi.webgpu.experimental.host.CpuWasiWebGpuHost
import io.github.fenriliuguang.wasmtime.android.Component
import io.github.fenriliuguang.wasmtime.android.Engine
import io.github.fenriliuguang.wasmtime.android.Linker
import io.github.fenriliuguang.wasmtime.android.Store
import io.github.fenriliuguang.wasmtime.android.webgpu.ExperimentalWebGpuBridge
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

/**
 * W3 `[method]` slice: guest imports `get-adapter` then
 * `wasi:webgpu/webgpu@0.3.0-rc.2#[method]gpu-adapter.request-device` (async;
 * resource self; still u32, not `result<gpu-device, …>`) via
 * [ExperimentalWebGpuBridge.attachRequestDevice] + [callRunConcurrent].
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
                    Linker.create(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachRequestDevice(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val rep = instance.callRunConcurrent(store)
                                assertNotEquals("device rep must be non-zero", 0, rep)
                                assertTrue("device rep should be positive", rep > 0)
                            }
                        }
                    }
                }
            }
        }
    }
}
