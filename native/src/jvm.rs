//! Process-wide `JavaVM` for host callbacks into Kotlin.

use jni::objects::{GlobalRef, JObject, JValue};
use jni::sys::JavaVM as SysJavaVM;
use jni::{JNIEnv, JavaVM};
use std::sync::OnceLock;

static JVM: OnceLock<JavaVM> = OnceLock::new();

pub fn set_vm(vm: *mut SysJavaVM) {
    let vm = unsafe { JavaVM::from_raw(vm) }.expect("JavaVM::from_raw");
    let _ = JVM.set(vm);
}

pub fn call_u32_u32_to_u32(cb: &GlobalRef, a: u32, b: u32) -> Result<u32, String> {
    let jvm = JVM.get().ok_or_else(|| "JavaVM not initialized".to_string())?;
    let mut env = jvm
        .attach_current_thread()
        .map_err(|e| format!("attach_current_thread: {e}"))?;
    let result = env
        .call_method(
            cb.as_obj(),
            "invoke",
            "(II)I",
            &[JValue::Int(a as i32), JValue::Int(b as i32)],
        )
        .map_err(|e| format!("host invoke: {e}"))?;
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_describe();
        let _ = env.exception_clear();
        return Err("host callback threw".into());
    }
    result
        .i()
        .map(|v| v as u32)
        .map_err(|e| format!("host invoke result: {e}"))
}

pub fn global_ref(env: &mut JNIEnv, obj: JObject) -> Result<GlobalRef, String> {
    env.new_global_ref(&obj)
        .map_err(|e| format!("new_global_ref: {e}"))
}
