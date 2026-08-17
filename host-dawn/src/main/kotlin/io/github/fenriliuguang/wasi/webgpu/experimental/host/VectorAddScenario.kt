package io.github.fenriliuguang.wasi.webgpu.experimental.host

import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * Shared compute acceptance path used by Kotlin callers, CPU host tests, and abi-mvp guests.
 */
object VectorAddScenario {

    /** Exact WGSL bytes are also embedded in `guest/vector-add/vector_add.wat` data section. */
    val SHADER: String =
        "@group(0) @binding(0) var<storage, read> inputA : array<f32>;\n" +
            "@group(0) @binding(1) var<storage, read> inputB : array<f32>;\n" +
            "@group(0) @binding(2) var<storage, read_write> output : array<f32>;\n" +
            "\n" +
            "@compute @workgroup_size(64)\n" +
            "fn main(@builtin(global_invocation_id) gid : vec3<u32>) {\n" +
            "  let i = gid.x;\n" +
            "  if (i >= arrayLength(&output)) {\n" +
            "    return;\n" +
            "  }\n" +
            "  output[i] = inputA[i] + inputB[i];\n" +
            "}"

    fun runOn(host: WasiWebGpuHost, a: FloatArray, b: FloatArray): FloatArray {
        require(a.size == b.size && a.isNotEmpty())
        val n = a.size
        val bytes = n * 4L

        val adapter = host.requestAdapter()
        val device = host.adapterRequestDevice(adapter)
        val queue = host.deviceGetQueue(device)

        val storageUsage = GpuBufferUsage.STORAGE or GpuBufferUsage.COPY_DST or GpuBufferUsage.COPY_SRC
        val bufA = host.deviceCreateBuffer(device, BufferDescriptor(size = bytes, usage = storageUsage))
        val bufB = host.deviceCreateBuffer(device, BufferDescriptor(size = bytes, usage = storageUsage))
        val bufOut = host.deviceCreateBuffer(device, BufferDescriptor(size = bytes, usage = storageUsage))
        val bufRead = host.deviceCreateBuffer(
            device,
            BufferDescriptor(
                size = bytes,
                usage = GpuBufferUsage.MAP_READ or GpuBufferUsage.COPY_DST,
            ),
        )

        host.queueWriteBuffer(queue, bufA, 0, floatsToBytes(a))
        host.queueWriteBuffer(queue, bufB, 0, floatsToBytes(b))

        val shaderModule = host.deviceCreateShaderModule(device, ShaderModuleDescriptor(code = SHADER))
        val layout = host.deviceCreateBindGroupLayout(
            device,
            BindGroupLayoutDescriptor(
                entries = listOf(
                    storageEntry(0, BufferBindingType.ReadOnlyStorage),
                    storageEntry(1, BufferBindingType.ReadOnlyStorage),
                    storageEntry(2, BufferBindingType.Storage),
                ),
            ),
        )
        val bindGroup = host.deviceCreateBindGroup(
            device,
            BindGroupDescriptor(
                layout = layout,
                entries = listOf(
                    BindGroupEntry(0, BufferBinding(bufA)),
                    BindGroupEntry(1, BufferBinding(bufB)),
                    BindGroupEntry(2, BufferBinding(bufOut)),
                ),
            ),
        )
        val pipelineLayout = host.deviceCreatePipelineLayout(
            device,
            PipelineLayoutDescriptor(bindGroupLayouts = listOf(layout)),
        )
        val pipeline = host.deviceCreateComputePipeline(
            device,
            ComputePipelineDescriptor(
                layout = pipelineLayout,
                compute = ProgrammableStage(module = shaderModule, entryPoint = "main"),
            ),
        )

        val encoder = host.deviceCreateCommandEncoder(device)
        val pass = host.commandEncoderBeginComputePass(encoder)
        host.computePassSetPipeline(pass, pipeline)
        host.computePassSetBindGroup(pass, 0, bindGroup)
        host.computePassDispatchWorkgroups(pass, (n + 63) / 64)
        host.computePassEnd(pass)
        host.commandEncoderCopyBufferToBuffer(encoder, bufOut, 0, bufRead, 0, bytes)
        val commands = host.commandEncoderFinish(encoder)
        host.queueSubmit(queue, listOf(commands))

        host.bufferMapAsync(bufRead, GpuMapMode.READ, 0, bytes)
        val raw = host.bufferGetMappedRange(bufRead, 0, bytes)
        host.bufferUnmap(bufRead)
        return bytesToFloats(raw)
    }

    fun floatsToBytes(values: FloatArray): ByteArray {
        val buffer = ByteBuffer.allocate(values.size * 4).order(ByteOrder.LITTLE_ENDIAN)
        values.forEach(buffer::putFloat)
        return buffer.array()
    }

    fun bytesToFloats(bytes: ByteArray): FloatArray {
        val buffer = ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)
        return FloatArray(bytes.size / 4) { buffer.float }
    }

    private fun storageEntry(binding: Int, type: BufferBindingType) = BindGroupLayoutEntry(
        binding = binding,
        visibility = GpuShaderStage.COMPUTE,
        buffer = BufferBindingLayout(type = type, minBindingSize = 4),
    )
}
