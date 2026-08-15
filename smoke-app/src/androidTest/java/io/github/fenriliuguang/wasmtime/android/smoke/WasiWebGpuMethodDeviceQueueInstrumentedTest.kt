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
 * W3 `[method]` slice: guest imports `get-device` then
 * `wasi:webgpu/webgpu@0.3.0-rc.2#[method]gpu-device.queue` (sync getter;
 * resource self; still u32, not `gpu-queue`) via
 * [ExperimentalWebGpuBridge.attachDeviceGetQueue] + [callRunConcurrent].
 * Flat `device-get-queue` remains registered. Not full wasi:webgpu compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodDeviceQueueInstrumentedTest {
    @Test
    fun guestDeviceQueueViaMethodNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_device_queue.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.create(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachDeviceGetQueue(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val rep = instance.callRunConcurrent(store)
                                assertNotEquals("queue rep must be non-zero", 0, rep)
                                assertTrue("queue rep should be positive", rep > 0)
                            }
                        }
                    }
                }
            }
        }
    }
}
