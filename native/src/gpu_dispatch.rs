//! Product `wasi:webgpu` consume backend: [`GpuBackend::NativeGpu`] or
//! [`GpuBackend::JniBackend`].
//!
//! ND-DISP: dispatch only. Default is JNI so existing tests stay green.
//! ND-HOST: NativeGpu trait + handle table. ND-BOOT: request-adapter /
//! request-device / queue / boot info. Further methods ND-ENC+. ND-SO: Dawn C
//! API `.so` recipe `scripts/build-dawn-c-android.py` (not loaded until
//! ND-DEFAULT). Do not reimplement `jvm::exp_*` here.

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
    /// [`GpuBackend::NativeGpu`] when the slot is set, else JNI (product default).
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
                "NativeGpu selected; consume methods land in ND-ENC",
            )),
            GpuBackend::JniBackend => self
                .experimental_host_cb
                .clone()
                .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set")),
        }
    }

    /// Native consume slot. `None` when JNI is the product default.
    #[allow(dead_code)] // ND-ENC+
    pub fn native_gpu_mut(&mut self) -> Option<&mut NativeGpuHost> {
        self.native_gpu.as_mut()
    }

    /// Required NativeGpu host when [`GpuBackend::NativeGpu`] is selected.
    pub fn require_native_gpu(&mut self) -> wasmtime::Result<&mut NativeGpuHost> {
        self.native_gpu
            .as_mut()
            .ok_or_else(|| wasmtime::Error::msg("NativeGpu selected but slot empty"))
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
}
