package io.github.fenriliuguang.wasi.webgpu.experimental.host

/**
 * Kotlin mirror of wasi:webgpu@0.3.0-rc.2 error records / kinds (compliant-world slice F).
 *
 * Host failures still surface as [HostException] on the experimental track (trap in CM
 * callbacks). The wasi:webgpu track lifts selected methods to WIT `result` using these shapes.
 *
 * @see docs/mapping/errors-async.md
 */
enum class GpuErrorKind {
    ValidationError,
    OutOfMemoryError,
    InternalError,
}

enum class RequestDeviceErrorKind {
    TypeError,
    OperationError,
}

enum class MapAsyncErrorKind {
    OperationError,
    RangeError,
    AbortError,
}

enum class GetMappedRangeErrorKind {
    OperationError,
    RangeError,
    TypeError,
}

enum class UnmapErrorKind {
    AbortError,
}

enum class SetBindGroupErrorKind {
    RangeError,
}

enum class WriteBufferErrorKind {
    OperationError,
}

enum class CreateQuerySetErrorKind {
    TypeError,
}

enum class PopErrorScopeErrorKind {
    OperationError,
}

enum class GpuPipelineErrorReason {
    Validation,
    Internal,
}

/**
 * Maps [HostException] onto standard-package error-kind discriminants.
 *
 * Heuristic (Dawn/spec may disagree): prefer validation for [HostException.Validation] /
 * [HostException.InvalidHandle]; treat [HostException.Unsupported] and [HostException.Backend]
 * as operation/internal unless a method has only a narrower kind set.
 */
object HostErrorMapping {

    fun gpuErrorKind(ex: HostException): GpuErrorKind = when (ex) {
        is HostException.Validation,
        is HostException.InvalidHandle,
        -> GpuErrorKind.ValidationError
        is HostException.Unsupported,
        is HostException.Backend,
        -> GpuErrorKind.InternalError
    }

    fun requestDevice(ex: HostException): RequestDeviceErrorKind = when (ex) {
        is HostException.Validation -> RequestDeviceErrorKind.TypeError
        else -> RequestDeviceErrorKind.OperationError
    }

    fun mapAsync(ex: HostException): MapAsyncErrorKind = when (ex) {
        is HostException.Validation -> MapAsyncErrorKind.RangeError
        is HostException.Backend -> MapAsyncErrorKind.AbortError
        else -> MapAsyncErrorKind.OperationError
    }

    fun getMappedRange(ex: HostException): GetMappedRangeErrorKind = when (ex) {
        is HostException.Validation -> GetMappedRangeErrorKind.TypeError
        else -> GetMappedRangeErrorKind.OperationError
    }

    @Suppress("UNUSED_PARAMETER")
    fun unmap(ex: HostException): UnmapErrorKind = UnmapErrorKind.AbortError

    @Suppress("UNUSED_PARAMETER")
    fun setBindGroup(ex: HostException): SetBindGroupErrorKind = SetBindGroupErrorKind.RangeError

    @Suppress("UNUSED_PARAMETER")
    fun writeBuffer(ex: HostException): WriteBufferErrorKind = WriteBufferErrorKind.OperationError

    @Suppress("UNUSED_PARAMETER")
    fun createQuerySet(ex: HostException): CreateQuerySetErrorKind = CreateQuerySetErrorKind.TypeError

    @Suppress("UNUSED_PARAMETER")
    fun popErrorScope(ex: HostException): PopErrorScopeErrorKind = PopErrorScopeErrorKind.OperationError

    fun pipelineReason(ex: HostException): GpuPipelineErrorReason = when (ex) {
        is HostException.Validation,
        is HostException.InvalidHandle,
        -> GpuPipelineErrorReason.Validation
        else -> GpuPipelineErrorReason.Internal
    }

    /** Discriminant kebab-case as in WIT (for ComponentVal.variant). */
    fun witCase(kind: Enum<*>): String = when (kind) {
        is GpuErrorKind -> when (kind) {
            GpuErrorKind.ValidationError -> "validation-error"
            GpuErrorKind.OutOfMemoryError -> "out-of-memory-error"
            GpuErrorKind.InternalError -> "internal-error"
        }
        is RequestDeviceErrorKind -> when (kind) {
            RequestDeviceErrorKind.TypeError -> "type-error"
            RequestDeviceErrorKind.OperationError -> "operation-error"
        }
        is MapAsyncErrorKind -> when (kind) {
            MapAsyncErrorKind.OperationError -> "operation-error"
            MapAsyncErrorKind.RangeError -> "range-error"
            MapAsyncErrorKind.AbortError -> "abort-error"
        }
        is GetMappedRangeErrorKind -> when (kind) {
            GetMappedRangeErrorKind.OperationError -> "operation-error"
            GetMappedRangeErrorKind.RangeError -> "range-error"
            GetMappedRangeErrorKind.TypeError -> "type-error"
        }
        is UnmapErrorKind -> "abort-error"
        is SetBindGroupErrorKind -> "range-error"
        is WriteBufferErrorKind -> "operation-error"
        is CreateQuerySetErrorKind -> "type-error"
        is PopErrorScopeErrorKind -> "operation-error"
        is GpuPipelineErrorReason -> when (kind) {
            GpuPipelineErrorReason.Validation -> "validation"
            GpuPipelineErrorReason.Internal -> "internal"
        }
        else -> kind.name
            .replace(Regex("([a-z])([A-Z])"), "$1-$2")
            .lowercase()
    }

    fun messageOf(ex: HostException): String = ex.message ?: ex::class.simpleName ?: "error"
}
