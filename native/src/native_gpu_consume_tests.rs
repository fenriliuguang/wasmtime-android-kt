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
