//! Map Rust errors to typed `WasmtimeException` subclasses.

use jni::JNIEnv;

#[derive(Clone, Copy)]
pub enum Kind {
    Api,
    Compile,
    Link,
    Trap,
}

impl Kind {
    fn class(self) -> &'static str {
        match self {
            Kind::Api => "io/github/fenriliuguang/wasmtime/android/api/WasmtimeApiException",
            Kind::Compile => {
                "io/github/fenriliuguang/wasmtime/android/api/WasmtimeCompileException"
            }
            Kind::Link => "io/github/fenriliuguang/wasmtime/android/api/WasmtimeLinkException",
            Kind::Trap => "io/github/fenriliuguang/wasmtime/android/api/WasmtimeTrapException",
        }
    }
}

pub fn throw_kind(env: &mut JNIEnv, kind: Kind, msg: impl AsRef<str>) {
    let msg = msg.as_ref();
    // ART aborts if FindClass runs with a pending Java exception (host callback
    // threw, jni-rs left it pending). Clear first, then throw our typed error.
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_describe();
        let _ = env.exception_clear();
    }
    if let Err(e) = env.throw_new(kind.class(), msg) {
        eprintln!("failed to throw {:?}: {msg}: {e}", kind.class());
    }
}

pub fn throw_api(env: &mut JNIEnv, msg: impl AsRef<str>) {
    throw_kind(env, Kind::Api, msg)
}

pub fn throw_compile(env: &mut JNIEnv, err: impl std::fmt::Display) {
    throw_kind(env, Kind::Compile, format!("{err}"))
}

pub fn throw_link(env: &mut JNIEnv, err: impl std::fmt::Display) {
    throw_kind(env, Kind::Link, format!("{err}"))
}

pub fn throw_trap(env: &mut JNIEnv, err: impl std::fmt::Display) {
    throw_kind(env, Kind::Trap, format!("{err}"))
}

/// Default / legacy helper → API (null handles, registration misuse).
pub fn throw(env: &mut JNIEnv, msg: impl AsRef<str>) {
    throw_api(env, msg)
}

/// Default display helper → trap (export/call failures).
pub fn throw_err(env: &mut JNIEnv, err: impl std::fmt::Display) {
    throw_trap(env, err)
}
