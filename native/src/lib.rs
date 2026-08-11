//! Track B thin L1: upstream Wasmtime + JNI.
//! M0: `JNI_OnLoad` + identity / version probes.
//! M1: sync Component Model compile / instantiate / call export.

mod cm;
mod error;
mod handles;

use jni::objects::JClass;
use jni::sys::{jint, jstring, JNI_VERSION_1_6, JavaVM};
use jni::JNIEnv;
use std::os::raw::c_void;

/// ART accepts only JNI 1.2 / 1.4 / 1.6 — not 1.8 (65544). Track A lesson.
#[no_mangle]
pub extern "system" fn JNI_OnLoad(_vm: *mut JavaVM, _reserved: *mut c_void) -> jint {
    JNI_VERSION_1_6
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeRuntimeId(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    to_jstring(&mut env, "wasmtime-android-kt/0.1.0-experimental")
}

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeWasmtimeVersion(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    // Constructing Engine proves the cdylib links upstream Wasmtime (M0 DoD).
    // Wasmtime 47.x has no public VERSION constant; keep in sync with Cargo.lock.
    let _engine = wasmtime::Engine::default();
    to_jstring(&mut env, "47.0.3")
}

fn to_jstring(env: &mut JNIEnv, s: &str) -> jstring {
    match env.new_string(s) {
        Ok(js) => js.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}
