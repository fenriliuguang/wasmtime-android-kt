//! Product `wasi:webgpu` consume backend: [`GpuBackend::NativeGpu`] or
//! [`GpuBackend::JniBackend`].
//!
//! ND-DISP: dispatch only. Default is JNI so existing tests stay green.
//! [`NativeGpu`] may be unset; trait + handle table land in ND-HOST.
//! Do not reimplement `jvm::exp_*` here.

use crate::host::HostState;
use jni::objects::GlobalRef;

/// In-process Dawn C consume (product path after ND-DEFAULT).
/// Empty occupant until ND-HOST lands the trait + handle table.
#[derive(Debug, Default)]
pub struct NativeGpu {
    _private: (),
}

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
    /// NativeGpu selected → `None` (consume methods land in ND-HOST+).
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
                "NativeGpu selected; consume methods land in ND-HOST",
            )),
            GpuBackend::JniBackend => self
                .experimental_host_cb
                .clone()
                .ok_or_else(|| wasmtime::Error::msg("experimental host callback not set")),
        }
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
        host.native_gpu = Some(NativeGpu::default());
        assert_eq!(host.webgpu_backend(), GpuBackend::NativeGpu);
        assert!(host.webgpu_jni_cb().is_none());
        assert!(host.require_webgpu_jni_cb().is_err());
    }
}
