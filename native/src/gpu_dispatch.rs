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
    /// Posts `on-resize` when the reported size changes (including first bind).
    pub fn bind_canvas_native_window(&mut self, window: i64, width: u32, height: u32) {
        let changed = self.canvas_width != width || self.canvas_height != height;
        self.canvas_native_window = window;
        self.canvas_width = width;
        self.canvas_height = height;
        if let Some(gpu) = self.native_gpu.as_mut() {
            let _ = gpu.bind_canvas_native_window(window, width, height);
        }
        if changed {
            self.gfx_on_resize.post(width, height);
        }
    }

    /// Bound window size, else constructor `create-desc` (0 if neither).
    pub fn surface_width(&self, desc: Option<u32>) -> u32 {
        if self.canvas_width > 0 {
            self.canvas_width
        } else {
            desc.unwrap_or(0)
        }
    }

    /// Bound window size, else constructor `create-desc` (0 if neither).
    pub fn surface_height(&self, desc: Option<u32>) -> u32 {
        if self.canvas_height > 0 {
            self.canvas_height
        } else {
            desc.unwrap_or(0)
        }
    }

    /// Guest `request-set-size`. `none` keeps that axis. Applies to the bound
    /// window record (NativeGpu swapchain size) when a handle is set.
    pub fn request_surface_size(&mut self, height: Option<u32>, width: Option<u32>) {
        let new_w = width.unwrap_or(self.surface_width(None));
        let new_h = height.unwrap_or(self.surface_height(None));
        if new_w == 0 || new_h == 0 {
            return;
        }
        if self.canvas_native_window != 0 {
            self.bind_canvas_native_window(self.canvas_native_window, new_w, new_h);
            return;
        }
        if self.canvas_width == new_w && self.canvas_height == new_h {
            return;
        }
        self.canvas_width = new_w;
        self.canvas_height = new_h;
        self.gfx_on_resize.post(new_w, new_h);
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
        assert_eq!(host.surface_width(None), 32);
        assert_eq!(host.surface_height(None), 16);
        match host.gfx_on_resize.wait_take(true) {
            crate::host::GfxOnResizeTake::Item(sz) => {
                assert_eq!(sz.width, 32);
                assert_eq!(sz.height, 16);
            }
            other => panic!("expected first bind resize, got {other:?}"),
        }
    }

    #[test]
    fn request_set_size_updates_bound_window() {
        let mut host = HostState::default();
        host.native_gpu = Some(NativeGpuHost::default());
        host.bind_canvas_native_window(0x2000, 32, 16);
        let _ = host.gfx_on_resize.wait_take(true);
        host.request_surface_size(Some(48), Some(64));
        assert_eq!(host.surface_width(None), 64);
        assert_eq!(host.surface_height(None), 48);
        let win = host
            .native_gpu
            .as_ref()
            .and_then(|g| g.canvas_window())
            .expect("resized");
        assert_eq!(win.native_window, 0x2000);
        assert_eq!(win.width, 64);
        assert_eq!(win.height, 48);
        match host.gfx_on_resize.wait_take(true) {
            crate::host::GfxOnResizeTake::Item(sz) => {
                assert_eq!(sz.width, 64);
                assert_eq!(sz.height, 48);
            }
            other => panic!("expected request-set-size resize, got {other:?}"),
        }
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

    #[test]
    fn unbound_size_uses_create_desc_then_request() {
        let mut host = HostState::default();
        assert_eq!(host.surface_width(Some(10)), 10);
        assert_eq!(host.surface_height(None), 0);
        host.request_surface_size(Some(20), Some(30));
        assert_eq!(host.surface_width(Some(10)), 30);
        assert_eq!(host.surface_height(Some(9)), 20);
        match host.gfx_on_resize.wait_take(true) {
            crate::host::GfxOnResizeTake::Item(sz) => {
                assert_eq!(sz.width, 30);
                assert_eq!(sz.height, 20);
            }
            other => panic!("expected request-set-size without window, got {other:?}"),
        }
    }
}
