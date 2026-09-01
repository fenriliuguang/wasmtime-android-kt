package io.github.fenriliuguang.wasmtime.android.smoke

import android.app.Activity
import android.os.SystemClock
import android.util.Log
import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.WindowManager
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.runner.lifecycle.ActivityLifecycleMonitorRegistry
import androidx.test.runner.lifecycle.Stage
import androidx.webgpu.helper.Util
import io.github.fenriliuguang.wasmtime.android.Component
import io.github.fenriliuguang.wasmtime.android.Engine
import io.github.fenriliuguang.wasmtime.android.Linker
import io.github.fenriliuguang.wasmtime.android.Store
import io.github.fenriliuguang.wasmtime.android.host.dawn.GpuBackends
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.io.FileInputStream
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference

/**
 * WG-6: guest records a render pass on `gpu-canvas-context.get-current-texture`,
 * then host presents after `queue.submit` (not host-only clear, not create-texture 1×1).
 *
 * ND-DEVICE: [GpuBackends.dawn] / NativeGpu after [Store.create]
 * (not `dawn-jni`, not [Store.createWithDiscoveredBackend]).
 * P010-FIX: guest wasm still imports fixture constructors, so this
 * instrument uses [Linker.createWithFixtureConstructors] (not product
 * [Linker.create]).
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuDawnGuestCanvasPresentInstrumentedTest {
    @Test
    fun guestDrawPresentsToBoundWindow() {
        withReadySurface { ctx ->
            runOnGpuThread("wg6-guest-canvas-present", timeoutSec = 90) {
                Log.i(TAG, "GpuThread: NativeGpu + bindCanvasNativeWindow ${ctx.width}x${ctx.height}")
                val bytes =
                    InstrumentationRegistry.getInstrumentation()
                        .context
                        .assets
                        .open("w1/webgpu_method_dawn_guest_canvas_present.wasm")
                        .use { it.readBytes() }
                Engine.create().use { engine ->
                    Component.compile(engine, bytes).use { component ->
                        Linker.createWithFixtureConstructors(engine).use { linker ->
                            Store.create(engine).use { store ->
                                store.setWebGpuBackend(GpuBackends.dawn())
                                store.bindCanvasNativeWindow(
                                    Util.windowFromSurface(ctx.surface),
                                    ctx.width,
                                    ctx.height,
                                )
                                linker.instantiate(store, component).use { instance ->
                                    val harness = instance.callRunConcurrent(store)
                                    assertEquals(
                                        "guest must get-current-texture + draw + submit and return harness 1",
                                        1,
                                        harness,
                                    )
                                }
                            }
                        }
                    }
                }
                Thread.sleep(SURFACE_RELEASE_SETTLE_MS)
            }
        }
    }

    private data class ReadySurface(
        val surface: Surface,
        val width: Int,
        val height: Int,
    )

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

    private fun withReadySurface(block: (ReadySurface) -> Unit) {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val context = instrumentation.targetContext

        wakeDeviceForSurface()
        val pkg = context.packageName
        shellCommand(
            "am start -W -n $pkg/.MainActivity " +
                "-f 0x10008000 --ez ${MainActivity.EXTRA_SKIP_DEMO_AUTORUN} true",
        )

        val activity = waitForResumedMainActivity(timeoutMs = 30_000)
        instrumentation.runOnMainSync {
            activity.window?.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        }
        val surfaceReady = CountDownLatch(1)
        val surfaceRef = AtomicReference<Surface?>(null)
        val widthRef = AtomicReference(0)
        val heightRef = AtomicReference(0)

        instrumentation.runOnMainSync {
            val surfaceView = activity.findViewById<SurfaceView>(R.id.demoSurface)
            fun capture(surface: Surface, width: Int, height: Int) {
                if (!surface.isValid || width <= 0 || height <= 0) return
                surfaceRef.set(surface)
                widthRef.set(width)
                heightRef.set(height)
                surfaceReady.countDown()
            }
            val holder = surfaceView.holder
            val frame = holder.surfaceFrame
            val existing = holder.surface
            if (existing != null && existing.isValid && frame.width() > 0 && frame.height() > 0) {
                capture(existing, frame.width(), frame.height())
            }
            holder.addCallback(
                object : SurfaceHolder.Callback {
                    override fun surfaceCreated(holder: SurfaceHolder) {
                        val f = holder.surfaceFrame
                        capture(holder.surface, f.width(), f.height())
                    }

                    override fun surfaceChanged(
                        holder: SurfaceHolder,
                        format: Int,
                        width: Int,
                        height: Int,
                    ) {
                        capture(holder.surface, width, height)
                    }

                    override fun surfaceDestroyed(holder: SurfaceHolder) = Unit
                },
            )
        }

        val deadline = SystemClock.uptimeMillis() + 20_000
        while (surfaceReady.count > 0 && SystemClock.uptimeMillis() < deadline) {
            instrumentation.runOnMainSync {
                if (surfaceReady.count == 0L) return@runOnMainSync
                if (activity.isFinishing) return@runOnMainSync
                val surfaceView = activity.findViewById<SurfaceView>(R.id.demoSurface)
                val holder = surfaceView.holder
                val frame = holder.surfaceFrame
                val surface = holder.surface
                if (surface != null && surface.isValid && frame.width() > 0 && frame.height() > 0) {
                    surfaceRef.set(surface)
                    widthRef.set(frame.width())
                    heightRef.set(frame.height())
                    surfaceReady.countDown()
                }
            }
            if (surfaceReady.count > 0) {
                Thread.sleep(50)
            }
        }
        assertTrue("Surface not ready within timeout", surfaceReady.await(1, TimeUnit.SECONDS))
        Thread.sleep(SURFACE_READY_SETTLE_MS)

        try {
            block(
                ReadySurface(
                    surface = requireNotNull(surfaceRef.get()),
                    width = widthRef.get(),
                    height = heightRef.get(),
                ),
            )
        } finally {
            instrumentation.runOnMainSync { activity.finish() }
            instrumentation.waitForIdleSync()
            Thread.sleep(ACTIVITY_TEARDOWN_SETTLE_MS)
        }
    }

    private fun waitForResumedMainActivity(timeoutMs: Long): MainActivity {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val monitor = ActivityLifecycleMonitorRegistry.getInstance()
        val deadline = SystemClock.uptimeMillis() + timeoutMs
        while (SystemClock.uptimeMillis() < deadline) {
            val found = AtomicReference<MainActivity?>(null)
            instrumentation.runOnMainSync {
                @Suppress("UNCHECKED_CAST")
                val resumed = monitor.getActivitiesInStage(Stage.RESUMED) as Collection<Activity>
                found.set(resumed.filterIsInstance<MainActivity>().firstOrNull())
            }
            found.get()?.let { return it }
            Thread.sleep(50)
        }
        error("MainActivity not RESUMED within ${timeoutMs}ms")
    }

    private fun wakeDeviceForSurface() {
        shellCommand("input keyevent KEYCODE_WAKEUP")
        shellCommand("wm dismiss-keyguard")
    }

    private fun shellCommand(cmd: String) {
        InstrumentationRegistry.getInstrumentation().uiAutomation.executeShellCommand(cmd).use { pfd ->
            FileInputStream(pfd.fileDescriptor).use { ins ->
                while (ins.read() != -1) {
                    // Drain so the shell command is not killed early.
                }
            }
        }
    }

    companion object {
        private const val TAG = "WG6GuestCanvasPresent"
        private const val SURFACE_RELEASE_SETTLE_MS = 400L
        private const val SURFACE_READY_SETTLE_MS = 300L
        private const val ACTIVITY_TEARDOWN_SETTLE_MS = 500L
    }
}
