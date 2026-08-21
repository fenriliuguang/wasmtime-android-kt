//! Process-wide `JavaVM` for host callbacks into Kotlin.

use jni::objects::{GlobalRef, JByteArray, JIntArray, JObject, JString, JValue};
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
    let jvm = JVM
        .get()
        .ok_or_else(|| "JavaVM not initialized".to_string())?;
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

enum HostArg {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Str(String),
    Ints(Vec<i32>),
    Bytes(Vec<u8>),
}

fn call_with_host_args<'a>(
    env: &mut JNIEnv<'a>,
    cb: &JObject<'a>,
    name: &'static str,
    sig: &'static str,
    args: &[HostArg],
) -> Result<jni::objects::JValueOwned<'a>, String> {
    let mut java_strings: Vec<JString<'a>> = Vec::new();
    let mut java_int_arrays: Vec<JIntArray<'a>> = Vec::new();
    let mut java_byte_arrays: Vec<JByteArray<'a>> = Vec::new();
    for arg in args {
        match arg {
            HostArg::Str(s) => {
                java_strings.push(
                    env.new_string(s)
                        .map_err(|e| format!("host {name} new_string: {e}"))?,
                );
            }
            HostArg::Ints(v) => {
                let arr = env
                    .new_int_array(v.len() as i32)
                    .map_err(|e| format!("host {name} new_int_array: {e}"))?;
                env.set_int_array_region(&arr, 0, v)
                    .map_err(|e| format!("host {name} set_int_array_region: {e}"))?;
                java_int_arrays.push(arr);
            }
            HostArg::Bytes(v) => {
                let arr = env
                    .new_byte_array(v.len() as i32)
                    .map_err(|e| format!("host {name} new_byte_array: {e}"))?;
                let i8s: Vec<i8> = v.iter().map(|b| *b as i8).collect();
                env.set_byte_array_region(&arr, 0, &i8s)
                    .map_err(|e| format!("host {name} set_byte_array_region: {e}"))?;
                java_byte_arrays.push(arr);
            }
            HostArg::Int(_) | HostArg::Long(_) | HostArg::Float(_) | HostArg::Double(_) => {}
        }
    }
    let mut str_i = 0usize;
    let mut ints_i = 0usize;
    let mut bytes_i = 0usize;
    let jargs: Vec<JValue> = args
        .iter()
        .map(|arg| match arg {
            HostArg::Int(v) => JValue::Int(*v),
            HostArg::Long(v) => JValue::Long(*v),
            HostArg::Float(v) => JValue::Float(*v),
            HostArg::Double(v) => JValue::Double(*v),
            HostArg::Str(_) => {
                let v = JValue::Object(&java_strings[str_i]);
                str_i += 1;
                v
            }
            HostArg::Ints(_) => {
                let v = JValue::Object(&java_int_arrays[ints_i]);
                ints_i += 1;
                v
            }
            HostArg::Bytes(_) => {
                let v = JValue::Object(&java_byte_arrays[bytes_i]);
                bytes_i += 1;
                v
            }
        })
        .collect();
    match env.call_method(cb, name, sig, &jargs) {
        Ok(v) => {
            check_exception(env)?;
            Ok(v)
        }
        Err(e) => {
            // jni-rs leaves a JavaException pending; a later FindClass (Drop /
            // throw_new) aborts ART ("JNI FindClass called with pending exception").
            if env.exception_check().unwrap_or(false) {
                let _ = env.exception_describe();
                let _ = env.exception_clear();
                return Err(format!("host {name} threw"));
            }
            Err(format!("host {name}: {e}"))
        }
    }
}

pub fn call_u32_u32_to_u32(cb: &GlobalRef, a: u32, b: u32) -> Result<u32, String> {
    let cb = cb.clone();
    with_env(move |env| {
        let result = env.call_method(
            cb.as_obj(),
            "invoke",
            "(II)I",
            &[JValue::Int(a as i32), JValue::Int(b as i32)],
        );
        let result = match result {
            Ok(v) => {
                check_exception(env)?;
                v
            }
            Err(e) => {
                if env.exception_check().unwrap_or(false) {
                    let _ = env.exception_describe();
                    let _ = env.exception_clear();
                    return Err("host invoke threw".into());
                }
                return Err(format!("host invoke: {e}"));
            }
        };
        result
            .i()
            .map(|v| v as u32)
            .map_err(|e| format!("host invoke result: {e}"))
    })
}

fn call_i(
    cb: &GlobalRef,
    name: &'static str,
    sig: &'static str,
    args: Vec<HostArg>,
) -> Result<u32, String> {
    let cb = cb.clone();
    with_env(move |env| {
        let result = call_with_host_args(env, cb.as_obj(), name, sig, &args)?;
        check_exception(env)?;
        result
            .i()
            .map(|v| v as u32)
            .map_err(|e| format!("host {name} result: {e}"))
    })
}

fn call_j(
    cb: &GlobalRef,
    name: &'static str,
    sig: &'static str,
    args: Vec<HostArg>,
) -> Result<u64, String> {
    let cb = cb.clone();
    with_env(move |env| {
        let result = call_with_host_args(env, cb.as_obj(), name, sig, &args)?;
        check_exception(env)?;
        result
            .j()
            .map(|v| v as u64)
            .map_err(|e| format!("host {name} result: {e}"))
    })
}

fn call_d(
    cb: &GlobalRef,
    name: &'static str,
    sig: &'static str,
    args: Vec<HostArg>,
) -> Result<f64, String> {
    let cb = cb.clone();
    with_env(move |env| {
        let result = call_with_host_args(env, cb.as_obj(), name, sig, &args)?;
        check_exception(env)?;
        result.d().map_err(|e| format!("host {name} result: {e}"))
    })
}

fn call_void(
    cb: &GlobalRef,
    name: &'static str,
    sig: &'static str,
    args: Vec<HostArg>,
) -> Result<(), String> {
    let cb = cb.clone();
    with_env(move |env| {
        call_with_host_args(env, cb.as_obj(), name, sig, &args)?;
        check_exception(env)
    })
}

fn call_bytes(
    cb: &GlobalRef,
    name: &'static str,
    sig: &'static str,
    args: Vec<HostArg>,
) -> Result<Vec<u8>, String> {
    let cb = cb.clone();
    with_env(move |env| {
        let result = call_with_host_args(env, cb.as_obj(), name, sig, &args)?;
        check_exception(env)?;
        let obj = result.l().map_err(|e| format!("host {name} result: {e}"))?;
        let arr = JByteArray::from(obj);
        env.convert_byte_array(&arr)
            .map_err(|e| format!("host {name} convert_byte_array: {e}"))
    })
}

fn call_string(
    cb: &GlobalRef,
    name: &'static str,
    sig: &'static str,
    args: Vec<HostArg>,
) -> Result<String, String> {
    let cb = cb.clone();
    with_env(move |env| {
        let result = call_with_host_args(env, cb.as_obj(), name, sig, &args)?;
        check_exception(env)?;
        let obj = result.l().map_err(|e| format!("host {name} result: {e}"))?;
        let jstr = JString::from(obj);
        env.get_string(&jstr)
            .map(|s| s.into())
            .map_err(|e| format!("host {name} get_string: {e}"))
    })
}

pub fn exp_request_adapter(cb: &GlobalRef) -> Result<u32, String> {
    call_i(cb, "requestAdapter", "()I", vec![])
}

/// L2: Guest `gpu.request-adapter`. `power_preference`: 0 none/undefined, 1 low-power,
/// 2 high-performance. `force_fallback`: 0 none/false, 1 true.
/// `feature_level` empty = none. `xr_compatible`: -1 none, 0 false, 1 true.
pub fn exp_request_adapter_described(
    cb: &GlobalRef,
    power_preference: i32,
    force_fallback: i32,
    feature_level: String,
    xr_compatible: i32,
) -> Result<u32, String> {
    call_i(
        cb,
        "requestAdapterDescribed",
        "(IILjava/lang/String;I)I",
        vec![
            HostArg::Int(power_preference),
            HostArg::Int(force_fallback),
            HostArg::Str(feature_level),
            HostArg::Int(xr_compatible),
        ],
    )
}

/// L2: Guest `record-gpu-pipeline-constant-value.add` (resource rep + key + f64).
pub fn exp_record_pipeline_constant_value_add_described(
    cb: &GlobalRef,
    handle: u32,
    key: String,
    value: f64,
) -> Result<(), String> {
    call_void(
        cb,
        "recordPipelineConstantValueAddDescribed",
        "(ILjava/lang/String;D)V",
        vec![
            HostArg::Int(handle as i32),
            HostArg::Str(key),
            HostArg::Double(value),
        ],
    )
}

/// L2: Guest `record-gpu-pipeline-constant-value.has` / `get` presence (0/1).
pub fn exp_record_pipeline_constant_value_has_described(
    cb: &GlobalRef,
    handle: u32,
    key: String,
) -> Result<u32, String> {
    call_i(
        cb,
        "recordPipelineConstantValueHasDescribed",
        "(ILjava/lang/String;)I",
        vec![HostArg::Int(handle as i32), HostArg::Str(key)],
    )
}

/// L2: Guest `record-gpu-pipeline-constant-value.get` value when `has` is 1.
pub fn exp_record_pipeline_constant_value_get_value_described(
    cb: &GlobalRef,
    handle: u32,
    key: String,
) -> Result<f64, String> {
    call_d(
        cb,
        "recordPipelineConstantValueGetValueDescribed",
        "(ILjava/lang/String;)D",
        vec![HostArg::Int(handle as i32), HostArg::Str(key)],
    )
}

/// L2: Guest `record-gpu-pipeline-constant-value.remove`.
pub fn exp_record_pipeline_constant_value_remove_described(
    cb: &GlobalRef,
    handle: u32,
    key: String,
) -> Result<(), String> {
    call_void(
        cb,
        "recordPipelineConstantValueRemoveDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(key)],
    )
}

/// L2: Guest `record-gpu-pipeline-constant-value.keys` count.
pub fn exp_record_pipeline_constant_value_keys_count_described(
    cb: &GlobalRef,
    handle: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "recordPipelineConstantValueKeysCountDescribed",
        "(I)I",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest `record-gpu-pipeline-constant-value.keys` entry at index.
pub fn exp_record_pipeline_constant_value_keys_get_described(
    cb: &GlobalRef,
    handle: u32,
    index: u32,
) -> Result<String, String> {
    call_string(
        cb,
        "recordPipelineConstantValueKeysGetDescribed",
        "(II)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32), HostArg::Int(index as i32)],
    )
}

/// L2: Guest `record-gpu-pipeline-constant-value.values` count.
pub fn exp_record_pipeline_constant_value_values_count_described(
    cb: &GlobalRef,
    handle: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "recordPipelineConstantValueValuesCountDescribed",
        "(I)I",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest `record-gpu-pipeline-constant-value.values` entry at index.
pub fn exp_record_pipeline_constant_value_values_get_described(
    cb: &GlobalRef,
    handle: u32,
    index: u32,
) -> Result<f64, String> {
    call_d(
        cb,
        "recordPipelineConstantValueValuesGetDescribed",
        "(II)D",
        vec![HostArg::Int(handle as i32), HostArg::Int(index as i32)],
    )
}

/// L2: Guest `record-gpu-pipeline-constant-value.entries` count.
pub fn exp_record_pipeline_constant_value_entries_count_described(
    cb: &GlobalRef,
    handle: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "recordPipelineConstantValueEntriesCountDescribed",
        "(I)I",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest `record-gpu-pipeline-constant-value.entries` key at index.
pub fn exp_record_pipeline_constant_value_entries_get_key_described(
    cb: &GlobalRef,
    handle: u32,
    index: u32,
) -> Result<String, String> {
    call_string(
        cb,
        "recordPipelineConstantValueEntriesGetKeyDescribed",
        "(II)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32), HostArg::Int(index as i32)],
    )
}

/// L2: Guest `record-gpu-pipeline-constant-value.entries` value at index.
pub fn exp_record_pipeline_constant_value_entries_get_value_described(
    cb: &GlobalRef,
    handle: u32,
    index: u32,
) -> Result<f64, String> {
    call_d(
        cb,
        "recordPipelineConstantValueEntriesGetValueDescribed",
        "(II)D",
        vec![HostArg::Int(handle as i32), HostArg::Int(index as i32)],
    )
}

/// L2: Guest `record-option-gpu-size64.add`. `has_value==0` means `none`.
pub fn exp_record_option_gpu_size64_add_described(
    cb: &GlobalRef,
    handle: u32,
    key: String,
    has_value: i32,
    value: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "recordOptionGpuSize64AddDescribed",
        "(ILjava/lang/String;IJ)V",
        vec![
            HostArg::Int(handle as i32),
            HostArg::Str(key),
            HostArg::Int(has_value),
            HostArg::Long(value as i64),
        ],
    )
}

/// L2: Guest `record-option-gpu-size64.has` (0/1).
pub fn exp_record_option_gpu_size64_has_described(
    cb: &GlobalRef,
    handle: u32,
    key: String,
) -> Result<u32, String> {
    call_i(
        cb,
        "recordOptionGpuSize64HasDescribed",
        "(ILjava/lang/String;)I",
        vec![HostArg::Int(handle as i32), HostArg::Str(key)],
    )
}

/// L2: Guest `record-option-gpu-size64.get` state: 0 missing, 1 present-none, 2 present-some.
pub fn exp_record_option_gpu_size64_get_state_described(
    cb: &GlobalRef,
    handle: u32,
    key: String,
) -> Result<u32, String> {
    call_i(
        cb,
        "recordOptionGpuSize64GetStateDescribed",
        "(ILjava/lang/String;)I",
        vec![HostArg::Int(handle as i32), HostArg::Str(key)],
    )
}

/// L2: Guest `record-option-gpu-size64.get` inner u64 when state is 2.
pub fn exp_record_option_gpu_size64_get_value_described(
    cb: &GlobalRef,
    handle: u32,
    key: String,
) -> Result<u64, String> {
    call_j(
        cb,
        "recordOptionGpuSize64GetValueDescribed",
        "(ILjava/lang/String;)J",
        vec![HostArg::Int(handle as i32), HostArg::Str(key)],
    )
}

/// L2: Guest `record-option-gpu-size64.remove`.
pub fn exp_record_option_gpu_size64_remove_described(
    cb: &GlobalRef,
    handle: u32,
    key: String,
) -> Result<(), String> {
    call_void(
        cb,
        "recordOptionGpuSize64RemoveDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(key)],
    )
}

/// L2: Guest `record-option-gpu-size64.keys` count.
pub fn exp_record_option_gpu_size64_keys_count_described(
    cb: &GlobalRef,
    handle: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "recordOptionGpuSize64KeysCountDescribed",
        "(I)I",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest `record-option-gpu-size64.keys` entry at index.
pub fn exp_record_option_gpu_size64_keys_get_described(
    cb: &GlobalRef,
    handle: u32,
    index: u32,
) -> Result<String, String> {
    call_string(
        cb,
        "recordOptionGpuSize64KeysGetDescribed",
        "(II)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32), HostArg::Int(index as i32)],
    )
}

/// L2: Guest `record-option-gpu-size64.values` count.
pub fn exp_record_option_gpu_size64_values_count_described(
    cb: &GlobalRef,
    handle: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "recordOptionGpuSize64ValuesCountDescribed",
        "(I)I",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest `record-option-gpu-size64.values` option state at index (`0` none, `1` some).
pub fn exp_record_option_gpu_size64_values_get_state_described(
    cb: &GlobalRef,
    handle: u32,
    index: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "recordOptionGpuSize64ValuesGetStateDescribed",
        "(II)I",
        vec![HostArg::Int(handle as i32), HostArg::Int(index as i32)],
    )
}

/// L2: Guest `record-option-gpu-size64.values` u64 when state is 1.
pub fn exp_record_option_gpu_size64_values_get_value_described(
    cb: &GlobalRef,
    handle: u32,
    index: u32,
) -> Result<u64, String> {
    call_j(
        cb,
        "recordOptionGpuSize64ValuesGetValueDescribed",
        "(II)J",
        vec![HostArg::Int(handle as i32), HostArg::Int(index as i32)],
    )
}

/// L2: Guest `record-option-gpu-size64.entries` count.
pub fn exp_record_option_gpu_size64_entries_count_described(
    cb: &GlobalRef,
    handle: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "recordOptionGpuSize64EntriesCountDescribed",
        "(I)I",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest `record-option-gpu-size64.entries` key at index.
pub fn exp_record_option_gpu_size64_entries_get_key_described(
    cb: &GlobalRef,
    handle: u32,
    index: u32,
) -> Result<String, String> {
    call_string(
        cb,
        "recordOptionGpuSize64EntriesGetKeyDescribed",
        "(II)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32), HostArg::Int(index as i32)],
    )
}

/// L2: Guest `record-option-gpu-size64.entries` option state at index (`0` none, `1` some).
pub fn exp_record_option_gpu_size64_entries_get_state_described(
    cb: &GlobalRef,
    handle: u32,
    index: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "recordOptionGpuSize64EntriesGetStateDescribed",
        "(II)I",
        vec![HostArg::Int(handle as i32), HostArg::Int(index as i32)],
    )
}

/// L2: Guest `record-option-gpu-size64.entries` u64 when state is 1.
pub fn exp_record_option_gpu_size64_entries_get_value_described(
    cb: &GlobalRef,
    handle: u32,
    index: u32,
) -> Result<u64, String> {
    call_j(
        cb,
        "recordOptionGpuSize64EntriesGetValueDescribed",
        "(II)J",
        vec![HostArg::Int(handle as i32), HostArg::Int(index as i32)],
    )
}

pub fn exp_adapter_request_device(cb: &GlobalRef, adapter: u32) -> Result<u32, String> {
    call_i(
        cb,
        "adapterRequestDevice",
        "(I)I",
        vec![HostArg::Int(adapter as i32)],
    )
}

/// L2: Guest `gpu-adapter.request-device`. `has_feature==0` means no required-features
/// (descriptor none / empty). `required_limits` 0 = none; `label` empty = none.
pub fn exp_adapter_request_device_described(
    cb: &GlobalRef,
    adapter: u32,
    has_feature: u32,
    feature: u32,
    required_limits: i32,
    label: String,
) -> Result<u32, String> {
    call_i(
        cb,
        "adapterRequestDeviceDescribed",
        "(IIIILjava/lang/String;)I",
        vec![
            HostArg::Int(adapter as i32),
            HostArg::Int(has_feature as i32),
            HostArg::Int(feature as i32),
            HostArg::Int(required_limits),
            HostArg::Str(label),
        ],
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

/// L2: Guest `gpu-device.queue` uses the device handle (0 → stub-create in the wrap).
pub fn exp_device_get_queue_described(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceGetQueueDescribed",
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

/// L2: Guest optional encoder label (none → empty string).
pub fn exp_create_command_encoder_described(
    cb: &GlobalRef,
    device: u32,
    label: String,
) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateCommandEncoderDescribed",
        "(ILjava/lang/String;)I",
        vec![HostArg::Int(device as i32), HostArg::Str(label)],
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

/// S4: Guest-decoded `gpu-buffer-descriptor` size/usage plus mapped-at-creation
/// (`-1` = none, `0` = false, `1` = true) and label (empty → none).
pub fn exp_create_buffer_described(
    cb: &GlobalRef,
    device: u32,
    size: u64,
    usage: u32,
    mapped_at_creation: i32,
    label: String,
) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateBufferDescribed",
        "(IJIILjava/lang/String;)I",
        vec![
            HostArg::Int(device as i32),
            HostArg::Long(size as i64),
            HostArg::Int(usage as i32),
            HostArg::Int(mapped_at_creation),
            HostArg::Str(label),
        ],
    )
}

/// Host-fixed map-async (no guest mode/offset/size). Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_buffer_map_async(cb: &GlobalRef, buffer: u32) -> Result<(), String> {
    call_void(
        cb,
        "bufferMapAsync",
        "(I)V",
        vec![HostArg::Int(buffer as i32)],
    )
}

/// S6+: Guest `gpu-map-mode` + optional offset/size forwarded to L2.
pub fn exp_buffer_map_async_described(
    cb: &GlobalRef,
    buffer: u32,
    mode: u32,
    offset: u64,
    size: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "bufferMapAsyncDescribed",
        "(IIJJ)V",
        vec![
            HostArg::Int(buffer as i32),
            HostArg::Int(mode as i32),
            HostArg::Long(offset as i64),
            HostArg::Long(size as i64),
        ],
    )
}

/// Host-fixed map-then-unmap leftover. Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_buffer_unmap(cb: &GlobalRef, buffer: u32) -> Result<(), String> {
    call_void(cb, "bufferUnmap", "(I)V", vec![HostArg::Int(buffer as i32)])
}

/// L2: Guest buffer rep (0 → stub create in the wrap).
pub fn exp_buffer_unmap_described(cb: &GlobalRef, buffer: u32) -> Result<(), String> {
    call_void(
        cb,
        "bufferUnmapDescribed",
        "(I)V",
        vec![HostArg::Int(buffer as i32)],
    )
}

/// L2: Guest buffer handle → size (`gpu-size64-out`).
pub fn exp_buffer_size_described(cb: &GlobalRef, buffer: u32) -> Result<u64, String> {
    call_j(
        cb,
        "bufferSizeDescribed",
        "(I)J",
        vec![HostArg::Int(buffer as i32)],
    )
}

/// L2: Guest buffer handle → WebGPU/Dawn `GPUBufferUsage` bits.
pub fn exp_buffer_usage_described(cb: &GlobalRef, buffer: u32) -> Result<u32, String> {
    call_i(
        cb,
        "bufferUsageDescribed",
        "(I)I",
        vec![HostArg::Int(buffer as i32)],
    )
}

/// L2: Guest buffer handle → WIT `gpu-buffer-map-state` ordinal.
pub fn exp_buffer_map_state_described(cb: &GlobalRef, buffer: u32) -> Result<u32, String> {
    call_i(
        cb,
        "bufferMapStateDescribed",
        "(I)I",
        vec![HostArg::Int(buffer as i32)],
    )
}

/// L2: Guest buffer handle → destroy.
pub fn exp_buffer_destroy_described(cb: &GlobalRef, buffer: u32) -> Result<(), String> {
    call_void(
        cb,
        "bufferDestroyDescribed",
        "(I)V",
        vec![HostArg::Int(buffer as i32)],
    )
}

/// L2: Guest buffer handle → WIT `gpu-buffer.label`.
pub fn exp_buffer_label_described(cb: &GlobalRef, buffer: u32) -> Result<String, String> {
    call_string(
        cb,
        "bufferLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(buffer as i32)],
    )
}

/// L2: Guest buffer handle + label string.
pub fn exp_buffer_set_label_described(
    cb: &GlobalRef,
    buffer: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "bufferSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(buffer as i32), HostArg::Str(label)],
    )
}

/// L2: Guest `gpu-canvas-context.configure` (context/device/format/usage).
/// `context == 0` → host allocates a canvas-context handle (not a product surface).
pub fn exp_canvas_context_configure_described(
    cb: &GlobalRef,
    context: u32,
    device: u32,
    format: u32,
    usage: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "canvasContextConfigureDescribed",
        "(IIII)I",
        vec![
            HostArg::Int(context as i32),
            HostArg::Int(device as i32),
            HostArg::Int(format as i32),
            HostArg::Int(usage as i32),
        ],
    )
}

/// L2: Guest `gpu-canvas-context.unconfigure` (context handle; 0 is a no-op).
pub fn exp_canvas_context_unconfigure_described(
    cb: &GlobalRef,
    context: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "canvasContextUnconfigureDescribed",
        "(I)V",
        vec![HostArg::Int(context as i32)],
    )
}

/// L2: Guest `gpu-canvas-context.get-current-texture` → texture handle.
pub fn exp_canvas_context_get_current_texture_described(
    cb: &GlobalRef,
    context: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "canvasContextGetCurrentTextureDescribed",
        "(I)I",
        vec![HostArg::Int(context as i32)],
    )
}

/// L2: `1` if canvas-context has a stored configuration, else `0`.
pub fn exp_canvas_context_has_configuration_described(
    cb: &GlobalRef,
    context: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "canvasContextHasConfigurationDescribed",
        "(I)I",
        vec![HostArg::Int(context as i32)],
    )
}

/// L2: Stored configure `device` handle (call only when has-configuration is 1).
pub fn exp_canvas_context_configuration_device_described(
    cb: &GlobalRef,
    context: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "canvasContextConfigurationDeviceDescribed",
        "(I)I",
        vec![HostArg::Int(context as i32)],
    )
}

/// L2: Stored configure Dawn `format` (call only when has-configuration is 1).
pub fn exp_canvas_context_configuration_format_described(
    cb: &GlobalRef,
    context: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "canvasContextConfigurationFormatDescribed",
        "(I)I",
        vec![HostArg::Int(context as i32)],
    )
}

/// L2: Stored configure WebGPU `usage` bits (call only when has-configuration is 1).
pub fn exp_canvas_context_configuration_usage_described(
    cb: &GlobalRef,
    context: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "canvasContextConfigurationUsageDescribed",
        "(I)I",
        vec![HostArg::Int(context as i32)],
    )
}

/// L2: Guest gpu-render-pipeline handle → WIT `gpu-render-pipeline.label`.
pub fn exp_render_pipeline_label_described(cb: &GlobalRef, handle: u32) -> Result<String, String> {
    call_string(
        cb,
        "renderPipelineLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-render-pipeline handle + label string.
pub fn exp_render_pipeline_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPipelineSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-render-pass-encoder handle → WIT `gpu-render-pass-encoder.label`.
pub fn exp_render_pass_encoder_label_described(
    cb: &GlobalRef,
    handle: u32,
) -> Result<String, String> {
    call_string(
        cb,
        "renderPassEncoderLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-render-pass-encoder handle + label string.
pub fn exp_render_pass_encoder_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassEncoderSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-render-bundle handle → WIT `gpu-render-bundle.label`.
pub fn exp_render_bundle_label_described(cb: &GlobalRef, handle: u32) -> Result<String, String> {
    call_string(
        cb,
        "renderBundleLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-render-bundle handle + label string.
pub fn exp_render_bundle_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "renderBundleSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-render-bundle-encoder handle → WIT `gpu-render-bundle-encoder.label`.
pub fn exp_render_bundle_encoder_label_described(
    cb: &GlobalRef,
    handle: u32,
) -> Result<String, String> {
    call_string(
        cb,
        "renderBundleEncoderLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-render-bundle-encoder handle + label string.
pub fn exp_render_bundle_encoder_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "renderBundleEncoderSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-compute-pipeline handle → WIT `gpu-compute-pipeline.label`.
pub fn exp_compute_pipeline_label_described(cb: &GlobalRef, handle: u32) -> Result<String, String> {
    call_string(
        cb,
        "computePipelineLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-compute-pipeline handle + label string.
pub fn exp_compute_pipeline_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "computePipelineSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-compute-pass-encoder handle → WIT `gpu-compute-pass-encoder.label`.
pub fn exp_compute_pass_encoder_label_described(
    cb: &GlobalRef,
    handle: u32,
) -> Result<String, String> {
    call_string(
        cb,
        "computePassEncoderLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-compute-pass-encoder handle + label string.
pub fn exp_compute_pass_encoder_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "computePassEncoderSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-command-buffer handle → WIT `gpu-command-buffer.label`.
pub fn exp_command_buffer_label_described(cb: &GlobalRef, handle: u32) -> Result<String, String> {
    call_string(
        cb,
        "commandBufferLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-command-buffer handle + label string.
pub fn exp_command_buffer_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "commandBufferSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-command-encoder handle → WIT `gpu-command-encoder.label`.
pub fn exp_command_encoder_label_described(cb: &GlobalRef, handle: u32) -> Result<String, String> {
    call_string(
        cb,
        "commandEncoderLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-command-encoder handle + label string.
pub fn exp_command_encoder_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "commandEncoderSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-queue handle → WIT `gpu-queue.label`.
pub fn exp_queue_label_described(cb: &GlobalRef, handle: u32) -> Result<String, String> {
    call_string(
        cb,
        "queueLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-queue handle + label string.
pub fn exp_queue_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "queueSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-device handle → WIT `gpu-device.label`.
pub fn exp_device_label_described(cb: &GlobalRef, handle: u32) -> Result<String, String> {
    call_string(
        cb,
        "deviceLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-device handle + label string.
pub fn exp_device_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "deviceSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-query-set handle → WIT `gpu-query-set.label`.
pub fn exp_query_set_label_described(cb: &GlobalRef, handle: u32) -> Result<String, String> {
    call_string(
        cb,
        "querySetLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-query-set handle + label string.
pub fn exp_query_set_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "querySetSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-pipeline-layout handle → WIT `gpu-pipeline-layout.label`.
pub fn exp_pipeline_layout_label_described(cb: &GlobalRef, handle: u32) -> Result<String, String> {
    call_string(
        cb,
        "pipelineLayoutLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-pipeline-layout handle + label string.
pub fn exp_pipeline_layout_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "pipelineLayoutSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-shader-module handle → WIT `gpu-shader-module.label`.
pub fn exp_shader_module_label_described(cb: &GlobalRef, handle: u32) -> Result<String, String> {
    call_string(
        cb,
        "shaderModuleLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-shader-module handle + label string.
pub fn exp_shader_module_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "shaderModuleSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-sampler handle → WIT `gpu-sampler.label`.
pub fn exp_sampler_label_described(cb: &GlobalRef, handle: u32) -> Result<String, String> {
    call_string(
        cb,
        "samplerLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-sampler handle + label string.
pub fn exp_sampler_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "samplerSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-texture-view handle → WIT `gpu-texture-view.label`.
pub fn exp_texture_view_label_described(cb: &GlobalRef, handle: u32) -> Result<String, String> {
    call_string(
        cb,
        "textureViewLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-texture-view handle + label string.
pub fn exp_texture_view_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "textureViewSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-texture handle → WIT `gpu-texture.label`.
pub fn exp_texture_label_described(cb: &GlobalRef, handle: u32) -> Result<String, String> {
    call_string(
        cb,
        "textureLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-texture handle + label string.
pub fn exp_texture_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "textureSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-bind-group-layout handle → WIT `gpu-bind-group-layout.label`.
pub fn exp_bind_group_layout_label_described(
    cb: &GlobalRef,
    handle: u32,
) -> Result<String, String> {
    call_string(
        cb,
        "bindGroupLayoutLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-bind-group-layout handle + label string.
pub fn exp_bind_group_layout_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "bindGroupLayoutSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest gpu-bind-group handle → WIT `gpu-bind-group.label`.
pub fn exp_bind_group_label_described(cb: &GlobalRef, handle: u32) -> Result<String, String> {
    call_string(
        cb,
        "bindGroupLabelDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(handle as i32)],
    )
}

/// L2: Guest gpu-bind-group handle + label string.
pub fn exp_bind_group_set_label_described(
    cb: &GlobalRef,
    handle: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "bindGroupSetLabelDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(handle as i32), HostArg::Str(label)],
    )
}

/// L2: Guest buffer handle + offset/size → mapped-range bytes.
pub fn exp_buffer_get_mapped_range_described(
    cb: &GlobalRef,
    buffer: u32,
    offset: u64,
    size: u64,
) -> Result<Vec<u8>, String> {
    call_bytes(
        cb,
        "bufferGetMappedRangeDescribed",
        "(IJJ)[B",
        vec![
            HostArg::Int(buffer as i32),
            HostArg::Long(offset as i64),
            HostArg::Long(size as i64),
        ],
    )
}

/// L2: Guest buffer handle + data + offset → write mapped range (data length = size).
pub fn exp_buffer_set_mapped_range_described(
    cb: &GlobalRef,
    buffer: u32,
    data: Vec<u8>,
    offset: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "bufferSetMappedRangeDescribed",
        "(I[BJ)V",
        vec![
            HostArg::Int(buffer as i32),
            HostArg::Bytes(data),
            HostArg::Long(offset as i64),
        ],
    )
}

/// L2: Guest query-set handle → destroy.
pub fn exp_query_set_destroy_described(cb: &GlobalRef, query_set: u32) -> Result<(), String> {
    call_void(
        cb,
        "querySetDestroyDescribed",
        "(I)V",
        vec![HostArg::Int(query_set as i32)],
    )
}

/// L2: Guest query-set handle → WIT `gpu-query-type` ordinal.
pub fn exp_query_set_type_described(cb: &GlobalRef, query_set: u32) -> Result<u32, String> {
    call_i(
        cb,
        "querySetTypeDescribed",
        "(I)I",
        vec![HostArg::Int(query_set as i32)],
    )
}

/// L2: Guest query-set handle → count.
pub fn exp_query_set_count_described(cb: &GlobalRef, query_set: u32) -> Result<u32, String> {
    call_i(
        cb,
        "querySetCountDescribed",
        "(I)I",
        vec![HostArg::Int(query_set as i32)],
    )
}

/// L2: Guest encoder + query-set + destination reps (0 → stub in the attach).
pub fn exp_resolve_query_set_described(
    cb: &GlobalRef,
    encoder: u32,
    query_set: u32,
    first_query: u32,
    query_count: u32,
    destination: u32,
    destination_offset: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "commandEncoderResolveQuerySetDescribed",
        "(IIIIIJ)V",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Int(query_set as i32),
            HostArg::Int(first_query as i32),
            HostArg::Int(query_count as i32),
            HostArg::Int(destination as i32),
            HostArg::Long(destination_offset as i64),
        ],
    )
}

/// L2: Guest encoder handle + group label.
pub fn exp_push_debug_group_described(
    cb: &GlobalRef,
    encoder: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "commandEncoderPushDebugGroupDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(encoder as i32), HostArg::Str(label)],
    )
}

/// L2: Guest encoder handle → pop debug group.
pub fn exp_pop_debug_group_described(cb: &GlobalRef, encoder: u32) -> Result<(), String> {
    call_void(
        cb,
        "commandEncoderPopDebugGroupDescribed",
        "(I)V",
        vec![HostArg::Int(encoder as i32)],
    )
}

/// L2: Guest encoder handle + marker label.
pub fn exp_insert_debug_marker_described(
    cb: &GlobalRef,
    encoder: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "commandEncoderInsertDebugMarkerDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(encoder as i32), HostArg::Str(label)],
    )
}

/// L2: Guest adapter handle → host validates before the local features lift.
pub fn exp_adapter_features_described(cb: &GlobalRef, adapter: u32) -> Result<(), String> {
    call_void(
        cb,
        "adapterFeaturesDescribed",
        "(I)V",
        vec![HostArg::Int(adapter as i32)],
    )
}

/// L2: Guest adapter handle → host validates before the local limits lift.
pub fn exp_adapter_limits_described(cb: &GlobalRef, adapter: u32) -> Result<(), String> {
    call_void(
        cb,
        "adapterLimitsDescribed",
        "(I)V",
        vec![HostArg::Int(adapter as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-bind-groups`.
pub fn exp_supported_limits_max_bind_groups_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxBindGroupsDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-bind-groups-plus-vertex-buffers`.
pub fn exp_supported_limits_max_bind_groups_plus_vertex_buffers_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxBindGroupsPlusVertexBuffersDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-bindings-per-bind-group`.
pub fn exp_supported_limits_max_bindings_per_bind_group_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxBindingsPerBindGroupDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-buffer-size`.
pub fn exp_supported_limits_max_buffer_size_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u64, String> {
    call_j(
        cb,
        "supportedLimitsMaxBufferSizeDescribed",
        "(II)J",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-color-attachment-bytes-per-sample`.
pub fn exp_supported_limits_max_color_attachment_bytes_per_sample_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxColorAttachmentBytesPerSampleDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-color-attachments`.
pub fn exp_supported_limits_max_color_attachments_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxColorAttachmentsDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-compute-invocations-per-workgroup`.
pub fn exp_supported_limits_max_compute_invocations_per_workgroup_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxComputeInvocationsPerWorkgroupDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-compute-workgroup-size-x`.
pub fn exp_supported_limits_max_compute_workgroup_size_x_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxComputeWorkgroupSizeXDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-compute-workgroup-size-y`.
pub fn exp_supported_limits_max_compute_workgroup_size_y_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxComputeWorkgroupSizeYDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-compute-workgroup-size-z`.
pub fn exp_supported_limits_max_compute_workgroup_size_z_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxComputeWorkgroupSizeZDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-compute-workgroups-per-dimension`.
pub fn exp_supported_limits_max_compute_workgroups_per_dimension_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxComputeWorkgroupsPerDimensionDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-compute-workgroup-storage-size`.
pub fn exp_supported_limits_max_compute_workgroup_storage_size_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxComputeWorkgroupStorageSizeDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-dynamic-storage-buffers-per-pipeline-layout`.
pub fn exp_supported_limits_max_dynamic_storage_buffers_per_pipeline_layout_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxDynamicStorageBuffersPerPipelineLayoutDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-dynamic-uniform-buffers-per-pipeline-layout`.
pub fn exp_supported_limits_max_dynamic_uniform_buffers_per_pipeline_layout_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxDynamicUniformBuffersPerPipelineLayoutDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-immediate-size`.
pub fn exp_supported_limits_max_immediate_size_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxImmediateSizeDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-inter-stage-shader-variables`.
pub fn exp_supported_limits_max_inter_stage_shader_variables_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxInterStageShaderVariablesDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-sampled-textures-per-shader-stage`.
pub fn exp_supported_limits_max_sampled_textures_per_shader_stage_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxSampledTexturesPerShaderStageDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-samplers-per-shader-stage`.
pub fn exp_supported_limits_max_samplers_per_shader_stage_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxSamplersPerShaderStageDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-storage-buffer-binding-size`.
pub fn exp_supported_limits_max_storage_buffer_binding_size_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u64, String> {
    call_j(
        cb,
        "supportedLimitsMaxStorageBufferBindingSizeDescribed",
        "(II)J",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-storage-buffers-in-fragment-stage`.
pub fn exp_supported_limits_max_storage_buffers_in_fragment_stage_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxStorageBuffersInFragmentStageDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-storage-buffers-in-vertex-stage`.
pub fn exp_supported_limits_max_storage_buffers_in_vertex_stage_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxStorageBuffersInVertexStageDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-storage-buffers-per-shader-stage`.
pub fn exp_supported_limits_max_storage_buffers_per_shader_stage_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxStorageBuffersPerShaderStageDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-storage-textures-in-fragment-stage`.
pub fn exp_supported_limits_max_storage_textures_in_fragment_stage_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxStorageTexturesInFragmentStageDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-storage-textures-in-vertex-stage`.
pub fn exp_supported_limits_max_storage_textures_in_vertex_stage_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxStorageTexturesInVertexStageDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-storage-textures-per-shader-stage`.
pub fn exp_supported_limits_max_storage_textures_per_shader_stage_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxStorageTexturesPerShaderStageDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-texture-array-layers`.
pub fn exp_supported_limits_max_texture_array_layers_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxTextureArrayLayersDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-texture-dimension1-d`.
pub fn exp_supported_limits_max_texture_dimension1_d_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxTextureDimension1DDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-texture-dimension2-d`.
pub fn exp_supported_limits_max_texture_dimension2_d_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxTextureDimension2DDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-texture-dimension3-d`.
pub fn exp_supported_limits_max_texture_dimension3_d_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxTextureDimension3DDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-uniform-buffer-binding-size`.
pub fn exp_supported_limits_max_uniform_buffer_binding_size_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u64, String> {
    call_j(
        cb,
        "supportedLimitsMaxUniformBufferBindingSizeDescribed",
        "(II)J",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-uniform-buffers-per-shader-stage`.
pub fn exp_supported_limits_max_uniform_buffers_per_shader_stage_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxUniformBuffersPerShaderStageDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-vertex-attributes`.
pub fn exp_supported_limits_max_vertex_attributes_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxVertexAttributesDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-vertex-buffer-array-stride`.
pub fn exp_supported_limits_max_vertex_buffer_array_stride_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxVertexBufferArrayStrideDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.max-vertex-buffers`.
pub fn exp_supported_limits_max_vertex_buffers_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMaxVertexBuffersDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.min-storage-buffer-offset-alignment`.
pub fn exp_supported_limits_min_storage_buffer_offset_alignment_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMinStorageBufferOffsetAlignmentDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter/device handles → WIT `gpu-supported-limits.min-uniform-buffer-offset-alignment`.
pub fn exp_supported_limits_min_uniform_buffer_offset_alignment_described(
    cb: &GlobalRef,
    adapter: u32,
    device: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedLimitsMinUniformBufferOffsetAlignmentDescribed",
        "(II)I",
        vec![HostArg::Int(adapter as i32), HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter handle → host validates before the local adapter-info lift.
pub fn exp_adapter_info_described(cb: &GlobalRef, adapter: u32) -> Result<(), String> {
    call_void(
        cb,
        "adapterInfoDescribed",
        "(I)V",
        vec![HostArg::Int(adapter as i32)],
    )
}

/// L2: Guest adapter handle → WIT `gpu-adapter-info.subgroup-min-size`.
pub fn exp_adapter_info_subgroup_min_size_described(
    cb: &GlobalRef,
    adapter: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "adapterInfoSubgroupMinSizeDescribed",
        "(I)I",
        vec![HostArg::Int(adapter as i32)],
    )
}

/// L2: Guest adapter handle → WIT `gpu-adapter-info.subgroup-max-size`.
pub fn exp_adapter_info_subgroup_max_size_described(
    cb: &GlobalRef,
    adapter: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "adapterInfoSubgroupMaxSizeDescribed",
        "(I)I",
        vec![HostArg::Int(adapter as i32)],
    )
}

/// L2: Guest adapter handle → WIT `gpu-adapter-info.is-fallback-adapter` (0/1).
pub fn exp_adapter_info_is_fallback_adapter_described(
    cb: &GlobalRef,
    adapter: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "adapterInfoIsFallbackAdapterDescribed",
        "(I)I",
        vec![HostArg::Int(adapter as i32)],
    )
}

/// L2: Guest device handle → owning adapter handle (for adapter-info getters).
pub fn exp_device_adapter_described(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceAdapterDescribed",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest adapter handle → WIT `gpu-adapter-info.vendor`.
pub fn exp_adapter_info_vendor_described(cb: &GlobalRef, adapter: u32) -> Result<String, String> {
    call_string(
        cb,
        "adapterInfoVendorDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(adapter as i32)],
    )
}

/// L2: Guest adapter handle → WIT `gpu-adapter-info.architecture`.
pub fn exp_adapter_info_architecture_described(
    cb: &GlobalRef,
    adapter: u32,
) -> Result<String, String> {
    call_string(
        cb,
        "adapterInfoArchitectureDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(adapter as i32)],
    )
}

/// L2: Guest adapter handle → WIT `gpu-adapter-info.device`.
pub fn exp_adapter_info_device_described(cb: &GlobalRef, adapter: u32) -> Result<String, String> {
    call_string(
        cb,
        "adapterInfoDeviceDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(adapter as i32)],
    )
}

/// L2: Guest adapter handle → WIT `gpu-adapter-info.description`.
pub fn exp_adapter_info_description_described(
    cb: &GlobalRef,
    adapter: u32,
) -> Result<String, String> {
    call_string(
        cb,
        "adapterInfoDescriptionDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(adapter as i32)],
    )
}

/// L2: Guest device handle → host validates before the local features lift.
pub fn exp_device_features_described(cb: &GlobalRef, device: u32) -> Result<(), String> {
    call_void(
        cb,
        "deviceFeaturesDescribed",
        "(I)V",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest device handle → host validates before the local limits lift.
pub fn exp_device_limits_described(cb: &GlobalRef, device: u32) -> Result<(), String> {
    call_void(
        cb,
        "deviceLimitsDescribed",
        "(I)V",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest device handle → host validates before the local adapter-info lift.
pub fn exp_device_adapter_info_described(cb: &GlobalRef, device: u32) -> Result<(), String> {
    call_void(
        cb,
        "deviceAdapterInfoDescribed",
        "(I)V",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest device handle → destroy.
pub fn exp_device_destroy_described(cb: &GlobalRef, device: u32) -> Result<(), String> {
    call_void(
        cb,
        "deviceDestroyDescribed",
        "(I)V",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest device handle → host validate (lost future stays local pending).
pub fn exp_device_lost_described(cb: &GlobalRef, device: u32) -> Result<(), String> {
    call_void(
        cb,
        "deviceLostDescribed",
        "(I)V",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest device handle → WIT `gpu-device-lost-info.reason` ordinal.
pub fn exp_device_lost_info_reason_described(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceLostInfoReasonDescribed",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest device handle → WIT `gpu-device-lost-info.message`.
pub fn exp_device_lost_info_message_described(
    cb: &GlobalRef,
    device: u32,
) -> Result<String, String> {
    call_string(
        cb,
        "deviceLostInfoMessageDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest device handle → WIT `gpu-error.kind` ordinal.
pub fn exp_gpu_error_kind_described(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "gpuErrorKindDescribed",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest device handle → WIT `gpu-error.message`.
pub fn exp_gpu_error_message_described(cb: &GlobalRef, device: u32) -> Result<String, String> {
    call_string(
        cb,
        "gpuErrorMessageDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest shader-module handle → WIT `gpu-compilation-message.type` ordinal.
pub fn exp_compilation_message_type_described(cb: &GlobalRef, shader: u32) -> Result<u32, String> {
    call_i(
        cb,
        "compilationMessageTypeDescribed",
        "(I)I",
        vec![HostArg::Int(shader as i32)],
    )
}

/// L2: Guest shader-module handle → WIT `gpu-compilation-message.line-num`.
pub fn exp_compilation_message_line_num_described(
    cb: &GlobalRef,
    shader: u32,
) -> Result<u64, String> {
    call_j(
        cb,
        "compilationMessageLineNumDescribed",
        "(I)J",
        vec![HostArg::Int(shader as i32)],
    )
}

/// L2: Guest shader-module handle → WIT `gpu-compilation-message.line-pos`.
pub fn exp_compilation_message_line_pos_described(
    cb: &GlobalRef,
    shader: u32,
) -> Result<u64, String> {
    call_j(
        cb,
        "compilationMessageLinePosDescribed",
        "(I)J",
        vec![HostArg::Int(shader as i32)],
    )
}

/// L2: Guest shader-module handle → WIT `gpu-compilation-message.offset`.
pub fn exp_compilation_message_offset_described(
    cb: &GlobalRef,
    shader: u32,
) -> Result<u64, String> {
    call_j(
        cb,
        "compilationMessageOffsetDescribed",
        "(I)J",
        vec![HostArg::Int(shader as i32)],
    )
}

/// L2: Guest shader-module handle → WIT `gpu-compilation-message.length`.
pub fn exp_compilation_message_length_described(
    cb: &GlobalRef,
    shader: u32,
) -> Result<u64, String> {
    call_j(
        cb,
        "compilationMessageLengthDescribed",
        "(I)J",
        vec![HostArg::Int(shader as i32)],
    )
}

/// L2: Guest shader-module handle → WIT `gpu-compilation-message.message`.
pub fn exp_compilation_message_message_described(
    cb: &GlobalRef,
    shader: u32,
) -> Result<String, String> {
    call_string(
        cb,
        "compilationMessageMessageDescribed",
        "(I)Ljava/lang/String;",
        vec![HostArg::Int(shader as i32)],
    )
}

/// L2: Guest shader-module handle → message count for `gpu-compilation-info.messages`.
pub fn exp_compilation_info_messages_count_described(
    cb: &GlobalRef,
    shader: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "compilationInfoMessagesCountDescribed",
        "(I)I",
        vec![HostArg::Int(shader as i32)],
    )
}

/// L2: Guest adapter handle + feature name → `gpu-supported-features.has`.
pub fn exp_supported_features_has_described(
    cb: &GlobalRef,
    adapter: u32,
    value: String,
) -> Result<u32, String> {
    call_i(
        cb,
        "supportedFeaturesHasDescribed",
        "(ILjava/lang/String;)I",
        vec![HostArg::Int(adapter as i32), HostArg::Str(value)],
    )
}

/// L2: Guest feature name → `wgsl-language-features.has`.
pub fn exp_wgsl_language_features_has_described(
    cb: &GlobalRef,
    value: String,
) -> Result<u32, String> {
    call_i(
        cb,
        "wgslLanguageFeaturesHasDescribed",
        "(Ljava/lang/String;)I",
        vec![HostArg::Str(value)],
    )
}

/// L2: `gpu.get-preferred-canvas-format` → Dawn `GPUTextureFormat` ordinal.
pub fn exp_gpu_get_preferred_canvas_format_described(cb: &GlobalRef) -> Result<u32, String> {
    call_i(cb, "gpuGetPreferredCanvasFormatDescribed", "()I", vec![])
}

/// L2: `gpu.wgsl-language-features` creator → host validates before local lift.
pub fn exp_gpu_wgsl_language_features_described(cb: &GlobalRef) -> Result<(), String> {
    call_void(cb, "gpuWgslLanguageFeaturesDescribed", "()V", vec![])
}

/// L2: Guest device handle + `gpu-error-filter` ordinal.
pub fn exp_device_push_error_scope_described(
    cb: &GlobalRef,
    device: u32,
    filter: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "devicePushErrorScopeDescribed",
        "(II)V",
        vec![HostArg::Int(device as i32), HostArg::Int(filter as i32)],
    )
}

/// L2: Guest device handle → popped error ordinal (0 = none).
pub fn exp_device_pop_error_scope_described(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "devicePopErrorScopeDescribed",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest device handle → host validate (uncaptured-error stream stays local empty).
pub fn exp_device_on_uncaptured_error_described(cb: &GlobalRef, device: u32) -> Result<(), String> {
    call_void(
        cb,
        "deviceOnUncapturedErrorDescribed",
        "(I)V",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest device handle → host validate before lifting `gpu-error` from event.
pub fn exp_uncaptured_error_event_error_described(
    cb: &GlobalRef,
    device: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "uncapturedErrorEventErrorDescribed",
        "(I)V",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest queue handle → host validate (completion future stays local ready).
pub fn exp_queue_on_submitted_work_done_described(
    cb: &GlobalRef,
    queue: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "queueOnSubmittedWorkDoneDescribed",
        "(I)V",
        vec![HostArg::Int(queue as i32)],
    )
}

/// L2: Guest shader-module handle → host validate (compilation-info stays local lift).
pub fn exp_shader_module_get_compilation_info_described(
    cb: &GlobalRef,
    shader: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "shaderModuleGetCompilationInfoDescribed",
        "(I)V",
        vec![HostArg::Int(shader as i32)],
    )
}

/// L2: Guest render-pipeline rep (0 → stub in the attach) + group index → BGL rep.
pub fn exp_render_pipeline_get_bind_group_layout_described(
    cb: &GlobalRef,
    pipeline: u32,
    index: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "renderPipelineGetBindGroupLayoutDescribed",
        "(II)I",
        vec![HostArg::Int(pipeline as i32), HostArg::Int(index as i32)],
    )
}

/// L2: Guest compute-pass handle + group label.
pub fn exp_compute_pass_push_debug_group_described(
    cb: &GlobalRef,
    pass: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "computePassPushDebugGroupDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(pass as i32), HostArg::Str(label)],
    )
}

/// L2: Guest compute-pass handle → pop debug group.
pub fn exp_compute_pass_pop_debug_group_described(cb: &GlobalRef, pass: u32) -> Result<(), String> {
    call_void(
        cb,
        "computePassPopDebugGroupDescribed",
        "(I)V",
        vec![HostArg::Int(pass as i32)],
    )
}

/// L2: Guest compute-pass handle + marker label.
pub fn exp_compute_pass_insert_debug_marker_described(
    cb: &GlobalRef,
    pass: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "computePassInsertDebugMarkerDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(pass as i32), HostArg::Str(label)],
    )
}

/// L2: Guest compute-pass handle + immediates (range offset, bytes, data offset).
pub fn exp_compute_pass_set_immediates_described(
    cb: &GlobalRef,
    pass: u32,
    range_offset: u32,
    data: Vec<u8>,
    data_offset: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "computePassSetImmediatesDescribed",
        "(II[BJ)V",
        vec![
            HostArg::Int(pass as i32),
            HostArg::Int(range_offset as i32),
            HostArg::Bytes(data),
            HostArg::Long(data_offset as i64),
        ],
    )
}

/// L2: Guest bundle-encoder + optional bundle label → bundle rep.
pub fn exp_render_bundle_encoder_finish_described(
    cb: &GlobalRef,
    encoder: u32,
    label: String,
) -> Result<u32, String> {
    call_i(
        cb,
        "renderBundleEncoderFinishDescribed",
        "(ILjava/lang/String;)I",
        vec![HostArg::Int(encoder as i32), HostArg::Str(label)],
    )
}

/// L2: Guest bundle-encoder + draw counts.
pub fn exp_render_bundle_encoder_draw_described(
    cb: &GlobalRef,
    encoder: u32,
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "renderBundleEncoderDrawDescribed",
        "(IIIII)V",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Int(vertex_count as i32),
            HostArg::Int(instance_count as i32),
            HostArg::Int(first_vertex as i32),
            HostArg::Int(first_instance as i32),
        ],
    )
}

/// L2: Guest bundle-encoder + indexed draw counts.
#[allow(clippy::too_many_arguments)]
pub fn exp_render_bundle_encoder_draw_indexed_described(
    cb: &GlobalRef,
    encoder: u32,
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "renderBundleEncoderDrawIndexedDescribed",
        "(IIIIII)V",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Int(index_count as i32),
            HostArg::Int(instance_count as i32),
            HostArg::Int(first_index as i32),
            HostArg::Int(base_vertex),
            HostArg::Int(first_instance as i32),
        ],
    )
}

/// L2: Guest bundle-encoder + pipeline rep (0 → stub in the attach).
pub fn exp_render_bundle_encoder_set_pipeline_described(
    cb: &GlobalRef,
    encoder: u32,
    pipeline: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "renderBundleEncoderSetPipelineDescribed",
        "(II)V",
        vec![HostArg::Int(encoder as i32), HostArg::Int(pipeline as i32)],
    )
}

/// L2: Guest bundle-encoder + vertex-buffer slot/rep/offset/size (0 → stub in the attach).
pub fn exp_render_bundle_encoder_set_vertex_buffer_described(
    cb: &GlobalRef,
    encoder: u32,
    slot: u32,
    buffer: u32,
    offset: u64,
    size: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "renderBundleEncoderSetVertexBufferDescribed",
        "(IIIJJ)V",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Int(slot as i32),
            HostArg::Int(buffer as i32),
            HostArg::Long(offset as i64),
            HostArg::Long(size as i64),
        ],
    )
}

/// L2: Guest bundle-encoder + index-buffer rep/format/offset/size (0 → stub in the attach).
pub fn exp_render_bundle_encoder_set_index_buffer_described(
    cb: &GlobalRef,
    encoder: u32,
    buffer: u32,
    format: u32,
    offset: u64,
    size: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "renderBundleEncoderSetIndexBufferDescribed",
        "(IIIJJ)V",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Int(buffer as i32),
            HostArg::Int(format as i32),
            HostArg::Long(offset as i64),
            HostArg::Long(size as i64),
        ],
    )
}

/// L2: Guest bundle-encoder + bind-group index/rep (0 → stub in the attach).
pub fn exp_render_bundle_encoder_set_bind_group_described(
    cb: &GlobalRef,
    encoder: u32,
    index: u32,
    bind_group: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "renderBundleEncoderSetBindGroupDescribed",
        "(III)V",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Int(index as i32),
            HostArg::Int(bind_group as i32),
        ],
    )
}

/// L2: Guest bundle-encoder + indirect buffer rep/offset (0 → stub in the attach).
pub fn exp_render_bundle_encoder_draw_indirect_described(
    cb: &GlobalRef,
    encoder: u32,
    buffer: u32,
    offset: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "renderBundleEncoderDrawIndirectDescribed",
        "(IIJ)V",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Int(buffer as i32),
            HostArg::Long(offset as i64),
        ],
    )
}

/// L2: Guest bundle-encoder + indexed-indirect buffer rep/offset (0 → stub in the attach).
pub fn exp_render_bundle_encoder_draw_indexed_indirect_described(
    cb: &GlobalRef,
    encoder: u32,
    buffer: u32,
    offset: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "renderBundleEncoderDrawIndexedIndirectDescribed",
        "(IIJ)V",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Int(buffer as i32),
            HostArg::Long(offset as i64),
        ],
    )
}

/// L2: Guest bundle-encoder + group label.
pub fn exp_render_bundle_encoder_push_debug_group_described(
    cb: &GlobalRef,
    encoder: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "renderBundleEncoderPushDebugGroupDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(encoder as i32), HostArg::Str(label)],
    )
}

/// L2: Guest bundle-encoder → pop debug group.
pub fn exp_render_bundle_encoder_pop_debug_group_described(
    cb: &GlobalRef,
    encoder: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "renderBundleEncoderPopDebugGroupDescribed",
        "(I)V",
        vec![HostArg::Int(encoder as i32)],
    )
}

/// L2: Guest bundle-encoder + marker label.
pub fn exp_render_bundle_encoder_insert_debug_marker_described(
    cb: &GlobalRef,
    encoder: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "renderBundleEncoderInsertDebugMarkerDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(encoder as i32), HostArg::Str(label)],
    )
}

/// L2: Guest bundle-encoder + immediates (range offset, bytes, data offset).
pub fn exp_render_bundle_encoder_set_immediates_described(
    cb: &GlobalRef,
    encoder: u32,
    range_offset: u32,
    data: Vec<u8>,
    data_offset: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "renderBundleEncoderSetImmediatesDescribed",
        "(II[BJ)V",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Int(range_offset as i32),
            HostArg::Bytes(data),
            HostArg::Long(data_offset as i64),
        ],
    )
}

/// L2: Guest compute-pipeline rep (0 → stub in the attach) + group index → BGL rep.
pub fn exp_compute_pipeline_get_bind_group_layout_described(
    cb: &GlobalRef,
    pipeline: u32,
    index: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "computePipelineGetBindGroupLayoutDescribed",
        "(II)I",
        vec![HostArg::Int(pipeline as i32), HostArg::Int(index as i32)],
    )
}

/// L2: Guest render-pass handle + viewport floats.
#[allow(clippy::too_many_arguments)]
pub fn exp_render_pass_set_viewport_described(
    cb: &GlobalRef,
    pass: u32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    min_depth: f32,
    max_depth: f32,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassSetViewportDescribed",
        "(IFFFFFF)V",
        vec![
            HostArg::Int(pass as i32),
            HostArg::Float(x),
            HostArg::Float(y),
            HostArg::Float(width),
            HostArg::Float(height),
            HostArg::Float(min_depth),
            HostArg::Float(max_depth),
        ],
    )
}

/// L2: Guest render-pass handle + scissor rect.
pub fn exp_render_pass_set_scissor_rect_described(
    cb: &GlobalRef,
    pass: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassSetScissorRectDescribed",
        "(IIIII)V",
        vec![
            HostArg::Int(pass as i32),
            HostArg::Int(x as i32),
            HostArg::Int(y as i32),
            HostArg::Int(width as i32),
            HostArg::Int(height as i32),
        ],
    )
}

/// L2: Guest render-pass handle + blend constant color.
pub fn exp_render_pass_set_blend_constant_described(
    cb: &GlobalRef,
    pass: u32,
    r: f64,
    g: f64,
    b: f64,
    a: f64,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassSetBlendConstantDescribed",
        "(IDDDD)V",
        vec![
            HostArg::Int(pass as i32),
            HostArg::Double(r),
            HostArg::Double(g),
            HostArg::Double(b),
            HostArg::Double(a),
        ],
    )
}

/// L2: Guest render-pass handle + stencil reference.
pub fn exp_render_pass_set_stencil_reference_described(
    cb: &GlobalRef,
    pass: u32,
    reference: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassSetStencilReferenceDescribed",
        "(II)V",
        vec![HostArg::Int(pass as i32), HostArg::Int(reference as i32)],
    )
}

/// L2: Guest render-pass handle + occlusion query index.
pub fn exp_render_pass_begin_occlusion_query_described(
    cb: &GlobalRef,
    pass: u32,
    query_index: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassBeginOcclusionQueryDescribed",
        "(II)V",
        vec![HostArg::Int(pass as i32), HostArg::Int(query_index as i32)],
    )
}

/// L2: Guest render-pass handle → end occlusion query.
pub fn exp_render_pass_end_occlusion_query_described(
    cb: &GlobalRef,
    pass: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassEndOcclusionQueryDescribed",
        "(I)V",
        vec![HostArg::Int(pass as i32)],
    )
}

/// L2: Guest render-pass handle + bundle reps (0 entries skipped in the attach).
pub fn exp_render_pass_execute_bundles_described(
    cb: &GlobalRef,
    pass: u32,
    bundles: Vec<i32>,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassExecuteBundlesDescribed",
        "(I[I)V",
        vec![HostArg::Int(pass as i32), HostArg::Ints(bundles)],
    )
}

/// L2: Guest render-pass handle + group label.
pub fn exp_render_pass_push_debug_group_described(
    cb: &GlobalRef,
    pass: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassPushDebugGroupDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(pass as i32), HostArg::Str(label)],
    )
}

/// L2: Guest render-pass handle → pop debug group.
pub fn exp_render_pass_pop_debug_group_described(cb: &GlobalRef, pass: u32) -> Result<(), String> {
    call_void(
        cb,
        "renderPassPopDebugGroupDescribed",
        "(I)V",
        vec![HostArg::Int(pass as i32)],
    )
}

/// L2: Guest render-pass handle + marker label.
pub fn exp_render_pass_insert_debug_marker_described(
    cb: &GlobalRef,
    pass: u32,
    label: String,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassInsertDebugMarkerDescribed",
        "(ILjava/lang/String;)V",
        vec![HostArg::Int(pass as i32), HostArg::Str(label)],
    )
}

/// L2: Guest render-pass handle + immediates (range offset, bytes, data offset).
pub fn exp_render_pass_set_immediates_described(
    cb: &GlobalRef,
    pass: u32,
    range_offset: u32,
    data: Vec<u8>,
    data_offset: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassSetImmediatesDescribed",
        "(II[BJ)V",
        vec![
            HostArg::Int(pass as i32),
            HostArg::Int(range_offset as i32),
            HostArg::Bytes(data),
            HostArg::Long(data_offset as i64),
        ],
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

/// Host-fixed occlusion query-set (count 1) for lift-only getter stubs.
pub fn exp_create_query_set(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateQuerySet",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest-decoded `gpu-query-set-descriptor` type ordinal + count.
pub fn exp_create_query_set_described(
    cb: &GlobalRef,
    device: u32,
    query_type: u32,
    count: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateQuerySetDescribed",
        "(III)I",
        vec![
            HostArg::Int(device as i32),
            HostArg::Int(query_type as i32),
            HostArg::Int(count as i32),
        ],
    )
}

/// L2: Guest-decoded bundle-encoder descriptor (first color format Dawn int + sample count).
pub fn exp_create_render_bundle_encoder_described(
    cb: &GlobalRef,
    device: u32,
    color_format: u32,
    sample_count: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateRenderBundleEncoderDescribed",
        "(III)I",
        vec![
            HostArg::Int(device as i32),
            HostArg::Int(color_format as i32),
            HostArg::Int(sample_count as i32),
        ],
    )
}

/// S6+: Guest-decoded `gpu-texture-descriptor` size/format/usage plus
/// mip-level-count / sample-count / Dawn `TextureDimension` plus
/// view-formats (empty → none) and label (empty → none).
pub fn exp_create_texture_described(
    cb: &GlobalRef,
    device: u32,
    width: u32,
    height: u32,
    depth: u32,
    format: u32,
    usage: u32,
    mip_level_count: u32,
    sample_count: u32,
    dimension: u32,
    view_formats: Vec<i32>,
    label: String,
) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateTextureDescribed",
        "(IIIIIIIII[ILjava/lang/String;)I",
        vec![
            HostArg::Int(device as i32),
            HostArg::Int(width as i32),
            HostArg::Int(height as i32),
            HostArg::Int(depth as i32),
            HostArg::Int(format as i32),
            HostArg::Int(usage as i32),
            HostArg::Int(mip_level_count as i32),
            HostArg::Int(sample_count as i32),
            HostArg::Int(dimension as i32),
            HostArg::Ints(view_formats),
            HostArg::Str(label),
        ],
    )
}

/// Host-fixed sampler (no guest record). Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_create_sampler(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateSampler",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest-decoded `gpu-sampler-descriptor` mag/min + address-mode u/v/w +
/// mipmap + compare (Dawn ints) + optional lod clamps (`hasLod*` 0 = absent).
pub fn exp_create_sampler_described(
    cb: &GlobalRef,
    device: u32,
    mag_filter: u32,
    min_filter: u32,
    address_mode_u: u32,
    address_mode_v: u32,
    address_mode_w: u32,
    mipmap_filter: u32,
    compare: u32,
    has_lod_min: i32,
    lod_min_clamp: f32,
    has_lod_max: i32,
    lod_max_clamp: f32,
) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateSamplerDescribed",
        "(IIIIIIIIIFIF)I",
        vec![
            HostArg::Int(device as i32),
            HostArg::Int(mag_filter as i32),
            HostArg::Int(min_filter as i32),
            HostArg::Int(address_mode_u as i32),
            HostArg::Int(address_mode_v as i32),
            HostArg::Int(address_mode_w as i32),
            HostArg::Int(mipmap_filter as i32),
            HostArg::Int(compare as i32),
            HostArg::Int(has_lod_min),
            HostArg::Float(lod_min_clamp),
            HostArg::Int(has_lod_max),
            HostArg::Float(lod_max_clamp),
        ],
    )
}

/// Host-fixed stub WGSL leftover. Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_create_shader_module(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateShaderModule",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest WGSL `code` plus label (empty → none) and compilation-hints
/// (empty entry-points → none; layouts: -1 none, 0 auto, >0 specific handle).
pub fn exp_create_shader_module_described(
    cb: &GlobalRef,
    device: u32,
    code: String,
    label: String,
    hint_layouts: Vec<i32>,
    hint_entries: String,
) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateShaderModuleDescribed",
        "(ILjava/lang/String;Ljava/lang/String;[ILjava/lang/String;)I",
        vec![
            HostArg::Int(device as i32),
            HostArg::Str(code),
            HostArg::Str(label),
            HostArg::Ints(hint_layouts),
            HostArg::Str(hint_entries),
        ],
    )
}

/// Host-fixed empty-layout leftover. Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_create_bind_group_layout(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateBindGroupLayout",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest bind-group-layout entries (parallel arrays; -1 = that option absent).
/// buffer: 0=uniform, 1=storage, 2=read-only-storage.
/// sampler: 0=filtering, 1=non-filtering, 2=comparison.
/// texture: 0=float, 1=unfilterable-float, 2=depth, 3=sint, 4=uint.
pub fn exp_create_bind_group_layout_described(
    cb: &GlobalRef,
    device: u32,
    bindings: Vec<i32>,
    visibilities: Vec<i32>,
    buffer_types: Vec<i32>,
    sampler_types: Vec<i32>,
    texture_sample_types: Vec<i32>,
) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateBindGroupLayoutDescribed",
        "(I[I[I[I[I[I)I",
        vec![
            HostArg::Int(device as i32),
            HostArg::Ints(bindings),
            HostArg::Ints(visibilities),
            HostArg::Ints(buffer_types),
            HostArg::Ints(sampler_types),
            HostArg::Ints(texture_sample_types),
        ],
    )
}

/// Host-fixed empty pipeline-layout leftover. Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_create_pipeline_layout(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreatePipelineLayout",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest bind-group-layout handles + optional label (none → empty string).
pub fn exp_create_pipeline_layout_described(
    cb: &GlobalRef,
    device: u32,
    bind_group_layouts: Vec<i32>,
    label: String,
) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreatePipelineLayoutDescribed",
        "(I[ILjava/lang/String;)I",
        vec![
            HostArg::Int(device as i32),
            HostArg::Ints(bind_group_layouts),
            HostArg::Str(label),
        ],
    )
}

/// Host-fixed empty bind-group leftover. Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_create_bind_group(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateBindGroup",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest layout handle + bind-group entries (binding/kind/handle arrays) + optional label.
/// kind: 0 = buffer, 1 = sampler, 2 = texture-view.
pub fn exp_create_bind_group_described(
    cb: &GlobalRef,
    device: u32,
    layout: u32,
    label: String,
    bindings: Vec<i32>,
    kinds: Vec<i32>,
    handles: Vec<i32>,
) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateBindGroupDescribed",
        "(IILjava/lang/String;[I[I[I)I",
        vec![
            HostArg::Int(device as i32),
            HostArg::Int(layout as i32),
            HostArg::Str(label),
            HostArg::Ints(bindings),
            HostArg::Ints(kinds),
            HostArg::Ints(handles),
        ],
    )
}

/// Host-fixed stub shader + triangle leftover. Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_create_render_pipeline(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateRenderPipeline",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest vertex/fragment shaders + entry-points + color format (0 = host RGBA8)
/// + layout (0 = auto) + label + vertex.buffers (stride/step/attributes)
/// + vertex/fragment `record-gpu-pipeline-constant-value` reps (0 = none)
/// + primitive (topology/strip/front/cull) + multisample + per-target blend tuples.
pub fn exp_create_render_pipeline_described(
    cb: &GlobalRef,
    device: u32,
    vertex_shader: u32,
    vertex_entry: String,
    fragment_shader: i32,
    fragment_entry: String,
    format: i32,
    layout: i32,
    label: String,
    vb_strides: Vec<i32>,
    vb_step_modes: Vec<i32>,
    attr_buffer_index: Vec<i32>,
    attr_formats: Vec<i32>,
    attr_offsets: Vec<i32>,
    attr_locations: Vec<i32>,
    vertex_constants: i32,
    fragment_constants: i32,
    primitive: Vec<i32>,
    multisample: Vec<i32>,
    blend: Vec<i32>,
) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateRenderPipelineDescribed",
        "(IILjava/lang/String;ILjava/lang/String;IILjava/lang/String;[I[I[I[I[I[III[I[I[I)I",
        vec![
            HostArg::Int(device as i32),
            HostArg::Int(vertex_shader as i32),
            HostArg::Str(vertex_entry),
            HostArg::Int(fragment_shader),
            HostArg::Str(fragment_entry),
            HostArg::Int(format),
            HostArg::Int(layout),
            HostArg::Str(label),
            HostArg::Ints(vb_strides),
            HostArg::Ints(vb_step_modes),
            HostArg::Ints(attr_buffer_index),
            HostArg::Ints(attr_formats),
            HostArg::Ints(attr_offsets),
            HostArg::Ints(attr_locations),
            HostArg::Int(vertex_constants),
            HostArg::Int(fragment_constants),
            HostArg::Ints(primitive),
            HostArg::Ints(multisample),
            HostArg::Ints(blend),
        ],
    )
}

/// Host-fixed stub shader + empty layout leftover. Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_create_compute_pipeline(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateComputePipeline",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

/// L2: Guest shader handle + entry-point + layout handle (0 = auto) + optional label
/// + compute `record-gpu-pipeline-constant-value` rep (0 = none).
pub fn exp_create_compute_pipeline_described(
    cb: &GlobalRef,
    device: u32,
    shader: u32,
    entry_point: String,
    layout: i32,
    label: String,
    constants: i32,
) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateComputePipelineDescribed",
        "(IILjava/lang/String;ILjava/lang/String;I)I",
        vec![
            HostArg::Int(device as i32),
            HostArg::Int(shader as i32),
            HostArg::Str(entry_point),
            HostArg::Int(layout),
            HostArg::Str(label),
            HostArg::Int(constants),
        ],
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

/// L2: Guest encoder + timestamp-write indices (none → 0/0).
pub fn exp_begin_compute_pass_described(
    cb: &GlobalRef,
    encoder: u32,
    beginning_of_pass_write_index: u32,
    end_of_pass_write_index: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "beginComputePassDescribed",
        "(III)I",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Int(beginning_of_pass_write_index as i32),
            HostArg::Int(end_of_pass_write_index as i32),
        ],
    )
}

pub fn exp_begin_render_pass_clear(cb: &GlobalRef, encoder: u32, view: u32) -> Result<u32, String> {
    call_i(
        cb,
        "beginRenderPassClear",
        "(II)I",
        vec![HostArg::Int(encoder as i32), HostArg::Int(view as i32)],
    )
}

/// L2: Guest encoder + all color attachments (view/load/store + optional clear
/// bits) + depth-stencil. `view==0` is a WIT `none` slot.
pub fn exp_begin_render_pass_described(
    cb: &GlobalRef,
    encoder: u32,
    views: Vec<i32>,
    load_ops: Vec<i32>,
    store_ops: Vec<i32>,
    has_clears: Vec<i32>,
    clear_bits: Vec<i32>,
    depth_view: u32,
    depth_load: i32,
    depth_store: i32,
    has_depth_clear: i32,
    depth_clear: f32,
) -> Result<u32, String> {
    call_i(
        cb,
        "beginRenderPassDescribed",
        "(I[I[I[I[I[IIIIIIF)I",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Ints(views),
            HostArg::Ints(load_ops),
            HostArg::Ints(store_ops),
            HostArg::Ints(has_clears),
            HostArg::Ints(clear_bits),
            HostArg::Int(depth_view as i32),
            HostArg::Int(depth_load),
            HostArg::Int(depth_store),
            HostArg::Int(has_depth_clear),
            HostArg::Float(depth_clear),
        ],
    )
}

pub fn exp_render_pass_end(cb: &GlobalRef, pass: u32) -> Result<(), String> {
    call_void(cb, "renderPassEnd", "(I)V", vec![HostArg::Int(pass as i32)])
}

/// Host-fixed triangle pipeline (ignores guest pipeline). Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_render_pass_set_pipeline(cb: &GlobalRef, pass: u32) -> Result<(), String> {
    call_void(
        cb,
        "renderPassSetPipeline",
        "(I)V",
        vec![HostArg::Int(pass as i32)],
    )
}

/// L2: Guest pass + pipeline reps (0 → stub in attach).
pub fn exp_render_pass_set_pipeline_described(
    cb: &GlobalRef,
    pass: u32,
    pipeline: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassSetPipelineDescribed",
        "(II)V",
        vec![HostArg::Int(pass as i32), HostArg::Int(pipeline as i32)],
    )
}

/// Host-fixed draw(3) leftover. Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_render_pass_draw(cb: &GlobalRef, pass: u32) -> Result<(), String> {
    call_void(
        cb,
        "renderPassDraw",
        "(I)V",
        vec![HostArg::Int(pass as i32)],
    )
}

/// L2: Guest pass rep + vertex-count / option instance-count / first-vertex / first-instance
/// (none → 1 / 0 / 0).
pub fn exp_render_pass_draw_described(
    cb: &GlobalRef,
    pass: u32,
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassDrawDescribed",
        "(IIIII)V",
        vec![
            HostArg::Int(pass as i32),
            HostArg::Int(vertex_count as i32),
            HostArg::Int(instance_count as i32),
            HostArg::Int(first_vertex as i32),
            HostArg::Int(first_instance as i32),
        ],
    )
}

/// L2: Guest pass/buffer reps + indirect-offset.
pub fn exp_render_pass_draw_indirect_described(
    cb: &GlobalRef,
    pass: u32,
    buffer: u32,
    offset: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassDrawIndirectDescribed",
        "(IIJ)V",
        vec![
            HostArg::Int(pass as i32),
            HostArg::Int(buffer as i32),
            HostArg::Long(offset as i64),
        ],
    )
}

/// L2: Guest pass/buffer reps + indirect-offset (indexed).
pub fn exp_render_pass_draw_indexed_indirect_described(
    cb: &GlobalRef,
    pass: u32,
    buffer: u32,
    offset: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassDrawIndexedIndirectDescribed",
        "(IIJ)V",
        vec![
            HostArg::Int(pass as i32),
            HostArg::Int(buffer as i32),
            HostArg::Long(offset as i64),
        ],
    )
}

/// L2: Guest pass rep + index-count / option instance-count / first-index / base-vertex /
/// first-instance (none → 1 / 0 / 0 / 0).
pub fn exp_render_pass_draw_indexed_described(
    cb: &GlobalRef,
    pass: u32,
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassDrawIndexedDescribed",
        "(IIIIII)V",
        vec![
            HostArg::Int(pass as i32),
            HostArg::Int(index_count as i32),
            HostArg::Int(instance_count as i32),
            HostArg::Int(first_index as i32),
            HostArg::Int(base_vertex),
            HostArg::Int(first_instance as i32),
        ],
    )
}

/// L2: Guest pass rep (0 → smoke rebuild in the wrap).
pub fn exp_render_pass_end_described(cb: &GlobalRef, pass: u32) -> Result<(), String> {
    call_void(
        cb,
        "renderPassEndDescribed",
        "(I)V",
        vec![HostArg::Int(pass as i32)],
    )
}

/// Host-fixed empty bind-group (ignores guest index/group). Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_render_pass_set_bind_group(cb: &GlobalRef, pass: u32) -> Result<(), String> {
    call_void(
        cb,
        "renderPassSetBindGroup",
        "(I)V",
        vec![HostArg::Int(pass as i32)],
    )
}

/// L2: Guest pass/bind-group reps + index (offsets none this slice → empty on host).
pub fn exp_render_pass_set_bind_group_described(
    cb: &GlobalRef,
    pass: u32,
    index: u32,
    bind_group: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassSetBindGroupDescribed",
        "(III)V",
        vec![
            HostArg::Int(pass as i32),
            HostArg::Int(index as i32),
            HostArg::Int(bind_group as i32),
        ],
    )
}

/// Host-fixed VERTEX slot 0 (ignores guest slot/buffer). Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_render_pass_set_vertex_buffer(cb: &GlobalRef, pass: u32) -> Result<(), String> {
    call_void(
        cb,
        "renderPassSetVertexBuffer",
        "(I)V",
        vec![HostArg::Int(pass as i32)],
    )
}

/// L2: Guest pass/buffer reps + slot + option offset/size (none → 0).
pub fn exp_render_pass_set_vertex_buffer_described(
    cb: &GlobalRef,
    pass: u32,
    slot: u32,
    buffer: u32,
    offset: u64,
    size: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassSetVertexBufferDescribed",
        "(IIIJJ)V",
        vec![
            HostArg::Int(pass as i32),
            HostArg::Int(slot as i32),
            HostArg::Int(buffer as i32),
            HostArg::Long(offset as i64),
            HostArg::Long(size as i64),
        ],
    )
}

/// L2: Guest pass/buffer reps + Dawn index-format + option offset/size (none → 0).
pub fn exp_render_pass_set_index_buffer_described(
    cb: &GlobalRef,
    pass: u32,
    buffer: u32,
    format: u32,
    offset: u64,
    size: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "renderPassSetIndexBufferDescribed",
        "(IIIJJ)V",
        vec![
            HostArg::Int(pass as i32),
            HostArg::Int(buffer as i32),
            HostArg::Int(format as i32),
            HostArg::Long(offset as i64),
            HostArg::Long(size as i64),
        ],
    )
}

/// Host-fixed begin-then-end (ignores guest pass). Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_compute_pass_end(cb: &GlobalRef, pass: u32) -> Result<(), String> {
    call_void(
        cb,
        "computePassEnd",
        "(I)V",
        vec![HostArg::Int(pass as i32)],
    )
}

/// L2: Guest compute-pass rep (0 → smoke rebuild in the wrap).
pub fn exp_compute_pass_end_described(cb: &GlobalRef, pass: u32) -> Result<(), String> {
    call_void(
        cb,
        "computePassEndDescribed",
        "(I)V",
        vec![HostArg::Int(pass as i32)],
    )
}

/// Host-fixed stub compute pipeline (ignores guest pipeline). Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_compute_pass_set_pipeline(cb: &GlobalRef, pass: u32) -> Result<(), String> {
    call_void(
        cb,
        "computePassSetPipeline",
        "(I)V",
        vec![HostArg::Int(pass as i32)],
    )
}

/// L2: Guest compute-pass + pipeline reps (0 → stub in attach).
pub fn exp_compute_pass_set_pipeline_described(
    cb: &GlobalRef,
    pass: u32,
    pipeline: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "computePassSetPipelineDescribed",
        "(II)V",
        vec![HostArg::Int(pass as i32), HostArg::Int(pipeline as i32)],
    )
}

/// Host-fixed empty bind-group (ignores guest index/group). Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_compute_pass_set_bind_group(cb: &GlobalRef, pass: u32) -> Result<(), String> {
    call_void(
        cb,
        "computePassSetBindGroup",
        "(I)V",
        vec![HostArg::Int(pass as i32)],
    )
}

/// L2: Guest pass/bind-group reps + index (offsets none this slice → empty on host).
pub fn exp_compute_pass_set_bind_group_described(
    cb: &GlobalRef,
    pass: u32,
    index: u32,
    bind_group: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "computePassSetBindGroupDescribed",
        "(III)V",
        vec![
            HostArg::Int(pass as i32),
            HostArg::Int(index as i32),
            HostArg::Int(bind_group as i32),
        ],
    )
}

/// Host-fixed dispatch(1,1,1) leftover. Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_compute_pass_dispatch_workgroups(cb: &GlobalRef, pass: u32) -> Result<(), String> {
    call_void(
        cb,
        "computePassDispatchWorkgroups",
        "(I)V",
        vec![HostArg::Int(pass as i32)],
    )
}

/// L2: Guest pass rep + workgroup-count-x / option y/z (none → 1).
pub fn exp_compute_pass_dispatch_workgroups_described(
    cb: &GlobalRef,
    pass: u32,
    x: u32,
    y: u32,
    z: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "computePassDispatchWorkgroupsDescribed",
        "(IIII)V",
        vec![
            HostArg::Int(pass as i32),
            HostArg::Int(x as i32),
            HostArg::Int(y as i32),
            HostArg::Int(z as i32),
        ],
    )
}

/// L2: Guest pass/buffer reps + indirect-offset.
pub fn exp_compute_pass_dispatch_workgroups_indirect_described(
    cb: &GlobalRef,
    pass: u32,
    buffer: u32,
    offset: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "computePassDispatchWorkgroupsIndirectDescribed",
        "(IIJ)V",
        vec![
            HostArg::Int(pass as i32),
            HostArg::Int(buffer as i32),
            HostArg::Long(offset as i64),
        ],
    )
}

/// Host-fixed 4-byte copy (ignores guest buffers). Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_copy_buffer_to_buffer(cb: &GlobalRef, encoder: u32) -> Result<(), String> {
    call_void(
        cb,
        "commandEncoderCopyBufferToBuffer",
        "(I)V",
        vec![HostArg::Int(encoder as i32)],
    )
}

/// L2: Guest encoder/buffer reps + option offsets/size (none → 0).
pub fn exp_copy_buffer_to_buffer_described(
    cb: &GlobalRef,
    encoder: u32,
    source: u32,
    source_offset: u64,
    destination: u32,
    destination_offset: u64,
    size: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "commandEncoderCopyBufferToBufferDescribed",
        "(IIJIJJ)V",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Int(source as i32),
            HostArg::Long(source_offset as i64),
            HostArg::Int(destination as i32),
            HostArg::Long(destination_offset as i64),
            HostArg::Long(size as i64),
        ],
    )
}

/// L2: Guest encoder/buffer reps + option offset/size (none → 0).
pub fn exp_clear_buffer_described(
    cb: &GlobalRef,
    encoder: u32,
    buffer: u32,
    offset: u64,
    size: u64,
) -> Result<(), String> {
    call_void(
        cb,
        "commandEncoderClearBufferDescribed",
        "(IIJJ)V",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Int(buffer as i32),
            HostArg::Long(offset as i64),
            HostArg::Long(size as i64),
        ],
    )
}

/// L2: Guest encoder/buffer/texture reps + copy-size (option height/depth → 1).
pub fn exp_copy_buffer_to_texture_described(
    cb: &GlobalRef,
    encoder: u32,
    source: u32,
    destination: u32,
    width: u32,
    height: u32,
    depth: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "commandEncoderCopyBufferToTextureDescribed",
        "(IIIIII)V",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Int(source as i32),
            HostArg::Int(destination as i32),
            HostArg::Int(width as i32),
            HostArg::Int(height as i32),
            HostArg::Int(depth as i32),
        ],
    )
}

/// L2: Guest encoder/texture/buffer reps + copy-size (option height/depth → 1).
pub fn exp_copy_texture_to_buffer_described(
    cb: &GlobalRef,
    encoder: u32,
    source: u32,
    destination: u32,
    width: u32,
    height: u32,
    depth: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "commandEncoderCopyTextureToBufferDescribed",
        "(IIIIII)V",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Int(source as i32),
            HostArg::Int(destination as i32),
            HostArg::Int(width as i32),
            HostArg::Int(height as i32),
            HostArg::Int(depth as i32),
        ],
    )
}

/// L2: Guest encoder/texture reps + copy-size (option height/depth → 1).
pub fn exp_copy_texture_to_texture_described(
    cb: &GlobalRef,
    encoder: u32,
    source: u32,
    destination: u32,
    width: u32,
    height: u32,
    depth: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "commandEncoderCopyTextureToTextureDescribed",
        "(IIIIII)V",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Int(source as i32),
            HostArg::Int(destination as i32),
            HostArg::Int(width as i32),
            HostArg::Int(height as i32),
            HostArg::Int(depth as i32),
        ],
    )
}

pub fn exp_command_encoder_finish(cb: &GlobalRef, encoder: u32) -> Result<u32, String> {
    call_i(
        cb,
        "commandEncoderFinish",
        "(I)I",
        vec![HostArg::Int(encoder as i32)],
    )
}

/// L2: Guest optional command-buffer label (none → empty string).
pub fn exp_command_encoder_finish_described(
    cb: &GlobalRef,
    encoder: u32,
    label: String,
) -> Result<u32, String> {
    call_i(
        cb,
        "commandEncoderFinishDescribed",
        "(ILjava/lang/String;)I",
        vec![HostArg::Int(encoder as i32), HostArg::Str(label)],
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

/// L2: Guest `list<borrow<gpu-command-buffer>>` handles (empty → empty `IntArray`).
pub fn exp_queue_submit_described(
    cb: &GlobalRef,
    queue: u32,
    command_buffers: Vec<i32>,
) -> Result<(), String> {
    call_void(
        cb,
        "queueSubmitDescribed",
        "(I[I)V",
        vec![HostArg::Int(queue as i32), HostArg::Ints(command_buffers)],
    )
}

/// Host-fixed 4-byte write leftover. Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_queue_write_buffer(cb: &GlobalRef, queue: u32, buffer: u32) -> Result<(), String> {
    call_void(
        cb,
        "queueWriteBuffer",
        "(II)V",
        vec![HostArg::Int(queue as i32), HostArg::Int(buffer as i32)],
    )
}

/// L2: Guest buffer handle + offset + `list<u8>` payload (sliced by data-offset/size).
pub fn exp_queue_write_buffer_described(
    cb: &GlobalRef,
    queue: u32,
    buffer: u32,
    buffer_offset: u64,
    data: Vec<u8>,
) -> Result<(), String> {
    call_void(
        cb,
        "queueWriteBufferDescribed",
        "(IIJ[B)V",
        vec![
            HostArg::Int(queue as i32),
            HostArg::Int(buffer as i32),
            HostArg::Long(buffer_offset as i64),
            HostArg::Bytes(data),
        ],
    )
}

/// Host-fixed 1×1 write leftover. Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_queue_write_texture(cb: &GlobalRef, queue: u32, texture: u32) -> Result<(), String> {
    call_void(
        cb,
        "queueWriteTexture",
        "(II)V",
        vec![HostArg::Int(queue as i32), HostArg::Int(texture as i32)],
    )
}

/// L2: Guest texture handle + `list<u8>` + copy size / bytes-per-row.
pub fn exp_queue_write_texture_described(
    cb: &GlobalRef,
    queue: u32,
    texture: u32,
    data: Vec<u8>,
    width: u32,
    height: u32,
    bytes_per_row: u32,
) -> Result<(), String> {
    call_void(
        cb,
        "queueWriteTextureDescribed",
        "(II[BIII)V",
        vec![
            HostArg::Int(queue as i32),
            HostArg::Int(texture as i32),
            HostArg::Bytes(data),
            HostArg::Int(width as i32),
            HostArg::Int(height as i32),
            HostArg::Int(bytes_per_row as i32),
        ],
    )
}

/// Host-fixed texture view (no guest record). Kept for older attach objects.
#[allow(dead_code)]
pub fn exp_texture_create_view(cb: &GlobalRef, texture: u32) -> Result<u32, String> {
    call_i(
        cb,
        "textureCreateView",
        "(I)I",
        vec![HostArg::Int(texture as i32)],
    )
}

/// L2: Guest texture handle → width (rep 0 stub-created in the wrap).
pub fn exp_texture_width_described(cb: &GlobalRef, texture: u32) -> Result<u32, String> {
    call_i(
        cb,
        "textureWidthDescribed",
        "(I)I",
        vec![HostArg::Int(texture as i32)],
    )
}

/// L2: Guest texture handle → height.
pub fn exp_texture_height_described(cb: &GlobalRef, texture: u32) -> Result<u32, String> {
    call_i(
        cb,
        "textureHeightDescribed",
        "(I)I",
        vec![HostArg::Int(texture as i32)],
    )
}

/// L2: Guest texture handle → depth-or-array-layers.
pub fn exp_texture_depth_or_array_layers_described(
    cb: &GlobalRef,
    texture: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "textureDepthOrArrayLayersDescribed",
        "(I)I",
        vec![HostArg::Int(texture as i32)],
    )
}

/// L2: Guest texture handle → mip-level-count.
pub fn exp_texture_mip_level_count_described(cb: &GlobalRef, texture: u32) -> Result<u32, String> {
    call_i(
        cb,
        "textureMipLevelCountDescribed",
        "(I)I",
        vec![HostArg::Int(texture as i32)],
    )
}

/// L2: Guest texture handle → sample-count.
pub fn exp_texture_sample_count_described(cb: &GlobalRef, texture: u32) -> Result<u32, String> {
    call_i(
        cb,
        "textureSampleCountDescribed",
        "(I)I",
        vec![HostArg::Int(texture as i32)],
    )
}

/// L2: Guest texture handle → Dawn `TextureDimension` int.
pub fn exp_texture_dimension_described(cb: &GlobalRef, texture: u32) -> Result<u32, String> {
    call_i(
        cb,
        "textureDimensionDescribed",
        "(I)I",
        vec![HostArg::Int(texture as i32)],
    )
}

/// L2: Guest texture handle → Dawn `TextureFormat` int.
pub fn exp_texture_format_described(cb: &GlobalRef, texture: u32) -> Result<u32, String> {
    call_i(
        cb,
        "textureFormatDescribed",
        "(I)I",
        vec![HostArg::Int(texture as i32)],
    )
}

/// L2: Guest texture handle → WebGPU/Dawn `GPUTextureUsage` bits.
pub fn exp_texture_usage_described(cb: &GlobalRef, texture: u32) -> Result<u32, String> {
    call_i(
        cb,
        "textureUsageDescribed",
        "(I)I",
        vec![HostArg::Int(texture as i32)],
    )
}

/// L2: Guest texture handle → Dawn `TextureViewDimension` (0 = none).
pub fn exp_texture_binding_view_dimension_described(
    cb: &GlobalRef,
    texture: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "textureBindingViewDimensionDescribed",
        "(I)I",
        vec![HostArg::Int(texture as i32)],
    )
}

/// L2: Guest texture handle → destroy.
pub fn exp_texture_destroy_described(cb: &GlobalRef, texture: u32) -> Result<(), String> {
    call_void(
        cb,
        "textureDestroyDescribed",
        "(I)V",
        vec![HostArg::Int(texture as i32)],
    )
}

/// L2: Guest-decoded `gpu-texture-view-descriptor` dimension/aspect/format (Dawn ints)
/// plus mip / array-layer window (`mipLevelCount`/`arrayLayerCount` `-1` = absent).
pub fn exp_texture_create_view_described(
    cb: &GlobalRef,
    texture: u32,
    dimension: u32,
    aspect: u32,
    format: u32,
    base_mip_level: i32,
    mip_level_count: i32,
    base_array_layer: i32,
    array_layer_count: i32,
) -> Result<u32, String> {
    call_i(
        cb,
        "textureCreateViewDescribed",
        "(IIIIIIII)I",
        vec![
            HostArg::Int(texture as i32),
            HostArg::Int(dimension as i32),
            HostArg::Int(aspect as i32),
            HostArg::Int(format as i32),
            HostArg::Int(base_mip_level),
            HostArg::Int(mip_level_count),
            HostArg::Int(base_array_layer),
            HostArg::Int(array_layer_count),
        ],
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
