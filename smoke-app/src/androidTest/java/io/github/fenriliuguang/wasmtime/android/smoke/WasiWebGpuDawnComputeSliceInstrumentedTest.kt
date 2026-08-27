package io.github.fenriliuguang.wasmtime.android.smoke

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.github.fenriliuguang.wasi.webgpu.experimental.dawn.DawnWasiWebGpuHost
import io.github.fenriliuguang.wasmtime.android.Component
import io.github.fenriliuguang.wasmtime.android.Engine
import io.github.fenriliuguang.wasmtime.android.Linker
import io.github.fenriliuguang.wasmtime.android.Store
import io.github.fenriliuguang.wasmtime.android.webgpu.ExperimentalWebGpuBridge
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference

/**
 * Lane D cite: canonical `[method]` compute slice on [DawnWasiWebGpuHost]
 * (not Cpu). Guest chains create-buffer → encoder → begin-compute-pass →
 * end → finish → [method]gpu-queue.submit. Not CTS / not a compliant product.
 * Dawn objects stay on one GpuThread ([docs/mapping/threading-android.md]).
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuDawnComputeSliceInstrumentedTest {
    @Test
    fun guestComputeSliceViaDawnDescribedMethods() {
        runOnGpuThread("d-dawn-compute-slice", timeoutSec = 90) {
            DawnWasiWebGpuHost.create().use { host ->
                val bytes =
                    InstrumentationRegistry.getInstrumentation()
                        .context
                        .assets
                        .open("w1/webgpu_method_dawn_compute_slice.wasm")
                        .use { it.readBytes() }
                Engine.create().use { engine ->
                    Component.compile(engine, bytes).use { component ->
                        Linker.createWithFixtureConstructors(engine).use { linker ->
                            Store.create(engine).use { store ->
                                ExperimentalWebGpuBridge.attachDawnComputeSlice(store, host)
                                linker.instantiate(store, component).use { instance ->
                                    val harness = instance.callRunConcurrent(store)
                                    assertEquals(
                                        "guest must complete the compute slice and return harness 1",
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

    private fun runOnGpuThread(name: String, timeoutSec: Long, block: () -> Unit) {
        val done = CountDownLatch(1)
        val error = AtomicReference<Throwable?>(null)
        Thread({
            try {
                block()
            } catch (t: Throwable) {
                error.set(t)
            } finally {
                done.countDown()
            }
        }, name).start()
        assertTrue("$name timed out", done.await(timeoutSec, TimeUnit.SECONDS))
        val failure = error.get()
        if (failure != null) {
            throw AssertionError("$name failed: ${failure.message}", failure)
        }
    }
}
