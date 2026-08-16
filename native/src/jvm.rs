//! Process-wide `JavaVM` for host callbacks into Kotlin.

use jni::objects::{GlobalRef, JObject, JValue};
use jni::sys::JavaVM as SysJavaVM;
use jni::{JNIEnv, JavaVM};
use std::cell::RefCell;
use std::sync::mpsc::{self, Sender};
use std::sync::OnceLock;

static JVM: OnceLock<JavaVM> = OnceLock::new();

enum PumpJob {
    Jni(Box<dyn FnOnce(&mut JNIEnv) + Send>),
    Done,
}

thread_local! {
    static PUMP_JNI: RefCell<Option<Sender<PumpJob>>> = const { RefCell::new(None) };
}

pub fn set_vm(vm: *mut SysJavaVM) {
    let vm = unsafe { JavaVM::from_raw(vm) }.expect("JavaVM::from_raw");
    let _ = JVM.set(vm);
}

/// Drive `f` on an 8MiB pthread. Host JNI from that thread is bounced here:
/// ART `AttachCurrentThread` on a custom-stack pthread aborts
/// (`FindStackTop` vs `GetStackEnd`, Vivo / Android 16).
pub fn run_on_cm_pump<F, T>(env: &mut JNIEnv, stack_bytes: usize, f: F) -> Result<T, String>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let pump_tx = tx.clone();
    let join = std::thread::Builder::new()
        .name("wasmtime-cm-pump".into())
        .stack_size(stack_bytes)
        .spawn(move || {
            PUMP_JNI.with(|slot| *slot.borrow_mut() = Some(pump_tx.clone()));
            let result = f();
            PUMP_JNI.with(|slot| *slot.borrow_mut() = None);
            let _ = pump_tx.send(PumpJob::Done);
            result
        })
        .map_err(|e| format!("cm pump spawn: {e}"))?;
    drop(tx);
    while let Ok(job) = rx.recv() {
        match job {
            PumpJob::Jni(work) => work(env),
            PumpJob::Done => break,
        }
    }
    join.join()
        .map_err(|_| "cm pump thread panicked".to_string())
}

fn with_env<T: Send + 'static>(
    f: impl FnOnce(&mut JNIEnv) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    let bounced = PUMP_JNI.with(|slot| slot.borrow().clone());
    if let Some(tx) = bounced {
        let (rtx, rrx) = mpsc::channel();
        tx.send(PumpJob::Jni(Box::new(move |env| {
            let _ = rtx.send(f(env));
        })))
        .map_err(|_| "cm pump JNI bounce closed".to_string())?;
        return rrx
            .recv()
            .map_err(|_| "cm pump JNI bounce dropped".to_string())?;
    }
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

#[derive(Clone, Copy)]
enum HostArg {
    Int(i32),
    Long(i64),
}

impl HostArg {
    fn as_jvalue(self) -> JValue<'static, 'static> {
        match self {
            HostArg::Int(v) => JValue::Int(v),
            HostArg::Long(v) => JValue::Long(v),
        }
    }
}

pub fn call_u32_u32_to_u32(cb: &GlobalRef, a: u32, b: u32) -> Result<u32, String> {
    let cb = cb.clone();
    with_env(move |env| {
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

fn call_i(cb: &GlobalRef, name: &'static str, sig: &'static str, args: Vec<HostArg>) -> Result<u32, String> {
    let cb = cb.clone();
    with_env(move |env| {
        let jargs: Vec<JValue> = args.iter().copied().map(HostArg::as_jvalue).collect();
        let result = env
            .call_method(cb.as_obj(), name, sig, &jargs)
            .map_err(|e| format!("host {name}: {e}"))?;
        check_exception(env)?;
        result
            .i()
            .map(|v| v as u32)
            .map_err(|e| format!("host {name} result: {e}"))
    })
}

fn call_void(cb: &GlobalRef, name: &'static str, sig: &'static str, args: Vec<HostArg>) -> Result<(), String> {
    let cb = cb.clone();
    with_env(move |env| {
        let jargs: Vec<JValue> = args.iter().copied().map(HostArg::as_jvalue).collect();
        env.call_method(cb.as_obj(), name, sig, &jargs)
            .map_err(|e| format!("host {name}: {e}"))?;
        check_exception(env)
    })
}

pub fn exp_request_adapter(cb: &GlobalRef) -> Result<u32, String> {
    call_i(cb, "requestAdapter", "()I", vec![])
}

pub fn exp_adapter_request_device(cb: &GlobalRef, adapter: u32) -> Result<u32, String> {
    call_i(
        cb,
        "adapterRequestDevice",
        "(I)I",
        vec![HostArg::Int(adapter as i32)],
    )
}

pub fn exp_device_get_queue(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceGetQueue",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

pub fn exp_create_surface(cb: &GlobalRef, window: u64) -> Result<u32, String> {
    call_i(
        cb,
        "createSurfaceFromNativeWindow",
        "(J)I",
        vec![HostArg::Long(window as i64)],
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
        vec![
            HostArg::Int(surface as i32),
            HostArg::Int(device as i32),
            HostArg::Int(adapter as i32),
            HostArg::Int(width as i32),
            HostArg::Int(height as i32),
        ],
    )
}

pub fn exp_surface_get_view(cb: &GlobalRef, surface: u32) -> Result<u32, String> {
    call_i(
        cb,
        "surfaceGetCurrentTextureView",
        "(I)I",
        vec![HostArg::Int(surface as i32)],
    )
}

pub fn exp_create_command_encoder(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateCommandEncoder",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

pub fn exp_create_buffer(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateBuffer",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

pub fn exp_create_texture(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateTexture",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

pub fn exp_create_sampler(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateSampler",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

pub fn exp_create_shader_module(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateShaderModule",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

pub fn exp_create_bind_group_layout(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateBindGroupLayout",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

pub fn exp_create_pipeline_layout(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreatePipelineLayout",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

pub fn exp_create_bind_group(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateBindGroup",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

pub fn exp_create_render_pipeline(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateRenderPipeline",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

pub fn exp_create_compute_pipeline(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateComputePipeline",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

pub fn exp_begin_compute_pass(cb: &GlobalRef, encoder: u32) -> Result<u32, String> {
    call_i(
        cb,
        "beginComputePass",
        "(I)I",
        vec![HostArg::Int(encoder as i32)],
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
        vec![HostArg::Int(encoder as i32), HostArg::Int(view as i32)],
    )
}

pub fn exp_render_pass_end(cb: &GlobalRef, pass: u32) -> Result<(), String> {
    call_void(cb, "renderPassEnd", "(I)V", vec![HostArg::Int(pass as i32)])
}

pub fn exp_command_encoder_finish(cb: &GlobalRef, encoder: u32) -> Result<u32, String> {
    call_i(
        cb,
        "commandEncoderFinish",
        "(I)I",
        vec![HostArg::Int(encoder as i32)],
    )
}

pub fn exp_queue_submit1(cb: &GlobalRef, queue: u32, commands: u32) -> Result<(), String> {
    call_void(
        cb,
        "queueSubmit1",
        "(II)V",
        vec![HostArg::Int(queue as i32), HostArg::Int(commands as i32)],
    )
}

pub fn exp_queue_write_buffer(cb: &GlobalRef, queue: u32, buffer: u32) -> Result<(), String> {
    call_void(
        cb,
        "queueWriteBuffer",
        "(II)V",
        vec![HostArg::Int(queue as i32), HostArg::Int(buffer as i32)],
    )
}

pub fn exp_queue_write_texture(cb: &GlobalRef, queue: u32, texture: u32) -> Result<(), String> {
    call_void(
        cb,
        "queueWriteTexture",
        "(II)V",
        vec![HostArg::Int(queue as i32), HostArg::Int(texture as i32)],
    )
}

pub fn exp_texture_create_view(cb: &GlobalRef, texture: u32) -> Result<u32, String> {
    call_i(
        cb,
        "textureCreateView",
        "(I)I",
        vec![HostArg::Int(texture as i32)],
    )
}

pub fn exp_surface_present(cb: &GlobalRef, surface: u32) -> Result<(), String> {
    call_void(
        cb,
        "surfacePresent",
        "(I)V",
        vec![HostArg::Int(surface as i32)],
    )
}

pub fn exp_surface_unconfigure(cb: &GlobalRef, surface: u32) -> Result<(), String> {
    call_void(
        cb,
        "surfaceUnconfigure",
        "(I)V",
        vec![HostArg::Int(surface as i32)],
    )
}

pub fn global_ref(env: &mut JNIEnv, obj: JObject) -> Result<GlobalRef, String> {
    env.new_global_ref(&obj)
        .map_err(|e| format!("new_global_ref: {e}"))
}
