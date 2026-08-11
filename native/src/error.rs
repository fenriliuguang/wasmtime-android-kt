//! Map Rust errors to `WasmtimeException`.

use jni::JNIEnv;

const EXCEPTION: &str = "io/github/fenriliuguang/wasmtime/android/api/WasmtimeException";

pub fn throw(env: &mut JNIEnv, msg: impl AsRef<str>) {
    let msg = msg.as_ref();
    if let Err(e) = env.throw_new(EXCEPTION, msg) {
        eprintln!("failed to throw WasmtimeException ({msg}): {e}");
    }
}

pub fn throw_err(env: &mut JNIEnv, err: impl std::fmt::Display) {
    throw(env, format!("{err}"));
}
