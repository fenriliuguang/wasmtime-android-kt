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
 * W3 `[method]` slice: `get-queue` then `[method]gpu-queue.submit` (single
 * command-buffer u32, not proposal `list`) via
 * [ExperimentalWebGpuBridge.attachQueueSubmit1] + [callRunConcurrent].
 * Flat `queue-submit1` remains. Not compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodQueueSubmitInstrumentedTest {
    @Test
    fun guestQueueSubmitViaMethodNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_queue_submit.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.create(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachQueueSubmit1(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val rep = instance.callRunConcurrent(store)
                                assertEquals("guest returns stub command-buffer 19", 19, rep)
                            }
                        }
                    }
                }
            }
        }
    }
}
