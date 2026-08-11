package io.github.fenriliuguang.wasmtime.android.api

/** Runtime failure from the Track B Wasmtime JNI layer. */
class WasmtimeException : RuntimeException {
    constructor(message: String) : super(message)
    constructor(message: String, cause: Throwable) : super(message, cause)
}
