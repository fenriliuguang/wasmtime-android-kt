package io.github.fenriliuguang.wasmtime.android.webgpu

import io.github.fenriliuguang.wasi.webgpu.experimental.abicm.AbiCmHostBindings
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BindGroupDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.BindGroupLayoutDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ComputePipelineDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.PipelineLayoutDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.ProgrammableStage
import io.github.fenriliuguang.wasi.webgpu.experimental.host.Extent3D
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuHandle
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

    /**
     * W3+: adapter + device + empty BGL then empty bind-group.
     * `[method]gpu-device.create-bind-group` (no Guest descriptor).
     */
    fun attachCreateBindGroup(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreateBindGroup(device: Int): Int {
                    val layout = bindings.deviceCreateBindGroupLayout(
                        device,
                        BindGroupLayoutDescriptor(entries = emptyList()),
                    )
                    return bindings.deviceCreateBindGroup(
                        device,
                        BindGroupDescriptor(
                            layout = GpuHandle(layout),
                            entries = emptyList(),
                        ),
                    )
                }
            },
        )
    }

    /**
     * W3+: adapter + device + stub shader + triangle render pipeline.
     * `[method]gpu-device.create-render-pipeline` (no Guest descriptor).
     */
    fun attachCreateRenderPipeline(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreateRenderPipeline(device: Int): Int {
                    val shader = bindings.deviceCreateShaderModule(device, STUB_WGSL)
                    return bindings.deviceCreateRenderPipelineTriangle(
                        device,
                        shader,
                        GpuTextureFormat.RGBA8_UNORM,
                    )
                }
            },
        )
    }

    /**
     * W3+: adapter + device + stub shader + empty pipeline-layout + compute pipeline.
     * `[method]gpu-device.create-compute-pipeline` (no Guest descriptor; explicit layout).
     */
    fun attachCreateComputePipeline(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreateComputePipeline(device: Int): Int {
                    val shader = bindings.deviceCreateShaderModule(device, STUB_WGSL)
                    val layout = bindings.deviceCreatePipelineLayout(
                        device,
                        PipelineLayoutDescriptor(bindGroupLayouts = emptyList()),
                    )
                    return bindings.deviceCreateComputePipeline(
                        device,
                        ComputePipelineDescriptor(
                            layout = GpuHandle(layout),
                            compute = ProgrammableStage(
                                module = GpuHandle(shader),
                                entryPoint = "main",
                            ),
                        ),
                    )
                }
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
     * W3+: adapter + device + encoder + begin-compute-pass (host-default descriptor).
     * `[method]gpu-command-encoder.begin-compute-pass` (no Guest descriptor).
     */
    fun attachBeginComputePass(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreateCommandEncoder(device: Int): Int =
                    bindings.deviceCreateCommandEncoder(device)

                override fun beginComputePass(encoder: Int): Int =
                    bindings.commandEncoderBeginComputePass(encoder)
            },
        )
    }

    /**
     * W3+: adapter + device + encoder + host-fixed 4-byte copy-buffer-to-buffer.
     * Guest passes stub source/destination `31`; JNI ignores them.
     * `[method]gpu-command-encoder.copy-buffer-to-buffer`.
     */
    fun attachCopyBufferToBuffer(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        var device = 0
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int {
                    device = bindings.adapterRequestDevice(adapter)
                    return device
                }

                override fun deviceCreateCommandEncoder(device: Int): Int =
                    bindings.deviceCreateCommandEncoder(device)

                override fun commandEncoderCopyBufferToBuffer(encoder: Int) {
                    val source =
                        bindings.deviceCreateBuffer(
                            device,
                            size = STUB_BUFFER_SIZE,
                            usage = GpuBufferUsage.COPY_SRC,
                        )
                    val destination =
                        bindings.deviceCreateBuffer(
                            device,
                            size = STUB_BUFFER_SIZE,
                            usage = GpuBufferUsage.COPY_DST,
                        )
                    bindings.commandEncoderCopyBufferToBuffer(
                        encoder,
                        source,
                        0L,
                        destination,
                        0L,
                        STUB_BUFFER_SIZE,
                    )
                }
            },
        )
    }

    /**
     * W3+: adapter + device + encoder + begin-compute-pass + compute-pass-end.
     * `[method]gpu-compute-pass-encoder.end` (Guest stub pass ignored).
     */
    fun attachComputePassEnd(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreateCommandEncoder(device: Int): Int =
                    bindings.deviceCreateCommandEncoder(device)

                override fun beginComputePass(encoder: Int): Int =
                    bindings.commandEncoderBeginComputePass(encoder)

                override fun computePassEnd(pass: Int) {
                    bindings.computePassEnd(pass)
                }
            },
        )
    }

    /**
     * W3+: adapter + device + encoder + begin-compute-pass + host-fixed
     * compute pipeline set-pipeline.
     * Guest passes stub pipeline `73`; JNI ignores it.
     * `[method]gpu-compute-pass-encoder.set-pipeline`.
     */
    fun attachComputePassSetPipeline(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        var device = 0
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int {
                    device = bindings.adapterRequestDevice(adapter)
                    return device
                }

                override fun deviceCreateCommandEncoder(device: Int): Int =
                    bindings.deviceCreateCommandEncoder(device)

                override fun beginComputePass(encoder: Int): Int =
                    bindings.commandEncoderBeginComputePass(encoder)

                override fun computePassSetPipeline(pass: Int) {
                    val shader = bindings.deviceCreateShaderModule(device, STUB_WGSL)
                    val layout = bindings.deviceCreatePipelineLayout(
                        device,
                        PipelineLayoutDescriptor(bindGroupLayouts = emptyList()),
                    )
                    val pipeline = bindings.deviceCreateComputePipeline(
                        device,
                        ComputePipelineDescriptor(
                            layout = GpuHandle(layout),
                            compute = ProgrammableStage(
                                module = GpuHandle(shader),
                                entryPoint = "main",
                            ),
                        ),
                    )
                    bindings.computePassSetPipeline(pass, pipeline)
                }
            },
        )
    }

    /**
     * W3+: adapter + device + encoder + begin-compute-pass + host-fixed
     * empty bind-group at index 0.
     * Guest passes stub bind-group `67`; JNI ignores it.
     * Cpu only accepts bind-group index 0.
     * `[method]gpu-compute-pass-encoder.set-bind-group`.
     */
    fun attachComputePassSetBindGroup(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        var device = 0
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int {
                    device = bindings.adapterRequestDevice(adapter)
                    return device
                }

                override fun deviceCreateCommandEncoder(device: Int): Int =
                    bindings.deviceCreateCommandEncoder(device)

                override fun beginComputePass(encoder: Int): Int =
                    bindings.commandEncoderBeginComputePass(encoder)

                override fun computePassSetBindGroup(pass: Int) {
                    val bgl = bindings.deviceCreateBindGroupLayout(
                        device,
                        BindGroupLayoutDescriptor(entries = emptyList()),
                    )
                    val bindGroup = bindings.deviceCreateBindGroup(
                        device,
                        BindGroupDescriptor(
                            layout = GpuHandle(bgl),
                            entries = emptyList(),
                        ),
                    )
                    bindings.computePassSetBindGroup(pass, 0, bindGroup)
                }
            },
        )
    }

    /**
     * W3+: adapter + device + encoder + begin-compute-pass + host-fixed
     * set-pipeline + empty bind-group + dispatch(1,1,1).
     * Guest workgroup counts ignored. Cpu requires pipeline and bind-group 0.
     * `[method]gpu-compute-pass-encoder.dispatch-workgroups`.
     */
    fun attachComputePassDispatchWorkgroups(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        var device = 0
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int {
                    device = bindings.adapterRequestDevice(adapter)
                    return device
                }

                override fun deviceCreateCommandEncoder(device: Int): Int =
                    bindings.deviceCreateCommandEncoder(device)

                override fun beginComputePass(encoder: Int): Int =
                    bindings.commandEncoderBeginComputePass(encoder)

                override fun computePassDispatchWorkgroups(pass: Int) {
                    val shader = bindings.deviceCreateShaderModule(device, STUB_WGSL)
                    val layout = bindings.deviceCreatePipelineLayout(
                        device,
                        PipelineLayoutDescriptor(bindGroupLayouts = emptyList()),
                    )
                    val pipeline = bindings.deviceCreateComputePipeline(
                        device,
                        ComputePipelineDescriptor(
                            layout = GpuHandle(layout),
                            compute = ProgrammableStage(
                                module = GpuHandle(shader),
                                entryPoint = "main",
                            ),
                        ),
                    )
                    val bgl = bindings.deviceCreateBindGroupLayout(
                        device,
                        BindGroupLayoutDescriptor(entries = emptyList()),
                    )
                    val bindGroup = bindings.deviceCreateBindGroup(
                        device,
                        BindGroupDescriptor(
                            layout = GpuHandle(bgl),
                            entries = emptyList(),
                        ),
                    )
                    bindings.computePassSetPipeline(pass, pipeline)
                    bindings.computePassSetBindGroup(pass, 0, bindGroup)
                    bindings.computePassDispatchWorkgroups(pass, 1, 1, 1)
                }
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

    /**
     * W3+: adapter + device + encoder + begin-render-pass-clear + host-fixed
     * triangle pipeline set-pipeline.
     * Same Cpu offscreen TextureView substitution as [attachBeginRenderPassClear].
     * Guest stub pipeline `71` ignored.
     * `[method]gpu-render-pass-encoder.set-pipeline`.
     */
    fun attachRenderPassSetPipeline(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        var colorView = 0
        var device = 0
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int {
                    device = bindings.adapterRequestDevice(adapter)
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

                override fun renderPassSetPipeline(pass: Int) {
                    val shader = bindings.deviceCreateShaderModule(device, STUB_WGSL)
                    val pipeline = bindings.deviceCreateRenderPipelineTriangle(
                        device,
                        shader,
                        GpuTextureFormat.RGBA8_UNORM,
                    )
                    bindings.renderPassSetPipeline(pass, pipeline)
                }
            },
        )
    }

    /**
     * W3+: adapter + device + encoder + begin-render-pass-clear + host-fixed
     * triangle pipeline set-pipeline + draw(3).
     * Same Cpu offscreen TextureView substitution as [attachBeginRenderPassClear].
     * Guest vertexCount ignored. `[method]gpu-render-pass-encoder.draw`.
     */
    fun attachRenderPassDraw(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        var colorView = 0
        var device = 0
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int {
                    device = bindings.adapterRequestDevice(adapter)
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

                override fun renderPassDraw(pass: Int) {
                    val shader = bindings.deviceCreateShaderModule(device, STUB_WGSL)
                    val pipeline = bindings.deviceCreateRenderPipelineTriangle(
                        device,
                        shader,
                        GpuTextureFormat.RGBA8_UNORM,
                    )
                    bindings.renderPassSetPipeline(pass, pipeline)
                    bindings.renderPassDraw(pass, 3)
                }
            },
        )
    }

    /**
     * W3+: adapter + device + encoder + begin-render-pass-clear + host-fixed
     * empty bind-group at index 0.
     * Same Cpu offscreen TextureView substitution as [attachBeginRenderPassClear].
     * Guest passes stub bind-group `67`; JNI ignores it.
     * `[method]gpu-render-pass-encoder.set-bind-group`.
     */
    fun attachRenderPassSetBindGroup(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        var colorView = 0
        var device = 0
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int {
                    device = bindings.adapterRequestDevice(adapter)
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

                override fun renderPassSetBindGroup(pass: Int) {
                    val bgl = bindings.deviceCreateBindGroupLayout(
                        device,
                        BindGroupLayoutDescriptor(entries = emptyList()),
                    )
                    val bindGroup = bindings.deviceCreateBindGroup(
                        device,
                        BindGroupDescriptor(
                            layout = GpuHandle(bgl),
                            entries = emptyList(),
                        ),
                    )
                    bindings.renderPassSetBindGroup(pass, 0, bindGroup)
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

    /**
     * W3+: adapter + device + queue + host-fixed 1×1 COPY_DST texture write.
     * `[method]gpu-queue.write-texture` (Guest texture u32 ignored).
     */
    fun attachWriteTexture(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceGetQueue(device: Int): Int = bindings.deviceGetQueue(device)

                override fun deviceCreateTexture(device: Int): Int =
                    bindings.deviceCreateTexture(
                        device,
                        TextureDescriptor(
                            size = Extent3D(width = 1, height = 1),
                            format = GpuTextureFormat.RGBA8_UNORM,
                            usage = GpuTextureUsage.COPY_DST or GpuTextureUsage.TEXTURE_BINDING,
                        ),
                    )

                override fun queueWriteTexture(queue: Int, texture: Int) {
                    bindings.queueWriteTexture(
                        queue,
                        texture,
                        STUB_TEXTURE_BYTES,
                        width = 1,
                        height = 1,
                        bytesPerRow = 4,
                    )
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
    private val STUB_TEXTURE_BYTES = byteArrayOf(1, 2, 3, 4)
    private const val STUB_WGSL = "@compute @workgroup_size(1) fn main() {}"
}
