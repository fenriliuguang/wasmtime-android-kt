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

/** S6+ `[method]gpu-supported-limits.max-buffer-size` via [ExperimentalWebGpuBridge.attachAdapterInfo]. */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodSupportedLimitsMaxBufferSizeInstrumentedTest {
    @Test
    fun guestSupportedLimitsMaxBufferSizeViaMethodNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_supported_limits_max_buffer_size.wasm")
                .use { it.readBytes() }
        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.create(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachAdapterInfo(store, host)
                            linker.instantiate(store, component).use { instance ->
                                assertEquals(1, instance.callRunConcurrent(store))
                            }
                        }
                    }
                }
            }
        }
    }
}