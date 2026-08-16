//! WIT types for `wasi:webgpu@0.3.0-rc.2` used by canonical-shape slices.
//! S2: `gpu-request-adapter-options` + `gpu-power-preference`.

use wasmtime::component::{ComponentType, Lift, Lower};

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
pub enum GpuPowerPreference {
    #[component(name = "low-power")]
    LowPower,
    #[component(name = "high-performance")]
    HighPerformance,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
pub struct GpuRequestAdapterOptions {
    #[component(name = "feature-level")]
    pub feature_level: Option<String>,
    #[component(name = "power-preference")]
    pub power_preference: Option<GpuPowerPreference>,
    #[component(name = "force-fallback-adapter")]
    pub force_fallback_adapter: Option<bool>,
    #[component(name = "xr-compatible")]
    pub xr_compatible: Option<bool>,
}
