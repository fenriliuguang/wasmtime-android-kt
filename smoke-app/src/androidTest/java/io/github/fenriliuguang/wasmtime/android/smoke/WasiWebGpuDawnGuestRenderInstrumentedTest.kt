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
 * WG-6: canonical `[method]` guest 3D on [DawnWasiWebGpuHost]
 * (not Cpu, not 1×1 color-clear cite, not `@builtin(vertex_index)`).
 * Guest chains shader + VERTEX buffer + render pipeline + set-vertex-buffer +
 * draw(3) + submit.
 * Dawn objects stay on one GpuThread ([docs/mapping/threading-android.md]).
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuDawnGuestRenderInstrumentedTest {
    @Test
    fun guestRenderViaDawnDescribedMethods() {
        runOnGpuThread("wg6-dawn-guest-render", timeoutSec = 90) {
            DawnWasiWebGpuHost.create().use { host ->
                val bytes =
                    InstrumentationRegistry.getInstrumentation()
                        .context
                        .assets
                        .open("w1/webgpu_method_dawn_guest_render.wasm")
                        .use { it.readBytes() }
                Engine.create().use { engine ->
                    Component.compile(engine, bytes).use { component ->
                        Linker.create(engine).use { linker ->
                            Store.create(engine).use { store ->
                                ExperimentalWebGpuBridge.attachDawnGuestRender(store, host)
                                linker.instantiate(store, component).use { instance ->
                                    val harness = instance.callRunConcurrent(store)
                                    assertEquals(
                                        "guest must complete vertex+draw+submit and return harness 1",
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
