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
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuMapMode
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuTextureFormat
import io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuTextureUsage
import io.github.fenriliuguang.wasi.webgpu.experimental.host.SamplerDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.TextureDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.TextureViewDescriptor
import io.github.fenriliuguang.wasi.webgpu.experimental.host.WasiWebGpuHost
import io.github.fenriliuguang.wasmtime.android.Store
import io.github.fenriliuguang.wasmtime.android.api.ExperimentalHostCallbacks

/**
 * Wire an in-tree L2 [WasiWebGpuHost] into L1 store callbacks.
 *
 * Lives in `:host-dawn` so `:runtime-jni` does not depend on Dawn types.
 * Slice `attach*` helpers keep host-fixed descriptors for instruments.
 * Product code prefers [io.github.fenriliuguang.wasmtime.android.host.dawn.GpuBackends].
 *
 * Flat u32-rep imports (legacy); not full WIT resource method names.
 */
object ExperimentalWebGpuBridge {
    /**
     * W1/W2 flat `request-adapter` and S2 `[method]gpu.request-adapter` share this
     * L2 callback. `get-gpu` is a test constructor (no Kotlin).
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
     * L2 callbacks). S3 `[method]gpu-adapter.request-device` shares this attach:
     * `get-adapter` is host-only (no Kotlin); the method then calls L2
     * `requestAdapter` (when adapter.rep is 0) + `adapterRequestDevice`, and
     * returns `result<own<gpu-device>, request-device-error>`.
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

    /** Adapter + device + queue. Shared by flat `device-get-queue` and S1 `[method]gpu-device.queue` (`get-device` is test-only). */
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
     * S4: adapter + device + `[method]gpu-device.create-buffer` with Guest
     * `gpu-buffer-descriptor` size/usage forwarded to L2 (mapped/label unused).
     */
    fun attachCreateBuffer(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreateBufferDescribed(
                    device: Int,
                    size: Long,
                    usage: Int,
                ): Int = bindings.deviceCreateBuffer(device, size = size, usage = usage)
            },
        )
    }

    /**
     * S6+: adapter + device + host-fixed MAP_READ buffer + map-async.
     * Guest stub buffer ignored; guest `gpu-map-mode` / offset / size forwarded.
     * True CM async (`func_wrap_concurrent`). `[method]gpu-buffer.map-async`.
     */
    fun attachBufferMapAsync(store: Store, host: WasiWebGpuHost) {
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
                        usage = GpuBufferUsage.MAP_READ or GpuBufferUsage.COPY_DST,
                    )

                override fun bufferMapAsyncDescribed(
                    buffer: Int,
                    mode: Int,
                    offset: Long,
                    size: Long,
                ) {
                    bindings.bufferMapAsync(buffer, mode, offset, size)
                }
            },
        )
    }

    /**
     * W3+ / S6+: adapter + device + host-fixed MAP_READ buffer + map then unmap.
     * Guest WIT `result<_, unmap-error>` (JNI still host-fixed).
     * `[method]gpu-buffer.unmap`.
     */
    fun attachBufferUnmap(store: Store, host: WasiWebGpuHost) {
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
                        usage = GpuBufferUsage.MAP_READ or GpuBufferUsage.COPY_DST,
                    )

                override fun bufferUnmap(buffer: Int) {
                    bindings.bufferMapAsync(
                        buffer,
                        GpuMapMode.READ,
                        0,
                        STUB_BUFFER_SIZE,
                    )
                    bindings.bufferUnmap(buffer)
                }
            },
        )
    }

    /**
     * S6+: `[method]gpu-buffer.get-mapped-range-get-with-copy` /
     * `[method]gpu-buffer.get-mapped-range-set-with-copy`.
     * Native lifts guest types and returns empty list / ok; L2 still unused (no new JNI).
     */
    fun attachGetMappedRange(store: Store, @Suppress("UNUSED_PARAMETER") host: WasiWebGpuHost) {
        store.setExperimentalHost(object : ExperimentalHostCallbacks {})
    }
    fun attachCreateTexture(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreateTextureDescribed(
                    device: Int,
                    width: Int,
                    height: Int,
                    depth: Int,
                    format: Int,
                    usage: Int,
                ): Int =
                    bindings.deviceCreateTexture(
                        device,
                        TextureDescriptor(
                            size = Extent3D(
                                width = width,
                                height = height,
                                depthOrArrayLayers = depth,
                            ),
                            format = format,
                            usage = usage,
                        ),
                    )
            },
        )
    }

    /**
     * L2: adapter + device + `[method]gpu-device.create-sampler` with Guest
     * `gpu-sampler-descriptor` mag/min filter and address-mode-u forwarded to L2.
     */
    fun attachCreateSampler(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        store.setExperimentalHost(
            object : ExperimentalHostCallbacks {
                override fun requestAdapter(): Int = bindings.requestAdapter()

                override fun adapterRequestDevice(adapter: Int): Int =
                    bindings.adapterRequestDevice(adapter)

                override fun deviceCreateSamplerDescribed(
                    device: Int,
                    magFilter: Int,
                    minFilter: Int,
                    addressModeU: Int,
                ): Int =
                    bindings.deviceCreateSampler(
                        device,
                        SamplerDescriptor(
                            magFilter = magFilter,
                            minFilter = minFilter,
                            addressModeU = addressModeU,
                        ),
                    )
            },
        )
    }

    /**
     * W3+: adapter + device + create-shader-module. Guest passes
     * `gpu-shader-module-descriptor`; L2 still host-fixed stub WGSL.
     * `[method]gpu-device.create-shader-module`.
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
     * W3+: adapter + device + empty bind-group-layout.
     * Guest passes `gpu-bind-group-layout-descriptor`; L2 still host-fixed empty entries.
     * `[method]gpu-device.create-bind-group-layout`.
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
     * W3+: adapter + device + empty pipeline-layout.
     * Guest passes `gpu-pipeline-layout-descriptor`; L2 still host-fixed empty bind-group-layouts.
     * `[method]gpu-device.create-pipeline-layout`.
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
     * Guest passes `gpu-bind-group-descriptor`; L2 still host-fixed empty BGL + empty entries.
     * `[method]gpu-device.create-bind-group`.
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
     * `[method]gpu-device.create-render-pipeline` (guest `gpu-render-pipeline-descriptor`;
     * L2 still host-fixed stub shader + triangle).
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
     * S6+: same L2 as [attachCreateRenderPipeline]; product guest is
     * `[method]gpu-device.create-render-pipeline-async`
     * (`result<own<pipeline>, create-pipeline-error>`; true CM async).
     */
    fun attachCreateRenderPipelineAsync(store: Store, host: WasiWebGpuHost) {
        attachCreateRenderPipeline(store, host)
    }

    /**
     * W3+: adapter + device + stub shader + empty pipeline-layout + compute pipeline.
     * `[method]gpu-device.create-compute-pipeline` (guest `gpu-compute-pipeline-descriptor`;
     * L2 still host-fixed stub shader + explicit empty layout).
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

    /**
     * S6+: same L2 as [attachCreateComputePipeline]; product guest is
     * `[method]gpu-device.create-compute-pipeline-async`
     * (`result<own<pipeline>, create-pipeline-error>`; true CM async).
     */
    fun attachCreateComputePipelineAsync(store: Store, host: WasiWebGpuHost) {
        attachCreateComputePipeline(store, host)
    }

    /** W3/S6: adapter + device + encoder. Shared by flat `device-create-command-encoder` and `[method]gpu-device.create-command-encoder` (S6 own; descriptor still none). */
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

    /** W3/S7: adapter + device + encoder + finish. Shared by flat `command-encoder-finish` and `[method]gpu-command-encoder.finish` (S7 own; descriptor still none). */
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
     * L2: adapter + device + encoder + `[method]gpu-command-encoder.copy-buffer-to-buffer`
     * / `clear-buffer` with Guest buffer reps (0 → stub 4-byte) and offsets/size.
     * Texture-copy described JNI (buffer/texture reps 0 → aligned stub) shares this attach.
     */
    fun attachCopyBufferToBuffer(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        var device = 0
        fun stubTexelBuffer(usage: Int): Int =
            bindings.deviceCreateBuffer(
                device,
                size = STUB_TEXEL_COPY_BYTES,
                usage = usage,
            )
        fun stubTexelTexture(width: Int, height: Int, depth: Int): Int =
            bindings.deviceCreateTexture(
                device,
                TextureDescriptor(
                    size = Extent3D(
                        width = width.coerceAtLeast(1),
                        height = height.coerceAtLeast(1),
                        depthOrArrayLayers = depth.coerceAtLeast(1),
                    ),
                    format = GpuTextureFormat.RGBA8_UNORM,
                    usage = GpuTextureUsage.COPY_SRC or GpuTextureUsage.COPY_DST,
                ),
            )
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

                override fun commandEncoderCopyBufferToBufferDescribed(
                    encoder: Int,
                    source: Int,
                    sourceOffset: Long,
                    destination: Int,
                    destinationOffset: Long,
                    size: Long,
                ) {
                    val src =
                        if (source != 0) {
                            source
                        } else {
                            bindings.deviceCreateBuffer(
                                device,
                                size = STUB_BUFFER_SIZE,
                                usage = GpuBufferUsage.COPY_SRC,
                            )
                        }
                    val dst =
                        if (destination != 0) {
                            destination
                        } else {
                            bindings.deviceCreateBuffer(
                                device,
                                size = STUB_BUFFER_SIZE,
                                usage = GpuBufferUsage.COPY_DST,
                            )
                        }
                    val copySize = if (size != 0L) size else STUB_BUFFER_SIZE
                    bindings.commandEncoderCopyBufferToBuffer(
                        encoder,
                        src,
                        sourceOffset,
                        dst,
                        destinationOffset,
                        copySize,
                    )
                }

                override fun commandEncoderClearBufferDescribed(
                    encoder: Int,
                    buffer: Int,
                    offset: Long,
                    size: Long,
                ) {
                    val buf =
                        if (buffer != 0) {
                            buffer
                        } else {
                            bindings.deviceCreateBuffer(
                                device,
                                size = STUB_BUFFER_SIZE,
                                usage = GpuBufferUsage.COPY_DST,
                            )
                        }
                    val clearSize = if (size != 0L) size else STUB_BUFFER_SIZE
                    bindings.commandEncoderClearBuffer(encoder, buf, offset, clearSize)
                }

                override fun commandEncoderCopyBufferToTextureDescribed(
                    encoder: Int,
                    source: Int,
                    destination: Int,
                    width: Int,
                    height: Int,
                    depth: Int,
                ) {
                    val src =
                        if (source != 0) {
                            source
                        } else {
                            stubTexelBuffer(GpuBufferUsage.COPY_SRC)
                        }
                    val dst =
                        if (destination != 0) {
                            destination
                        } else {
                            stubTexelTexture(width, height, depth)
                        }
                    bindings.commandEncoderCopyBufferToTexture(
                        encoder,
                        src,
                        dst,
                        width,
                        height,
                        depth,
                    )
                }

                override fun commandEncoderCopyTextureToBufferDescribed(
                    encoder: Int,
                    source: Int,
                    destination: Int,
                    width: Int,
                    height: Int,
                    depth: Int,
                ) {
                    val src =
                        if (source != 0) {
                            source
                        } else {
                            stubTexelTexture(width, height, depth)
                        }
                    val dst =
                        if (destination != 0) {
                            destination
                        } else {
                            stubTexelBuffer(GpuBufferUsage.COPY_DST)
                        }
                    bindings.commandEncoderCopyTextureToBuffer(
                        encoder,
                        src,
                        dst,
                        width,
                        height,
                        depth,
                    )
                }

                override fun commandEncoderCopyTextureToTextureDescribed(
                    encoder: Int,
                    source: Int,
                    destination: Int,
                    width: Int,
                    height: Int,
                    depth: Int,
                ) {
                    val src =
                        if (source != 0) {
                            source
                        } else {
                            stubTexelTexture(width, height, depth)
                        }
                    val dst =
                        if (destination != 0) {
                            destination
                        } else {
                            stubTexelTexture(width, height, depth)
                        }
                    bindings.commandEncoderCopyTextureToTexture(
                        encoder,
                        src,
                        dst,
                        width,
                        height,
                        depth,
                    )
                }
            },
        )
    }

    /**
     * L2: same attach as [attachCopyBufferToBuffer]; product guests are
     * `copy-buffer-to-texture` / `copy-texture-to-buffer` /
     * `copy-texture-to-texture` (described copy-size JNI) plus L2 `clear-buffer`.
     */
    fun attachCommandEncoderCopy(store: Store, host: WasiWebGpuHost) {
        attachCopyBufferToBuffer(store, host)
    }

    /**
     * S6+: `[method]gpu-command-encoder.resolve-query-set` /
     * `push-debug-group` / `pop-debug-group` / `insert-debug-marker`.
     * Native lifts guest args; L2 unused (no new JNI).
     */
    fun attachCommandEncoderState(store: Store, @Suppress("UNUSED_PARAMETER") host: WasiWebGpuHost) {
        store.setExperimentalHost(object : ExperimentalHostCallbacks {})
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
     * W3+ / S6+: adapter + device + encoder + begin-compute-pass + host-fixed
     * compute pipeline set-pipeline.
     * Guest passes WIT `borrow<gpu-compute-pipeline>` (JNI still host-fixed).
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
     * W3+ / S6+: adapter + device + encoder + begin-compute-pass + host-fixed
     * empty bind-group at index 0.
     * Guest passes WIT option bind-group (JNI still host-fixed).
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
     * W3+ / S6+: adapter + device + encoder + begin-compute-pass + host-fixed
     * set-pipeline + empty bind-group + dispatch(1,1,1).
     * Guest WIT option counts (JNI still host-fixed). Cpu requires pipeline and bind-group 0.
     * `[method]gpu-compute-pass-encoder.dispatch-workgroups`.
     * Also used by `dispatch-workgroups-indirect` (L2 still host-fixed 1×1×1).
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
     * S6+: same L2 as [attachComputePassDispatchWorkgroups]; product guest is
     * `[method]gpu-compute-pass-encoder.dispatch-workgroups-indirect`.
     */
    fun attachComputePassDispatchWorkgroupsIndirect(store: Store, host: WasiWebGpuHost) {
        attachComputePassDispatchWorkgroups(store, host)
    }

    /**
     * S6+: `[method]gpu-compute-pass-encoder.set-immediates` /
     * `push-debug-group` / `pop-debug-group` / `insert-debug-marker`.
     * Native lifts guest args; L2 unused (no new JNI).
     */
    fun attachComputePassState(store: Store, @Suppress("UNUSED_PARAMETER") host: WasiWebGpuHost) {
        store.setExperimentalHost(object : ExperimentalHostCallbacks {})
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
     * L2: adapter + device + encoder + begin-render-pass-clear + render-pass-end.
     * Guest pass rep is forwarded when non-zero. Same Cpu offscreen TextureView
     * substitution as [attachBeginRenderPassClear]. Shared by flat `render-pass-end`
     * and `[method]gpu-render-pass-encoder.end`.
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

                override fun renderPassEndDescribed(pass: Int) {
                    bindings.renderPassEnd(pass)
                }
            },
        )
    }

    /**
     * W3+ / S6+: adapter + device + encoder + begin-render-pass-clear + host-fixed
     * triangle pipeline set-pipeline.
     * Same Cpu offscreen TextureView substitution as [attachBeginRenderPassClear].
     * Guest passes WIT `borrow<gpu-render-pipeline>` (JNI still host-fixed).
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
     * L2: adapter + device + encoder + begin-render-pass-clear + triangle pipeline
     * + described draw / draw-indexed (guest counts). Host-fixed [renderPassDraw]
     * remains for indirect attaches. Same Cpu offscreen TextureView substitution as
     * [attachBeginRenderPassClear]. `[method]gpu-render-pass-encoder.draw` /
     * `draw-indexed`.
     */
    fun attachRenderPassDraw(store: Store, host: WasiWebGpuHost) {
        val bindings = AbiCmHostBindings(host)
        var colorView = 0
        var device = 0
        fun bindTrianglePipeline(pass: Int) {
            val shader = bindings.deviceCreateShaderModule(device, STUB_WGSL)
            val pipeline = bindings.deviceCreateRenderPipelineTriangle(
                device,
                shader,
                GpuTextureFormat.RGBA8_UNORM,
            )
            bindings.renderPassSetPipeline(pass, pipeline)
        }
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
                    bindTrianglePipeline(pass)
                    bindings.renderPassDraw(pass, 3)
                }

                override fun renderPassDrawDescribed(
                    pass: Int,
                    vertexCount: Int,
                    instanceCount: Int,
                    firstVertex: Int,
                    firstInstance: Int,
                ) {
                    bindTrianglePipeline(pass)
                    bindings.renderPassDraw(
                        pass,
                        vertexCount,
                        instanceCount,
                        firstVertex,
                        firstInstance,
                    )
                }

                override fun renderPassDrawIndexedDescribed(
                    pass: Int,
                    indexCount: Int,
                    instanceCount: Int,
                    firstIndex: Int,
                    baseVertex: Int,
                    firstInstance: Int,
                ) {
                    bindTrianglePipeline(pass)
                    bindings.renderPassDrawIndexed(
                        pass,
                        indexCount,
                        instanceCount,
                        firstIndex,
                        baseVertex,
                        firstInstance,
                    )
                }

                override fun renderPassEndDescribed(pass: Int) {
                    bindings.renderPassEnd(pass)
                }
            },
        )
    }

    /**
     * W3+ / S6+: adapter + device + encoder + begin-render-pass-clear + host-fixed
     * empty bind-group at index 0.
     * Same Cpu offscreen TextureView substitution as [attachBeginRenderPassClear].
     * Guest passes WIT option bind-group (JNI still host-fixed).
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

    /**
     * W3+ / S6+: adapter + device + encoder + begin-render-pass-clear + host-fixed
     * VERTEX buffer at slot 0.
     * Same Cpu offscreen TextureView substitution as [attachBeginRenderPassClear].
     * Guest passes WIT option buffer (JNI still host-fixed).
     * `[method]gpu-render-pass-encoder.set-vertex-buffer`.
     */
    fun attachRenderPassSetVertexBuffer(store: Store, host: WasiWebGpuHost) {
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

                override fun renderPassSetVertexBuffer(pass: Int) {
                    val buffer = bindings.deviceCreateBuffer(
                        device,
                        size = STUB_BUFFER_SIZE,
                        usage = GpuBufferUsage.VERTEX,
                    )
                    bindings.renderPassSetVertexBuffer(pass, 0, buffer, 0, STUB_BUFFER_SIZE)
                }
            },
        )
    }

    /**
     * S6+: `[method]gpu-render-pass-encoder.set-viewport` / `set-scissor-rect` /
     * `set-blend-constant` / `set-stencil-reference` / `push-debug-group` /
     * `pop-debug-group` / `insert-debug-marker` / `begin-occlusion-query` /
     * `end-occlusion-query` / `execute-bundles` / `set-immediates`.
     * Native lifts guest args; L2 unused (no new JNI).
     */
    fun attachRenderPassState(store: Store, @Suppress("UNUSED_PARAMETER") host: WasiWebGpuHost) {
        store.setExperimentalHost(object : ExperimentalHostCallbacks {})
    }

    /**
     * S6+: `[method]gpu-render-bundle-encoder.finish` / `set-pipeline` /
     * `set-bind-group` / `draw` / `set-index-buffer` / `set-vertex-buffer` /
     * `draw-indexed` / `draw-indirect` / `draw-indexed-indirect` /
     * `push-debug-group` / `pop-debug-group` / `insert-debug-marker` /
     * `set-immediates`. Native lifts guest args; L2 unused (no new JNI).
     */
    fun attachRenderBundleState(store: Store, @Suppress("UNUSED_PARAMETER") host: WasiWebGpuHost) {
        store.setExperimentalHost(object : ExperimentalHostCallbacks {})
    }

    /**
     * S6+: `[method]gpu-device.create-render-bundle-encoder` /
     * `create-query-set` / `destroy` / `[method]gpu-buffer.destroy` /
     * `[method]gpu-texture.destroy` / `[method]gpu-query-set.destroy` /
     * `type` / `count`. Native lifts guest args; L2 unused (no new JNI).
     */
    fun attachDeviceQueryAndDestroy(
        store: Store,
        @Suppress("UNUSED_PARAMETER") host: WasiWebGpuHost,
    ) {
        store.setExperimentalHost(object : ExperimentalHostCallbacks {})
    }

    /**
     * S6+: `[method]gpu-adapter.features` / `limits` / `info` and
     * `[method]gpu-adapter-info.vendor` / `architecture` / `device` /
     * `description` / `subgroup-min-size` / `subgroup-max-size` /
     * `is-fallback-adapter`, and `[method]gpu-supported-limits.max-*`
     * getters. Native lifts; L2 unused (no new JNI).
     */
    fun attachAdapterInfo(store: Store, @Suppress("UNUSED_PARAMETER") host: WasiWebGpuHost) {
        store.setExperimentalHost(object : ExperimentalHostCallbacks {})
    }

    /**
     * S6+: `[method]gpu-bind-group.label` / `set-label`,
     * `[method]gpu-bind-group-layout.label` / `set-label`, and
     * `[method]gpu-buffer.label` / `set-label` / `size` / `usage` / `map-state`.
     * Native lifts; L2 unused (no new JNI).
     */
    fun attachBindGroupBufferLabel(
        store: Store,
        @Suppress("UNUSED_PARAMETER") host: WasiWebGpuHost,
    ) {
        store.setExperimentalHost(object : ExperimentalHostCallbacks {})
    }

    /**
     * S6+: `[method]gpu-texture.width` / `height` / `depth-or-array-layers` /
     * `mip-level-count` / `sample-count` / `dimension` / `format` / `usage` /
     * `texture-binding-view-dimension` / `label` / `set-label`.
     * Native lifts; L2 unused (no new JNI).
     */
    fun attachTextureInfo(store: Store, @Suppress("UNUSED_PARAMETER") host: WasiWebGpuHost) {
        store.setExperimentalHost(object : ExperimentalHostCallbacks {})
    }

    /**
     * S6+: `[constructor]record-gpu-pipeline-constant-value` and
     * `[method]record-gpu-pipeline-constant-value.add` / `get` / `has` /
     * `remove` / `keys` / `values` / `entries`. Native lifts; L2 unused (no new JNI).
     */
    fun attachRecordPipelineConstantValue(
        store: Store,
        @Suppress("UNUSED_PARAMETER") host: WasiWebGpuHost,
    ) {
        store.setExperimentalHost(object : ExperimentalHostCallbacks {})
    }

    /**
     * S6+: `[constructor]record-option-gpu-size64` and
     * `[method]record-option-gpu-size64.add` / `get` / `has` /
     * `remove` / `keys` / `values` / `entries`. Native lifts; L2 unused (no new JNI).
     */
    fun attachRecordOptionGpuSize64(
        store: Store,
        @Suppress("UNUSED_PARAMETER") host: WasiWebGpuHost,
    ) {
        store.setExperimentalHost(object : ExperimentalHostCallbacks {})
    }

    /**
     * S6+: `[method]gpu-command-buffer.label` / `set-label`,
     * `[method]gpu-command-encoder.label` / `set-label`,
     * `[method]gpu-compilation-info.messages`,
     * `[method]gpu-compilation-message.message` / `type` / `line-num` /
     * `line-pos` / `offset` / `length`, and
     * `[method]gpu-shader-module.get-compilation-info` / `label` / `set-label`.
     * Native lifts; L2 unused (no new JNI).
     */
    fun attachCommandCompilationLabel(
        store: Store,
        @Suppress("UNUSED_PARAMETER") host: WasiWebGpuHost,
    ) {
        store.setExperimentalHost(object : ExperimentalHostCallbacks {})
    }

    /**
     * S6+: `[method]gpu-compute-pass-encoder.label` / `set-label` and
     * `[method]gpu-compute-pipeline.label` / `set-label` /
     * `get-bind-group-layout`. Native lifts; L2 unused (no new JNI).
     */
    fun attachComputePassPipelineLabel(
        store: Store,
        @Suppress("UNUSED_PARAMETER") host: WasiWebGpuHost,
    ) {
        store.setExperimentalHost(object : ExperimentalHostCallbacks {})
    }

    /**
     * S6+: `[method]gpu-render-bundle.label` / `set-label`,
     * `[method]gpu-render-bundle-encoder.label` / `set-label`,
     * `[method]gpu-render-pass-encoder.label` / `set-label`, and
     * `[method]gpu-render-pipeline.label` / `set-label` /
     * `get-bind-group-layout`. Native lifts; L2 unused (no new JNI).
     */
    fun attachRenderBundlePassPipelineLabel(
        store: Store,
        @Suppress("UNUSED_PARAMETER") host: WasiWebGpuHost,
    ) {
        store.setExperimentalHost(object : ExperimentalHostCallbacks {})
    }

    /**
     * S6+: `[method]gpu-device.adapter-info` / `features` / `limits` / `label` /
     * `set-label` / `lost` / `push-error-scope` / `pop-error-scope` /
     * `on-uncaptured-error` and `[method]gpu-device-lost-info.reason` / `message`,
     * `[method]gpu-uncaptured-error-event.error`.
     * Native lifts; L2 unused (no new JNI).
     */
    fun attachDeviceInfoError(
        store: Store,
        @Suppress("UNUSED_PARAMETER") host: WasiWebGpuHost,
    ) {
        store.setExperimentalHost(object : ExperimentalHostCallbacks {})
    }

    /**
     * S6+: same L2 as [attachRenderPassSetVertexBuffer]; product guest is
     * `[method]gpu-render-pass-encoder.set-index-buffer`.
     */
    fun attachRenderPassSetIndexBuffer(store: Store, host: WasiWebGpuHost) {
        attachRenderPassSetVertexBuffer(store, host)
    }

    /**
     * L2: same attach as [attachRenderPassDraw]; product guests are
     * `draw-indexed` (described) / `draw-indirect` / `draw-indexed-indirect`
     * (host-fixed draw JNI).
     */
    fun attachRenderPassDrawIndexed(store: Store, host: WasiWebGpuHost) {
        attachRenderPassDraw(store, host)
    }

    /** W3/S5: adapter + device + queue + encoder + finish + submit1. Shared by flat `queue-submit1` and `[method]gpu-queue.submit` (S5 list still host-fixed L2). */
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
     * L2: adapter + device + host-fixed 1×1 texture + `[method]gpu-texture.create-view`
     * with Guest `gpu-texture-view-descriptor` dimension/aspect forwarded to L2.
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

                override fun textureCreateViewDescribed(
                    texture: Int,
                    dimension: Int,
                    aspect: Int,
                ): Int =
                    bindings.textureCreateView(
                        texture,
                        TextureViewDescriptor(
                            dimension = dimension,
                            aspect = aspect,
                        ),
                    )
            },
        )
    }

    /**
     * W3+ / S6+: adapter + device + queue + host-fixed create-buffer + write-buffer.
     * Guest passes WIT borrow buffer + list data (JNI still host-fixed 4 bytes).
     * `[method]gpu-queue.write-buffer-with-copy`.
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
     * W3+ / S6+: adapter + device + queue + host-fixed 1×1 COPY_DST texture write.
     * `[method]gpu-queue.write-texture-with-copy` (JNI still host-fixed).
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
    /** 256-byte row alignment for described texel copies (1×1 RGBA8). */
    private const val STUB_TEXEL_COPY_BYTES = 256L
    private val STUB_BUFFER_BYTES = byteArrayOf(1, 2, 3, 4)
    private val STUB_TEXTURE_BYTES = byteArrayOf(1, 2, 3, 4)
    private const val STUB_WGSL = "@compute @workgroup_size(1) fn main() {}"
}
