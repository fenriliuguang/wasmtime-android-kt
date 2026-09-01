package io.github.fenriliuguang.wasmtime.android.smoke

import androidx.test.ext.junit.runners.AndroidJUnit4
import io.github.fenriliuguang.wasi.webgpu.experimental.host.CpuWasiWebGpuHost
import io.github.fenriliuguang.wasi.webgpu.experimental.host.Extent3D
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuTextureFormat
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuTextureUsage
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ResourceKind
import io.github.fenriliuguang.wasi.webgpu.experimental.host.TextureDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.TextureViewDescriptor
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Product canvas `get-current-texture` + present must not grow the handle table.
 * Cpu hitch twin (keep last+pending). NativeGpu keep-3 + GPU-done retire lives
 * in `native_gpu.rs`; this instrument does not attach `dawn-jni`.
 */
@RunWith(AndroidJUnit4::class)
class WasiWebGpuCanvasContextFrameLifetimeInstrumentedTest {
    @Test
    fun canvasGetCurrentTexturePresentDoesNotGrowHandleTable() {
        CpuWasiWebGpuHost().use { host ->
            val adapter = host.requestAdapter()
            val device = host.adapterRequestDevice(adapter)
            val ctx = host.canvasContextConfigure(
                0,
                device,
                GpuTextureFormat.RGBA8_UNORM,
                GpuTextureUsage.RENDER_ATTACHMENT,
            )
            val persistent = host.deviceCreateTexture(
                device,
                TextureDescriptor(
                    size = Extent3D(width = 8, height = 8),
                    format = GpuTextureFormat.RGBA8_UNORM,
                    usage = GpuTextureUsage.RENDER_ATTACHMENT,
                ),
            )
            val persistentView = host.textureCreateView(persistent, TextureViewDescriptor())
            val texturesBefore = host.handleCount(ResourceKind.Texture)
            val viewsBefore = host.handleCount(ResourceKind.TextureView)
            val totalBefore = host.handleCount()

            repeat(50) {
                val texture = host.canvasContextGetCurrentTexture(ctx.raw)
                host.textureCreateView(texture, TextureViewDescriptor())
                host.canvasContextPresent(ctx.raw)
            }

            assertEquals(
                "guest-owned texture must survive 50 canvas presents",
                8,
                host.textureWidth(persistent),
            )
            val texturesAfter = host.handleCount(ResourceKind.Texture)
            val viewsAfter = host.handleCount(ResourceKind.TextureView)
            val totalAfter = host.handleCount()
            assertTrue(
                "canvas textures must recycle (got $texturesAfter, baseline $texturesBefore)",
                texturesAfter in texturesBefore..(texturesBefore + 1),
            )
            assertTrue(
                "canvas views must recycle (got $viewsAfter, baseline $viewsBefore)",
                viewsAfter in viewsBefore..(viewsBefore + 1),
            )
            assertTrue(
                "handle table must not grow with get-current-texture + present (got $totalAfter, baseline $totalBefore)",
                totalAfter in totalBefore..(totalBefore + 2),
            )
            host.tryDrop(persistentView)
            host.tryDrop(persistent)
        }
    }
}
