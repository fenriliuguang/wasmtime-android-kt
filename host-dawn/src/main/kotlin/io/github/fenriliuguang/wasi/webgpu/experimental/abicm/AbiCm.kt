package io.github.fenriliuguang.wasi.webgpu.experimental.abicm

/**
 * Experimental Component Model ABI constants for compute vector-add +
 * minimal Android surface/render.
 *
 * NOT compliant wasi:webgpu — package is experimental; handles are WIT resources
 * (internally mapped to L2 [io.github.fenriliuguang.wasi.webgpu.experimental.host.GpuHandle]).
 */
object AbiCm {
    const val PACKAGE: String = "experimental:webgpu-cm"
    const val INTERFACE: String = "host"
    const val VERSION: String = "0.8.0"

    /** Full import interface id as emitted by wit-bindgen / wasm-tools. */
    const val IMPORT_INTERFACE: String = "$PACKAGE/$INTERFACE@$VERSION"

    const val EXPORT_RUN_VECTOR_ADD: String = "run-vector-add"
    const val EXPORT_RUN_TRIANGLE: String = "run-triangle"
    const val EXPORT_INIT_TRIANGLE: String = "init-triangle"
    /** Shared export name on triangle / cube worlds (separate wasm components). */
    const val EXPORT_DRAW_FRAME: String = "draw-frame"
    const val EXPORT_DROP_TRIANGLE: String = "drop-triangle"
    const val EXPORT_RUN_CUBE: String = "run-cube"
    const val EXPORT_INIT_CUBE: String = "init-cube"
    const val EXPORT_DROP_CUBE: String = "drop-cube"

    object Resource {
        const val ADAPTER = "adapter"
        const val DEVICE = "device"
        const val QUEUE = "queue"
        const val BUFFER = "buffer"
        const val SHADER_MODULE = "shader-module"
        const val BIND_GROUP_LAYOUT = "bind-group-layout"
        const val BIND_GROUP = "bind-group"
        const val PIPELINE_LAYOUT = "pipeline-layout"
        const val COMPUTE_PIPELINE = "compute-pipeline"
        const val RENDER_PIPELINE = "render-pipeline"
        const val SAMPLER = "sampler"
        const val TEXTURE = "texture"
        const val TEXTURE_VIEW = "texture-view"
        const val SURFACE = "surface"
        const val COMMAND_ENCODER = "command-encoder"
        const val COMPUTE_PASS_ENCODER = "compute-pass-encoder"
        const val RENDER_PASS_ENCODER = "render-pass-encoder"
        const val COMMAND_BUFFER = "command-buffer"

        val ALL: List<String> = listOf(
            ADAPTER,
            DEVICE,
            QUEUE,
            BUFFER,
            SHADER_MODULE,
            BIND_GROUP_LAYOUT,
            BIND_GROUP,
            PIPELINE_LAYOUT,
            COMPUTE_PIPELINE,
            RENDER_PIPELINE,
            SAMPLER,
            TEXTURE,
            TEXTURE_VIEW,
            SURFACE,
            COMMAND_ENCODER,
            COMPUTE_PASS_ENCODER,
            RENDER_PASS_ENCODER,
            COMMAND_BUFFER,
        )
    }

    object Func {
        const val REQUEST_ADAPTER = "request-adapter"
        const val CREATE_SURFACE_FROM_NATIVE_WINDOW = "create-surface-from-native-window"
        const val ADAPTER_REQUEST_DEVICE = "[method]adapter.request-device"
        const val DEVICE_GET_QUEUE = "[method]device.get-queue"
        const val DEVICE_CREATE_BUFFER = "[method]device.create-buffer"
        const val QUEUE_WRITE_BUFFER = "[method]queue.write-buffer"
        const val QUEUE_WRITE_TEXTURE = "[method]queue.write-texture"
        const val DEVICE_CREATE_SHADER_MODULE = "[method]device.create-shader-module"
        const val DEVICE_CREATE_BIND_GROUP_LAYOUT = "[method]device.create-bind-group-layout"
        const val DEVICE_CREATE_BIND_GROUP = "[method]device.create-bind-group"
        const val DEVICE_CREATE_TEXTURE = "[method]device.create-texture"
        const val DEVICE_CREATE_SAMPLER = "[method]device.create-sampler"
        const val DEVICE_CREATE_PIPELINE_LAYOUT = "[method]device.create-pipeline-layout"
        const val DEVICE_CREATE_COMPUTE_PIPELINE = "[method]device.create-compute-pipeline"
        const val DEVICE_CREATE_RENDER_PIPELINE = "[method]device.create-render-pipeline"
        const val DEVICE_CREATE_BIND_GROUP_LAYOUT_STORAGE3 =
            "[method]device.create-bind-group-layout-storage3"
        const val DEVICE_CREATE_BIND_GROUP3 = "[method]device.create-bind-group3"
        const val DEVICE_CREATE_COMPUTE_PIPELINE_BGL =
            "[method]device.create-compute-pipeline-bgl"
        const val DEVICE_CREATE_RENDER_PIPELINE_TRIANGLE =
            "[method]device.create-render-pipeline-triangle"
        const val DEVICE_CREATE_RENDER_PIPELINE_TRIANGLE_BUFFERS =
            "[method]device.create-render-pipeline-triangle-buffers"
        const val DEVICE_CREATE_COMMAND_ENCODER = "[method]device.create-command-encoder"
        const val TEXTURE_CREATE_VIEW = "[method]texture.create-view"
        const val SURFACE_CONFIGURE = "[method]surface.configure"
        const val SURFACE_GET_CURRENT_TEXTURE_VIEW = "[method]surface.get-current-texture-view"
        const val SURFACE_PRESENT = "[method]surface.present"
        const val SURFACE_UNCONFIGURE = "[method]surface.unconfigure"
        const val COMMAND_ENCODER_BEGIN_COMPUTE_PASS =
            "[method]command-encoder.begin-compute-pass"
        const val COMMAND_ENCODER_BEGIN_RENDER_PASS =
            "[method]command-encoder.begin-render-pass"
        const val COMMAND_ENCODER_BEGIN_RENDER_PASS_CLEAR =
            "[method]command-encoder.begin-render-pass-clear"
        const val COMPUTE_PASS_SET_PIPELINE = "[method]compute-pass-encoder.set-pipeline"
        const val COMPUTE_PASS_SET_BIND_GROUP = "[method]compute-pass-encoder.set-bind-group"
        const val COMPUTE_PASS_DISPATCH_WORKGROUPS =
            "[method]compute-pass-encoder.dispatch-workgroups"
        const val COMPUTE_PASS_END = "[method]compute-pass-encoder.end"
        const val RENDER_PASS_SET_PIPELINE = "[method]render-pass-encoder.set-pipeline"
        const val RENDER_PASS_SET_BIND_GROUP = "[method]render-pass-encoder.set-bind-group"
        const val RENDER_PASS_SET_VERTEX_BUFFER = "[method]render-pass-encoder.set-vertex-buffer"
        const val RENDER_PASS_DRAW = "[method]render-pass-encoder.draw"
        const val RENDER_PASS_END = "[method]render-pass-encoder.end"
        const val COMMAND_ENCODER_COPY_BUFFER_TO_BUFFER =
            "[method]command-encoder.copy-buffer-to-buffer"
        const val COMMAND_ENCODER_FINISH = "[method]command-encoder.finish"
        const val QUEUE_SUBMIT = "[method]queue.submit"
        const val QUEUE_SUBMIT1 = "[method]queue.submit1"
        const val BUFFER_MAP_ASYNC = "[method]buffer.map-async"
        const val BUFFER_GET_MAPPED_RANGE = "[method]buffer.get-mapped-range"
        const val BUFFER_UNMAP = "[method]buffer.unmap"
    }
}
