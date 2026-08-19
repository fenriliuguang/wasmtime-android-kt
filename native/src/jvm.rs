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
            HostArg::Int(_) | HostArg::Long(_) => {}
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
    env.call_method(cb, name, sig, &jargs)
        .map_err(|e| format!("host {name}: {e}"))
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

/// S4: Guest-decoded `gpu-buffer-descriptor` size/usage (mapped/label still unused).
pub fn exp_create_buffer_described(
    cb: &GlobalRef,
    device: u32,
    size: u64,
    usage: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateBufferDescribed",
        "(IJI)I",
        vec![
            HostArg::Int(device as i32),
            HostArg::Long(size as i64),
            HostArg::Int(usage as i32),
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

pub fn exp_create_texture(cb: &GlobalRef, device: u32) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateTexture",
        "(I)I",
        vec![HostArg::Int(device as i32)],
    )
}

/// S6+: Guest-decoded `gpu-texture-descriptor` size/format/usage (Dawn format int).
pub fn exp_create_texture_described(
    cb: &GlobalRef,
    device: u32,
    width: u32,
    height: u32,
    depth: u32,
    format: u32,
    usage: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateTextureDescribed",
        "(IIIIII)I",
        vec![
            HostArg::Int(device as i32),
            HostArg::Int(width as i32),
            HostArg::Int(height as i32),
            HostArg::Int(depth as i32),
            HostArg::Int(format as i32),
            HostArg::Int(usage as i32),
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

/// L2: Guest-decoded `gpu-sampler-descriptor` mag/min filter + address-mode-u (Dawn ints).
pub fn exp_create_sampler_described(
    cb: &GlobalRef,
    device: u32,
    mag_filter: u32,
    min_filter: u32,
    address_mode_u: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateSamplerDescribed",
        "(IIII)I",
        vec![
            HostArg::Int(device as i32),
            HostArg::Int(mag_filter as i32),
            HostArg::Int(min_filter as i32),
            HostArg::Int(address_mode_u as i32),
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

/// L2: Guest WGSL `code` (hints/label still unused).
pub fn exp_create_shader_module_described(
    cb: &GlobalRef,
    device: u32,
    code: String,
) -> Result<u32, String> {
    call_i(
        cb,
        "deviceCreateShaderModuleDescribed",
        "(ILjava/lang/String;)I",
        vec![HostArg::Int(device as i32), HostArg::Str(code)],
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

/// L2: Guest encoder + first color-attachment view/load/store (view 0 → stub in attach).
pub fn exp_begin_render_pass_described(
    cb: &GlobalRef,
    encoder: u32,
    view: u32,
    load_op: u32,
    store_op: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "beginRenderPassDescribed",
        "(IIII)I",
        vec![
            HostArg::Int(encoder as i32),
            HostArg::Int(view as i32),
            HostArg::Int(load_op as i32),
            HostArg::Int(store_op as i32),
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

/// L2: Guest-decoded `gpu-texture-view-descriptor` dimension + aspect (Dawn ints).
pub fn exp_texture_create_view_described(
    cb: &GlobalRef,
    texture: u32,
    dimension: u32,
    aspect: u32,
) -> Result<u32, String> {
    call_i(
        cb,
        "textureCreateViewDescribed",
        "(III)I",
        vec![
            HostArg::Int(texture as i32),
            HostArg::Int(dimension as i32),
            HostArg::Int(aspect as i32),
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
