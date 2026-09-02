//! Product `define_host` wraps on NativeGpu (reuse w1 fixtures; no parallel set).

use crate::cm::define_host;
use crate::host::HostState;
use crate::native_gpu::NativeGpuHost;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

fn native_host() -> HostState {
    let mut host = HostState::default();
    host.native_gpu = Some(NativeGpuHost::new());
    host
}

fn run_w1(wasm_name: &str, fixture_ctors: bool) -> wasmtime::Result<u32> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let path = format!("{}/../fixtures/w1/{wasm_name}", env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(&path)?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<HostState> = Linker::new(&engine);
    define_host(&mut linker, fixture_ctors).map_err(wasmtime::Error::msg)?;
    let mut store = Store::new(&engine, native_host());
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                let func = accessor
                    .with(|mut access| instance.get_typed_func::<(), (u32,)>(&mut access, "run"))?;
                let (value,) = func.call_concurrent(accessor, ()).await?;
                Ok(value)
            })
            .await?
    })
}

#[test]
fn request_adapter_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_request_adapter.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn request_device_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_request_device.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn device_queue_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_device_queue.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn adapter_info_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_adapter_info.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn adapter_features_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_adapter_features.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn adapter_limits_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_adapter_limits.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn create_buffer_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_create_buffer.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn create_texture_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_create_texture.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn create_sampler_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_create_sampler.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn create_shader_module_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_create_shader_module.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn texture_create_view_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_texture_create_view.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn create_bind_group_layout_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_create_bind_group_layout.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn create_pipeline_layout_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_create_pipeline_layout.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn create_bind_group_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_create_bind_group.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn create_compute_pipeline_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_create_compute_pipeline.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn create_compute_pipeline_async_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_create_compute_pipeline_async.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn create_compute_pipeline_constants_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_create_compute_pipeline_constants.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn create_render_pipeline_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_create_render_pipeline.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn create_render_pipeline_async_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_create_render_pipeline_async.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn create_command_encoder_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_create_command_encoder.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn begin_render_pass_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_begin_render_pass.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn render_pass_end_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_render_pass_end.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn command_encoder_finish_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_command_encoder_finish.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn begin_compute_pass_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_begin_compute_pass.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn compute_pass_end_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_compute_pass_end.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn render_pass_draw_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_render_pass_draw.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn copy_buffer_to_buffer_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_copy_buffer_to_buffer.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn create_query_set_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_create_query_set.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn queue_submit_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_queue_submit.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn write_buffer_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_write_buffer.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn write_texture_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_write_texture.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn queue_on_submitted_work_done_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_queue_on_submitted_work_done.wasm", false)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn canvas_context_present_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_canvas_context_present.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn canvas_context_get_configuration_fixture_on_native_gpu() -> wasmtime::Result<()> {
    let v = run_w1("webgpu_method_canvas_context_get_configuration.wasm", true)?;
    assert_eq!(v, 1);
    Ok(())
}

/// ND-REST: every remaining pin `[method]` fixture on product `define_host` + NativeGpu.
#[test]
fn all_w1_method_fixtures_on_native_gpu() -> wasmtime::Result<()> {
    let dir = format!("{}/../fixtures/w1", env!("CARGO_MANIFEST_DIR"));
    let mut names: Vec<String> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("webgpu_method_") && n.ends_with(".wasm"))
        .collect();
    names.sort();
    assert!(
        names.len() >= 200,
        "expected the full w1 method suite, got {}",
        names.len()
    );
    let mut failed = Vec::new();
    for name in &names {
        match run_w1(name, true) {
            Ok(1) => {}
            Ok(other) => failed.push(format!("{name}: run={other}")),
            Err(err) => failed.push(format!("{name}: {err}")),
        }
    }
    assert!(
        failed.is_empty(),
        "NativeGpu wasi_webgpu_method sweep failed ({}):\n{}",
        failed.len(),
        failed.join("\n")
    );
    Ok(())
}

#[test]
fn gfx_surface_size_against_bound_window() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let path = format!(
        "{}/../fixtures/wasi/gfx_size.wasm",
        env!("CARGO_MANIFEST_DIR")
    );
    let bytes = std::fs::read(&path)?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<HostState> = Linker::new(&engine);
    define_host(&mut linker, false).map_err(wasmtime::Error::msg)?;
    let mut host = native_host();
    host.bind_canvas_native_window(0x1000, 64, 48);
    let mut store = Store::new(&engine, host);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let v = pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                let func = accessor
                    .with(|mut access| instance.get_typed_func::<(), (u32,)>(&mut access, "run"))?;
                let (value,) = func.call_concurrent(accessor, ()).await?;
                Ok(value)
            })
            .await?
    })?;
    assert_eq!(v, 1, "guest height/width/request-set-size/on-resize, got {v}");
    let win = store
        .data()
        .native_gpu
        .as_ref()
        .and_then(|g| g.canvas_window())
        .expect("bound window resized");
    assert_eq!(win.native_window, 0x1000);
    assert_eq!(win.width, 80);
    assert_eq!(win.height, 96);
    Ok(())
}
