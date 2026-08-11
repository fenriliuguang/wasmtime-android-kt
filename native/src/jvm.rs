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

fn with_env<T>(f: impl FnOnce(&mut JNIEnv) -> Result<T, String>) -> Result<T, String> {
    let jvm = JVM.get().ok_or_else(|| "JavaVM not initialized".to_string())?;
    let mut env = jvm
        .attach_current_thread()
        .map_err(|e| format!("attach_current_thread: {e}"))?;
    f(&mut env)
}

fn check_exception(env: &mut JNIEnv) -> Result<(), String> {
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_describe();
        let _ = env.exception_clear();
        return Err("host callback threw".into());
    }
    Ok(())
}

pub fn call_u32_u32_to_u32(cb: &GlobalRef, a: u32, b: u32) -> Result<u32, String> {
    with_env(|env| {
        let result = env
            .call_method(
                cb.as_obj(),
                "invoke",
                "(II)I",
                &[JValue::Int(a as i32), JValue::Int(b as i32)],
            )
            .map_err(|e| format!("host invoke: {e}"))?;
        check_exception(env)?;
        result
            .i()
            .map(|v| v as u32)
            .map_err(|e| format!("host invoke result: {e}"))
    })
}

fn call_i(cb: &GlobalRef, name: &str, sig: &str, args: &[JValue]) -> Result<u32, String> {
    with_env(|env| {
        let result = env
            .call_method(cb.as_obj(), name, sig, args)
            .map_err(|e| format!("host {name}: {e}"))?;
        check_exception(env)?;
        result
            .i()
            .map(|v| v as u32)
            .map_err(|e| format!("host {name} result: {e}"))
    })
}

fn call_void(cb: &GlobalRef, name: &str, sig: &str, args: &[JValue]) -> Result<(), String> {
    with_env(|env| {
        env.call_method(cb.as_obj(), name, sig, args)
            .map_err(|e| format!("host {name}: {e}"))?;
        check_exception(env)
    })
}

pub fn exp_request_adapter(cb: &GlobalRef) -> Result<u32, String> {
    call_i(cb, "requestAdapter", "()I", &[])
}

pub fn exp_adapter_request_device(cb: &GlobalRef, adapter: u32) -> Result<u32, String> {
    call_i(
        cb,
        "adapterRequestDevice",
        "(I)I",
        &[JValue::Int(adapter as i32)],
    )
}

pub fn exp_device_get_queue(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceGetQueue",
        "(I)I",
        &[JValue::Int(device as i32)],
    )
}

pub fn exp_create_surface(cb: &GlobalRef, window: u64) -> Result<u32, String> {
    call_i(
        cb,
        "createSurfaceFromNativeWindow",
        "(J)I",
        &[JValue::Long(window as i64)],
    )
}

pub fn exp_surface_configure(
    cb: &GlobalRef,
    surface: u32,
    device: u32,
    adapter: u32,
    width: u32,
    height: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "surfaceConfigure",
        "(IIIII)I",
        &[
            JValue::Int(surface as i32),
            JValue::Int(device as i32),
            JValue::Int(adapter as i32),
            JValue::Int(width as i32),
            JValue::Int(height as i32),
        ],
    )
}

pub fn exp_surface_get_view(cb: &GlobalRef, surface: u32) -> Result<u32, String> {
    call_i(
        cb,
        "surfaceGetCurrentTextureView",
        "(I)I",
        &[JValue::Int(surface as i32)],
    )
}

pub fn exp_create_command_encoder(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateCommandEncoder",
        "(I)I",
        &[JValue::Int(device as i32)],
    )
}

pub fn exp_begin_render_pass_clear(
    cb: &GlobalRef,
    encoder: u32,
    view: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "beginRenderPassClear",
        "(II)I",
        &[JValue::Int(encoder as i32), JValue::Int(view as i32)],
    )
}

pub fn exp_render_pass_end(cb: &GlobalRef, pass: u32) -> Result<(), String> {
    call_void(cb, "renderPassEnd", "(I)V", &[JValue::Int(pass as i32)])
}

pub fn exp_command_encoder_finish(cb: &GlobalRef, encoder: u32) -> Result<u32, String> {
    call_i(
        cb,
        "commandEncoderFinish",
        "(I)I",
        &[JValue::Int(encoder as i32)],
    )
}

pub fn exp_queue_submit1(cb: &GlobalRef, queue: u32, commands: u32) -> Result<(), String> {
    call_void(
        cb,
        "queueSubmit1",
        "(II)V",
        &[JValue::Int(queue as i32), JValue::Int(commands as i32)],
    )
}

pub fn exp_surface_present(cb: &GlobalRef, surface: u32) -> Result<(), String> {
    call_void(
        cb,
        "surfacePresent",
        "(I)V",
        &[JValue::Int(surface as i32)],
    )
}

pub fn exp_surface_unconfigure(cb: &GlobalRef, surface: u32) -> Result<(), String> {
    call_void(
        cb,
        "surfaceUnconfigure",
        "(I)V",
        &[JValue::Int(surface as i32)],
    )
}

pub fn global_ref(env: &mut JNIEnv, obj: JObject) -> Result<GlobalRef, String> {
    env.new_global_ref(&obj)
        .map_err(|e| format!("new_global_ref: {e}"))
}
