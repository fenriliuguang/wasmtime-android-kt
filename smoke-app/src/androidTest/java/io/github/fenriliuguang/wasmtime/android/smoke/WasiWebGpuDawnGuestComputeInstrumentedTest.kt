package io.github.fenriliuguang.wasmtime.android.smoke

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.github.fenriliuguang.wasmtime.android.Component
import io.github.fenriliuguang.wasmtime.android.Engine
import io.github.fenriliuguang.wasmtime.android.Linker
import io.github.fenriliuguang.wasmtime.android.Store
import io.github.fenriliuguang.wasmtime.android.host.dawn.GpuBackends
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference

/**
 * WG-6: canonical `[method]` guest compute on native default
 * ([GpuBackends.dawn] / NativeGpu, not Cpu, not `dawn-jni`, not empty
 * begin-compute-pass). Guest chains BGL + bind-group + compute pipeline +
 * set-bind-group + dispatch-workgroups + submit.
 * Consume stays on one GpuThread ([docs/mapping/threading-android.md]).
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuDawnGuestComputeInstrumentedTest {
    @Test
    fun guestComputeViaDawnDescribedMethods() {
        runOnGpuThread("wg6-dawn-guest-compute", timeoutSec = 90) {
            val bytes =
                InstrumentationRegistry.getInstrumentation()
                    .context
                    .assets
                    .open("w1/webgpu_method_dawn_guest_compute.wasm")
                    .use { it.readBytes() }
            Engine.create().use { engine ->
                Component.compile(engine, bytes).use { component ->
                    Linker.createWithFixtureConstructors(engine).use { linker ->
                        Store.create(engine).use { store ->
                            store.setWebGpuBackend(GpuBackends.dawn())
                            linker.instantiate(store, component).use { instance ->
                                val harness = instance.callRunConcurrent(store)
                                assertEquals(
                                    "guest must complete BGL+bind-group+dispatch and return harness 1",
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
