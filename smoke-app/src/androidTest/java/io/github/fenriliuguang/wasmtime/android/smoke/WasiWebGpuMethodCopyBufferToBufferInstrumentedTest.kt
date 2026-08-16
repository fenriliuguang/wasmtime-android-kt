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
 * W3+ `[method]` slice: `get-encoder` then
 * `[method]gpu-command-encoder.copy-buffer-to-buffer` (stub src/dst 31,
 * host-fixed 4-byte copy) via
 * [ExperimentalWebGpuBridge.attachCopyBufferToBuffer] + [callRunConcurrent].
 * Not compliance.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodCopyBufferToBufferInstrumentedTest {
    @Test
    fun guestCopyBufferToBufferViaMethodNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_copy_buffer_to_buffer.wasm")
                .use { it.readBytes() }

        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.create(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachCopyBufferToBuffer(store, host)
                            linker.instantiate(store, component).use { instance ->
                                val rep = instance.callRunConcurrent(store)
                                assertEquals("guest returns stub buffer 31 after copy", 31, rep)
                            }
                        }
                    }
                }
            }
        }
    }
}
