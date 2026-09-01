//! Product `wasi:webgpu` consume backend: [`GpuBackend::NativeGpu`] or
//! [`GpuBackend::JniBackend`].
//!
//! ND-DEFAULT: `GpuBackends.dawn()` / `id == "dawn"` selects NativeGpu.
//! Unwired `Store.create` still leaves the slot unset (`request-adapter`
//! `none` via JNI leftover). `dawn-jni` / `setExperimentalHost` keep
//! [`GpuBackend::JniBackend`]. Do not reimplement `jvm::exp_*` here.

use crate::host::HostState;
use crate::native_gpu::NativeGpuHost;
use jni::objects::GlobalRef;

/// Backend selected for pin `wasi:webgpu` imports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GpuBackend {
    NativeGpu,
    JniBackend,
}

impl HostState {
    /// [`GpuBackend::NativeGpu`] when the slot is set, else JNI leftover.
    pub fn webgpu_backend(&self) -> GpuBackend {
        match self.native_gpu {
            Some(_) => GpuBackend::NativeGpu,
            None => GpuBackend::JniBackend,
        }
    }

    /// JNI leftover callback when the backend is [`GpuBackend::JniBackend`].
    /// NativeGpu selected → `None` (consume methods land in ND-BOOT+).
    pub fn webgpu_jni_cb(&self) -> Option<GlobalRef> {
        match self.webgpu_backend() {
            GpuBackend::NativeGpu => None,
            GpuBackend::JniBackend => self.experimental_host_cb.clone(),
        }
    }

    /// Required JNI callback for wraps that already trap when unwired.
    pub fn require_webgpu_jni_cb(&self) -> wasmtime::Result<GlobalRef> {
        match self.webgpu_backend() {
            GpuBackend::NativeGpu => Err(wasmtime::Error::msg(
                "NativeGpu selected; JNI leftover is GpuBackends.dawnJni()",
            )),
            GpuBackend::JniBackend => self
                .experimental_host_cb
                .clone()
                .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set")),
        }
    }

    /// Native consume slot. `None` when JNI is the product default.
    #[allow(dead_code)] // ND-REST+
    pub fn native_gpu_mut(&mut self) -> Option<&mut NativeGpuHost> {
        self.native_gpu.as_mut()
    }

    /// Required NativeGpu host when [`GpuBackend::NativeGpu`] is selected.
    pub fn require_native_gpu(&mut self) -> wasmtime::Result<&mut NativeGpuHost> {
        self.native_gpu
            .as_mut()
            .ok_or_else(|| wasmtime::Error::msg("NativeGpu selected but slot empty"))
    }

    /// `bindCanvasNativeWindow` / Store window handle. Forwards to NativeGpu when set.
    pub fn bind_canvas_native_window(&mut self, window: i64, width: u32, height: u32) {
        self.canvas_native_window = window;
        self.canvas_width = width;
        self.canvas_height = height;
        if let Some(gpu) = self.native_gpu.as_mut() {
            let _ = gpu.bind_canvas_native_window(window, width, height);
        }
    }

    /// Product `GpuBackends.dawn()` / `id == "dawn"`.
    pub fn enable_native_gpu(&mut self) {
        let mut gpu = NativeGpuHost::new();
        if self.canvas_native_window != 0 && self.canvas_width > 0 && self.canvas_height > 0 {
            let _ = gpu.bind_canvas_native_window(
                self.canvas_native_window,
                self.canvas_width,
                self.canvas_height,
            );
        }
        let _ = NativeGpuHost::try_load_dawn_c();
        self.native_gpu = Some(gpu);
        self.experimental_host_cb = None;
    }

    /// Leftover `dawn-jni` / `setExperimentalHost` clears the NativeGpu slot.
    pub fn disable_native_gpu(&mut self) {
        self.native_gpu = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_jni_backend_native_slot_unset() {
        let host = HostState::default();
        assert!(host.native_gpu.is_none());
        assert_eq!(host.webgpu_backend(), GpuBackend::JniBackend);
        assert!(host.webgpu_jni_cb().is_none());
    }

    #[test]
    fn native_slot_selects_native_gpu() {
        let mut host = HostState::default();
        host.native_gpu = Some(NativeGpuHost::default());
        assert_eq!(host.webgpu_backend(), GpuBackend::NativeGpu);
        assert!(host.webgpu_jni_cb().is_none());
        assert!(host.require_webgpu_jni_cb().is_err());
        assert!(host.native_gpu_mut().is_some());
    }

    #[test]
    fn store_window_forwards_to_native_gpu() {
        let mut host = HostState::default();
        host.native_gpu = Some(NativeGpuHost::default());
        host.bind_canvas_native_window(0x2000, 32, 16);
        let win = host
            .native_gpu
            .as_ref()
            .and_then(|g| g.canvas_window())
            .expect("forwarded");
        assert_eq!(win.native_window, 0x2000);
        assert_eq!(win.height, 16);
        assert_eq!(win.buffer_count, crate::native_gpu::SWAPCHAIN_BUFFER_COUNT);
    }

    #[test]
    fn enable_native_gpu_selects_product_default() {
        let mut host = HostState::default();
        host.bind_canvas_native_window(0x3000, 8, 8);
        host.enable_native_gpu();
        assert_eq!(host.webgpu_backend(), GpuBackend::NativeGpu);
        assert!(host.webgpu_jni_cb().is_none());
        let win = host
            .native_gpu
            .as_ref()
            .and_then(|g| g.canvas_window())
            .expect("rebound");
        assert_eq!(win.native_window, 0x3000);
        host.disable_native_gpu();
        assert_eq!(host.webgpu_backend(), GpuBackend::JniBackend);
    }
}
