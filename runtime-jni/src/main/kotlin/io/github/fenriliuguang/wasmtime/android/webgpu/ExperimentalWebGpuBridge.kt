package io.github.fenriliuguang.wasmtime.android.webgpu

import io.github.fenriliuguang.wasi.webgpu.experimental.abicm.AbiCmHostBindings
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BindGroupLayoutDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.PipelineLayoutDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.Extent3D
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuBufferUsage
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuTextureFormat
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuTextureUsage
import io.github.fenriliuguang.wasi.webgpu.experimental.host.TextureDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.WasiWebGpuHost
import io.github.fenriliuguang.wasmtime.android.Store
import io.github.fenriliuguang.wasmtime.android.api.ExperimentalHostCallbacks

/**
 * Wire Track A L2 ([WasiWebGpuHost]) into Track B L1 store callbacks for the
 * experimental CM host interface (`AbiCm.IMPORT_INTERFACE`).
 *
 * Flat u32-rep imports (M3/M4); not full WIT resource method names.
 */
object ExperimentalWebGpuBridge {
    /**
     * W1/W2 flat `request-adapter` and W3 `[method]gpu.request-adapter` share this
     * L2 callback. `get-gpu` is host-only (no Kotlin).
     */
    fun attachRequestAdapter(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()
            },
        )
    }

    /**
     * W2 remainder: adapter + device (proposal-name async path still uses these
     * L2 callbacks). W3 `[method]gpu-adapter.request-device` shares this attach:
     * `get-adapter` is host-only (no Kotlin); the method then calls L2
     * `requestAdapter` + `adapterRequestDevice`.
     */
    fun attachRequestDevice(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)
            },
        )
    }

    /** W3: adapter + device + queue. Shared by flat `device-get-queue` and `[method]gpu-device.queue` (`get-device` is host-only). */
    fun attachDeviceGetQueue(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceGetQueue(device: Int): Int = bindings.deviceGetQueue(device)
            },
        )
    }

    /**
     * W3+: adapter + device + create-buffer. Host-fixed 4-byte COPY_DST|VERTEX
     * descriptor (no Guest record). `[method]gpu-device.create-buffer` only.
     */
    fun attachCreateBuffer(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreateBuffer(device: Int): Int =
                    bindings.deviceCreateBuffer(
                        device,
                        size = STUB_BUFFER_SIZE,
                        usage = GpuBufferUsage.COPY_DST or GpuBufferUsage.VERTEX,
                    )
            },
        )
    }

    /**
     * W3+: adapter + device + create-texture. Host-fixed 1×1 RGBA8
     * RENDER_ATTACHMENT (no Guest record). `[method]gpu-device.create-texture`.
     */
    fun attachCreateTexture(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreateTexture(device: Int): Int =
                    bindings.deviceCreateTexture(
                        device,
                        TextureDescriptor(
                            size = Extent3D(width = 1, height = 1),
                            format = GpuTextureFormat.RGBA8_UNORM,
                            usage = GpuTextureUsage.RENDER_ATTACHMENT,
                        ),
                    )
            },
        )
    }

    /**
     * W3+: adapter + device + create-sampler. Host-fixed default sampler
     * (no Guest record). `[method]gpu-device.create-sampler`.
     */
    fun attachCreateSampler(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreateSampler(device: Int): Int =
                    bindings.deviceCreateSampler(device)
            },
        )
    }

    /**
     * W3+: adapter + device + create-shader-module. Host-fixed stub WGSL
     * (no Guest string). `[method]gpu-device.create-shader-module`.
     */
    fun attachCreateShaderModule(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreateShaderModule(device: Int): Int =
                    bindings.deviceCreateShaderModule(device, STUB_WGSL)
            },
        )
    }

    /**
     * W3+: adapter + device + host-fixed empty bind-group-layout.
     * `[method]gpu-device.create-bind-group-layout` (no Guest descriptor).
     */
    fun attachCreateBindGroupLayout(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreateBindGroupLayout(device: Int): Int =
                    bindings.deviceCreateBindGroupLayout(
                        device,
                        BindGroupLayoutDescriptor(entries = emptyList()),
                    )
            },
        )
    }

    /**
     * W3+: adapter + device + host-fixed empty pipeline-layout.
     * `[method]gpu-device.create-pipeline-layout` (no Guest descriptor).
     */
    fun attachCreatePipelineLayout(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreatePipelineLayout(device: Int): Int =
                    bindings.deviceCreatePipelineLayout(
                        device,
                        PipelineLayoutDescriptor(bindGroupLayouts = emptyList()),
                    )
            },
        )
    }

    /** W3: adapter + device + encoder. Shared by flat `device-create-command-encoder` and `[method]gpu-device.create-command-encoder`. */
    fun attachCreateCommandEncoder(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreateCommandEncoder(device: Int): Int =
                    bindings.deviceCreateCommandEncoder(device)
            },
        )
    }

    /** W3: adapter + device + encoder + finish. Shared by flat `command-encoder-finish` and `[method]gpu-command-encoder.finish`. */
    fun attachCommandEncoderFinish(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreateCommandEncoder(device: Int): Int =
                    bindings.deviceCreateCommandEncoder(device)

                override fun commandEncoderFinish(encoder: Int): Int =
                    bindings.commandEncoderFinish(encoder)
            },
        )
    }

    /**
     * W3 slice: adapter + device + encoder + begin-render-pass-clear.
     *
     * Guest passes transitional stub view `23` (not a surface texture). After
     * [adapterRequestDevice] this attach creates a 1×1 Cpu offscreen color
     * TextureView and substitutes it so L2 sees a real handle. Shared by flat
     * `command-encoder-begin-render-pass-clear` and
     * `[method]gpu-command-encoder.begin-render-pass`. Not present / wasi-gfx.
     */
    fun attachBeginRenderPassClear(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        var colorView = 0
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int {
                    val device = bindings.adapterRequestDevice(adapter)
                    val texture =
                        bindings.deviceCreateTexture(
                            device,
                            TextureDescriptor(
                                size = Extent3D(width = 1, height = 1),
                                format = GpuTextureFormat.RGBA8_UNORM,
                                usage = GpuTextureUsage.RENDER_ATTACHMENT,
                            ),
                        )
                    colorView = bindings.textureCreateView(texture)
                    return device
                }

                override fun deviceCreateCommandEncoder(device: Int): Int =
                    bindings.deviceCreateCommandEncoder(device)

                override fun beginRenderPassClear(encoder: Int, view: Int): Int {
                    val resolved = if (colorView != 0) colorView else view
                    return bindings.commandEncoderBeginRenderPassClear(
                        encoder,
                        resolved,
                        CLEAR_R,
                        CLEAR_G,
                        CLEAR_B,
                        CLEAR_A,
                    )
                }
            },
        )
    }

    /**
     * W3 slice: adapter + device + encoder + begin-render-pass-clear + render-pass-end.
     *
     * Same Cpu offscreen TextureView substitution as [attachBeginRenderPassClear].
     * Shared by flat `render-pass-end` and `[method]gpu-render-pass-encoder.end`.
     */
    fun attachRenderPassEnd(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        var colorView = 0
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int {
                    val device = bindings.adapterRequestDevice(adapter)
                    val texture =
                        bindings.deviceCreateTexture(
                            device,
                            TextureDescriptor(
                                size = Extent3D(width = 1, height = 1),
                                format = GpuTextureFormat.RGBA8_UNORM,
                                usage = GpuTextureUsage.RENDER_ATTACHMENT,
                            ),
                        )
                    colorView = bindings.textureCreateView(texture)
                    return device
                }

                override fun deviceCreateCommandEncoder(device: Int): Int =
                    bindings.deviceCreateCommandEncoder(device)

                override fun beginRenderPassClear(encoder: Int, view: Int): Int {
                    val resolved = if (colorView != 0) colorView else view
                    return bindings.commandEncoderBeginRenderPassClear(
                        encoder,
                        resolved,
                        CLEAR_R,
                        CLEAR_G,
                        CLEAR_B,
                        CLEAR_A,
                    )
                }

                override fun renderPassEnd(pass: Int) {
                    bindings.renderPassEnd(pass)
                }
            },
        )
    }

    /** W3: adapter + device + queue + encoder + finish + submit1. Shared by flat `queue-submit1` and `[method]gpu-queue.submit`. */
    fun attachQueueSubmit1(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceGetQueue(device: Int): Int = bindings.deviceGetQueue(device)

                override fun deviceCreateCommandEncoder(device: Int): Int =
                    bindings.deviceCreateCommandEncoder(device)

                override fun commandEncoderFinish(encoder: Int): Int =
                    bindings.commandEncoderFinish(encoder)

                override fun queueSubmit1(queue: Int, commandBuffer: Int) {
                    bindings.queueSubmit1(queue, commandBuffer)
                }
            },
        )
    }

    /**
     * W3+: adapter + device + host-fixed 1×1 texture + create-view.
     * `[method]gpu-texture.create-view` (no Guest descriptor).
     */
    fun attachCreateTextureView(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreateTexture(device: Int): Int =
                    bindings.deviceCreateTexture(
                        device,
                        TextureDescriptor(
                            size = Extent3D(width = 1, height = 1),
                            format = GpuTextureFormat.RGBA8_UNORM,
                            usage = GpuTextureUsage.RENDER_ATTACHMENT,
                        ),
                    )

                override fun textureCreateView(texture: Int): Int =
                    bindings.textureCreateView(texture)
            },
        )
    }

    /**
     * W3+: adapter + device + queue + host-fixed create-buffer + write-buffer.
     * Guest passes stub buffer `31`; JNI ignores it and writes 4 host bytes.
     * `[method]gpu-queue.write-buffer` only (not proposal `list<u8>`).
     */
    fun attachWriteBuffer(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceGetQueue(device: Int): Int = bindings.deviceGetQueue(device)

                override fun deviceCreateBuffer(device: Int): Int =
                    bindings.deviceCreateBuffer(
                        device,
                        size = STUB_BUFFER_SIZE,
                        usage = GpuBufferUsage.COPY_DST or GpuBufferUsage.VERTEX,
                    )

                override fun queueWriteBuffer(queue: Int, buffer: Int) {
                    bindings.queueWriteBuffer(queue, buffer, 0L, STUB_BUFFER_BYTES)
                }
            },
        )
    }

    /** M4: clear→present subset for dedicated render smoke Guest. */
    fun attachRenderSmoke(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceGetQueue(device: Int): Int = bindings.deviceGetQueue(device)

                override fun createSurfaceFromNativeWindow(windowHandle: Long): Int =
                    bindings.createSurfaceFromNativeWindow(windowHandle)

                override fun surfaceConfigure(
                    surface: Int,
                    device: Int,
                    adapter: Int,
                    width: Int,
                    height: Int,
                ): Int = bindings.surfaceConfigure(surface, device, adapter, width, height)

                override fun surfaceGetCurrentTextureView(surface: Int): Int =
                    bindings.surfaceGetCurrentTextureView(surface)

                override fun deviceCreateCommandEncoder(device: Int): Int =
                    bindings.deviceCreateCommandEncoder(device)

                override fun beginRenderPassClear(encoder: Int, view: Int): Int =
                    bindings.commandEncoderBeginRenderPassClear(
                        encoder,
                        view,
                        CLEAR_R,
                        CLEAR_G,
                        CLEAR_B,
                        CLEAR_A,
                    )

                override fun renderPassEnd(pass: Int) {
                    bindings.renderPassEnd(pass)
                }

                override fun commandEncoderFinish(encoder: Int): Int =
                    bindings.commandEncoderFinish(encoder)

                override fun queueSubmit1(queue: Int, commandBuffer: Int) {
                    bindings.queueSubmit1(queue, commandBuffer)
                }

                override fun surfacePresent(surface: Int) {
                    bindings.surfacePresent(surface)
                }

                override fun surfaceUnconfigure(surface: Int) {
                    bindings.surfaceUnconfigure(surface)
                }
            },
        )
    }

    private const val CLEAR_R = 0.12f
    private const val CLEAR_G = 0.28f
    private const val CLEAR_B = 0.72f
    private const val CLEAR_A = 1.0f
    private const val STUB_BUFFER_SIZE = 4L
    private val STUB_BUFFER_BYTES = byteArrayOf(1, 2, 3, 4)
    private const val STUB_WGSL = "@compute @workgroup_size(1) fn main() {}"
}
