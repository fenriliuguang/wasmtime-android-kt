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

/** L2 `[method]gpu-queue.label` via [ExperimentalWebGpuBridge.attachDeviceGetQueue]. */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuMethodQueueLabelInstrumentedTest {
    @Test
    fun guestQueueLabelViaMethodNameSync() {
        val bytes =
            InstrumentationRegistry.getInstrumentation()
                .context
                .assets
                .open("w1/webgpu_method_queue_label.wasm")
                .use { it.readBytes() }
        CpuWasiWebGpuHost().use { host ->
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.createWithFixtureConstructors(engine).use { linker ->
                        Store.create(engine).use { store ->
                            ExperimentalWebGpuBridge.attachDeviceGetQueue(store, host)
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
