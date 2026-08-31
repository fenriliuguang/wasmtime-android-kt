//! Track B thin L1: upstream Wasmtime + JNI.
//! M0: `JNI_OnLoad` + identity / version probes.
//! M1: sync Component Model (host import + u32 resource).
//! M2: true CM async (`func_wrap_concurrent` + `FutureReader` + `run_concurrent`).
//! P3: stream.read (host→guest) + stream.write/take (guest→host).

mod cm;
mod engine;
mod error;
mod gpu_dispatch;
mod handles;
mod host;
mod jvm;
mod native_gpu;
#[cfg(test)]
mod native_gpu_consume_tests;
mod webgpu_abi;

use jni::objects::JClass;
use jni::sys::{jint, jlong, jstring, JavaVM, JNI_VERSION_1_6};
use jni::JNIEnv;
use std::os::raw::c_void;

/// ART accepts only JNI 1.2 / 1.4 / 1.6 — not 1.8 (65544). Track A lesson.
#[no_mangle]
pub extern "system" fn JNI_OnLoad(vm: *mut JavaVM, _reserved: *mut c_void) -> jint {
    jvm::set_vm(vm);
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

// H9: cap BLAST images. `ANativeWindow_setBufferCount` lives in
// libnativewindow.so (not libandroid); public NDK headers omit it.
#[cfg(target_os = "android")]
extern "C" {
    fn dlopen(filename: *const i8, flags: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const i8) -> *mut c_void;
}

#[cfg(target_os = "android")]
const RTLD_NOW: i32 = 2;

#[no_mangle]
pub extern "system" fn Java_io_github_fenriliuguang_wasmtime_android_jni_NativeBridge_nativeSetANativeWindowBufferCount(
    _env: JNIEnv,
    _class: JClass,
    window: jlong,
    count: jint,
) -> jint {
    #[cfg(target_os = "android")]
    {
        if window == 0 || count < 2 {
            return -1;
        }
        unsafe {
            let lib = dlopen(c"libnativewindow.so".as_ptr() as *const i8, RTLD_NOW);
            if lib.is_null() {
                return -2;
            }
            let sym = dlsym(lib, c"ANativeWindow_setBufferCount".as_ptr() as *const i8);
            if sym.is_null() {
                return -3;
            }
            let f: extern "C" fn(*mut c_void, usize) -> i32 = std::mem::transmute(sym);
            f(window as *mut c_void, count as usize)
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (window, count);
        -1
    }
}
