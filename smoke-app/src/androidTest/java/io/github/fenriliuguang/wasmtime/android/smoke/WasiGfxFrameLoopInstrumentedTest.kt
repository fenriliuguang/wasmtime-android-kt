package io.github.fenriliuguang.wasmtime.android.smoke

import android.app.Activity
import android.os.SystemClock
import android.util.Log
import android.view.Choreographer
import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.WindowManager
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.runner.lifecycle.ActivityLifecycleMonitorRegistry
import androidx.test.runner.lifecycle.Stage
import androidx.webgpu.helper.Util
import io.github.fenriliuguang.wasi.webgpu.experimental.dawn.DawnWasiWebGpuHost
import io.github.fenriliuguang.wasmtime.android.Component
import io.github.fenriliuguang.wasmtime.android.Engine
import io.github.fenriliuguang.wasmtime.android.Linker
import io.github.fenriliuguang.wasmtime.android.Store
import io.github.fenriliuguang.wasmtime.android.webgpu.ExperimentalWebGpuBridge
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.io.FileInputStream
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference

/**
 * P010-GFXV: product gfx guest chains pin `get-gpu` → `gpu.request-adapter` →
 * `gpu-adapter.request-device`, then loops `on-frame` → `get-current-texture` →
 * `queue.submit` → `context.present` paced by Choreographer vsync (not two
 * events at construct). `surfaceDestroyed` closes the stream so `run` unblocks.
 * WG-6 one-shot `gpu-canvas-context` present stays a regression.
 *
 * Product [Linker.create] (no fixture `get-device`). Explicit Dawn attach
 * (P010-DISC). Cloud has no device.
 */
@RunWith(AndroidJUnit4::class)
class WasiGfxFrameLoopInstrumentedTest {
    @Test
    fun guestLoopsVsyncPacedPresentsToBoundWindow() {
        val storeRef = AtomicReference<Store?>(null)
        withReadySurface(storeRef) { ctx ->
            runOnGpuThread("GpuThread", timeoutSec = 90) {
                Log.i(TAG, "GpuThread: create Dawn host")
                DawnWasiWebGpuHost.create().use { host ->
                    Log.i(TAG, "GpuThread: bindCanvasNativeWindow ${ctx.width}x${ctx.height}")
                    host.bindCanvasNativeWindow(
                        Util.windowFromSurface(ctx.surface),
                        ctx.width,
                        ctx.height,
                    )
                    val bytes =
                        InstrumentationRegistry.getInstrumentation()
                            .context
                            .assets
                            .open("wasi/gfx_frame_loop.wasm")
                            .use { it.readBytes() }
                    Engine.create().use { engine ->
                        Component.compile(engine, bytes).use { component ->
                            Linker.create(engine).use { linker ->
                                Store.create(engine).use { store ->
                                    ExperimentalWebGpuBridge.attachDawnGuestCanvasPresent(store, host)
                                    linker.instantiate(store, component).use { instance ->
                                        startVsyncOnMain(store, storeRef)
                                        val closer =
                                            Thread({
                                                Thread.sleep(CLOSE_AFTER_VSYNC_MS)
                                                storeRef.get()?.closeGfxOnFrame()
                                            }, "gfx-on-frame-close")
                                        closer.start()
                                        val frames = instance.callRunConcurrent(store)
                                        storeRef.set(null)
                                        runCatching { store.closeGfxOnFrame() }
                                        closer.join(1_000)
                                        assertTrue(
                                            "guest must loop ≥2 vsync-paced on-frame presents, got $frames",
                                            frames >= 2,
                                        )
                                    }
                                }
                            }
                        }
                    }
                    runCatching { host.releaseAllGpuObjects() }
                    host.flushEvents()
                    Thread.sleep(SURFACE_RELEASE_SETTLE_MS)
                }
            }
        }
    }

    private fun startVsyncOnMain(store: Store, storeRef: AtomicReference<Store?>) {
        val started = CountDownLatch(1)
        InstrumentationRegistry.getInstrumentation().runOnMainSync {
            storeRef.set(store)
            Choreographer.getInstance().postFrameCallback(
                object : Choreographer.FrameCallback {
                    override fun doFrame(frameTimeNanos: Long) {
                        val s = storeRef.get() ?: return
                        s.postGfxVsync()
                        if (storeRef.get() != null) {
                            Choreographer.getInstance().postFrameCallback(this)
                        }
                    }
                },
            )
            started.countDown()
        }
        assertTrue("Choreographer vsync not posted", started.await(5, TimeUnit.SECONDS))
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

    private fun withReadySurface(
        storeRef: AtomicReference<Store?>,
        block: (ReadySurface) -> Unit,
    ) {
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

                    override fun surfaceDestroyed(holder: SurfaceHolder) {
                        storeRef.get()?.closeGfxOnFrame()
                    }
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
        private const val TAG = "P010GfxFrameLoop"
        private const val CLOSE_AFTER_VSYNC_MS = 500L
        private const val SURFACE_RELEASE_SETTLE_MS = 400L
        private const val SURFACE_READY_SETTLE_MS = 300L
        private const val ACTIVITY_TEARDOWN_SETTLE_MS = 500L
    }
}
