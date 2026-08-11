package io.github.fenriliuguang.wasmtime.android.api

/**
 * Base failure from the Track B Wasmtime JNI layer.
 *
 * Prefer catching a specific subtype ([WasmtimeApiException], [WasmtimeCompileException],
 * [WasmtimeLinkException], [WasmtimeTrapException]) when branching; `catch (WasmtimeException)`
 * remains valid for all L1 failures.
 *
 * Mapping policy: `docs/mapping/errors.md`.
 */
open class WasmtimeException : RuntimeException {
    val kind: Kind

    constructor(kind: Kind, message: String?) : super(message) {
        this.kind = kind
    }

    constructor(kind: Kind, message: String?, cause: Throwable?) : super(message, cause) {
        this.kind = kind
    }

    /** Kept for older call sites; defaults to [Kind.TRAP]. */
    constructor(message: String?) : this(Kind.TRAP, message)

    constructor(message: String?, cause: Throwable?) : this(Kind.TRAP, message, cause)

    enum class Kind {
        /** Closed handle, null JNI args, API misuse. */
        API,

        /** Component bytes failed to compile. */
        COMPILE,

        /** Linker define / instantiate failed. */
        LINK,

        /**
         * Guest trap or export call failure.
         * Includes Track A L2 `HostException` raised inside experimental host callbacks
         * (sync-compat → trap).
         */
        TRAP,
    }
}

/** API misuse (null/closed handles, missing callbacks at registration). */
class WasmtimeApiException : WasmtimeException {
    constructor(message: String?) : super(Kind.API, message)
    constructor(message: String?, cause: Throwable?) : super(Kind.API, message, cause)
}

/** Component compile failure. */
class WasmtimeCompileException : WasmtimeException {
    constructor(message: String?) : super(Kind.COMPILE, message)
    constructor(message: String?, cause: Throwable?) : super(Kind.COMPILE, message, cause)
}

/** Linker / instantiate failure. */
class WasmtimeLinkException : WasmtimeException {
    constructor(message: String?) : super(Kind.LINK, message)
    constructor(message: String?, cause: Throwable?) : super(Kind.LINK, message, cause)
}

/** Guest trap or export call failure (includes host-callback → trap). */
class WasmtimeTrapException : WasmtimeException {
    constructor(message: String?) : super(Kind.TRAP, message)
    constructor(message: String?, cause: Throwable?) : super(Kind.TRAP, message, cause)
}
