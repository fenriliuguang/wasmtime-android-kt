//! In-process Dawn C consume: [`NativeGpu`] trait + handle table.
//!
//! Kinds match Kotlin `ResourceKind` in
//! `host-dawn/.../experimental/host/Handles.kt`. This module must not import
//! `jni` — table insert/drop is the ND-HOST smoke that the GPU hot path does
//! not bounce through ART. Dawn C `u64` slots stay 0 until a consume lane
//! dlopens `libwebgpu_dawn.so`. Consume methods land in ND-BOOT+; until then
//! the table is exercised from `#[cfg(test)]` only (cdylib has no rlib).
#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;

/// Dawn C object pointer/id. `0` until a later lane binds `webgpu.h`.
pub type DawnSlot = u64;

/// Opaque WASI-style resource handle (`u32`). Handle `0` is reserved as null.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GpuHandle(u32);

impl GpuHandle {
    pub const NULL: u32 = 0;

    pub fn from_raw(raw: u32) -> Result<Self, NativeGpuError> {
        if raw == Self::NULL {
            return Err(NativeGpuError::InvalidHandle {
                handle: raw,
                message: "handle 0 is reserved as null",
            });
        }
        Ok(Self(raw))
    }

    pub fn raw(self) -> u32 {
        self.0
    }
}

/// GPU resource kinds. Order matches `DawnWasiWebGpuHost` / Kotlin `ResourceKind`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ResourceKind {
    Adapter,
    Device,
    Buffer,
    ShaderModule,
    BindGroupLayout,
    BindGroup,
    PipelineLayout,
    ComputePipeline,
    CommandEncoder,
    ComputePassEncoder,
    CommandBuffer,
    Queue,
    Surface,
    CanvasContext,
    Texture,
    TextureView,
    Sampler,
    RenderPipeline,
    RenderPassEncoder,
    QuerySet,
    RenderBundleEncoder,
    RenderBundle,
}

/// Live table row: kind + optional Dawn C slot (0 = table-backed placeholder).
#[derive(Clone, Copy, Debug)]
pub struct HandleEntry {
    pub kind: ResourceKind,
    pub dawn: DawnSlot,
}

/// In-memory handle table (Kotlin `HandleTable` equivalent). Single-threaded
/// host calls (same model as P0 `docs/mapping/threading.md`).
#[derive(Debug, Default)]
pub struct HandleTable {
    next_id: u32,
    entries: HashMap<u32, HandleEntry>,
}

impl HandleTable {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, kind: ResourceKind, dawn: DawnSlot) -> GpuHandle {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        if self.next_id == GpuHandle::NULL {
            self.next_id = 1;
        }
        self.entries.insert(id, HandleEntry { kind, dawn });
        GpuHandle(id)
    }

    pub fn contains(&self, handle: GpuHandle) -> bool {
        self.entries.contains_key(&handle.0)
    }

    pub fn get(
        &self,
        handle: GpuHandle,
        kind: ResourceKind,
    ) -> Result<&HandleEntry, NativeGpuError> {
        let entry = self
            .entries
            .get(&handle.0)
            .ok_or(NativeGpuError::InvalidHandle {
                handle: handle.0,
                message: "unknown handle",
            })?;
        if entry.kind != kind {
            return Err(NativeGpuError::KindMismatch {
                handle: handle.0,
                expected: kind,
                found: entry.kind,
            });
        }
        Ok(entry)
    }

    pub fn drop_handle(&mut self, handle: GpuHandle) -> Result<HandleEntry, NativeGpuError> {
        self.try_drop(handle).ok_or(NativeGpuError::InvalidHandle {
            handle: handle.0,
            message: "already dropped or unknown",
        })
    }

    pub fn try_drop(&mut self, handle: GpuHandle) -> Option<HandleEntry> {
        self.entries.remove(&handle.0)
    }

    pub fn handles_of_kind(&self, kind: ResourceKind) -> Vec<GpuHandle> {
        let mut out: Vec<GpuHandle> = self
            .entries
            .iter()
            .filter_map(|(&id, entry)| {
                if entry.kind == kind {
                    Some(GpuHandle(id))
                } else {
                    None
                }
            })
            .collect();
        out.sort_by_key(|h| h.0);
        out
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeGpuError {
    InvalidHandle {
        handle: u32,
        message: &'static str,
    },
    KindMismatch {
        handle: u32,
        expected: ResourceKind,
        found: ResourceKind,
    },
}

impl fmt::Display for NativeGpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NativeGpuError::InvalidHandle { handle, message } => {
                write!(f, "invalid GPU handle {handle}: {message}")
            }
            NativeGpuError::KindMismatch {
                handle,
                expected,
                found,
            } => write!(
                f,
                "invalid GPU handle {handle}: expected {expected:?} but found {found:?}"
            ),
        }
    }
}

impl std::error::Error for NativeGpuError {}

/// In-process Dawn C consume (product path after ND-DEFAULT).
pub trait NativeGpu: Send {
    fn insert(&mut self, kind: ResourceKind, dawn: DawnSlot) -> GpuHandle;
    fn contains(&self, handle: GpuHandle) -> bool;
    fn get(&self, handle: GpuHandle, kind: ResourceKind) -> Result<&HandleEntry, NativeGpuError>;
    fn drop_handle(&mut self, handle: GpuHandle) -> Result<HandleEntry, NativeGpuError>;
    fn try_drop(&mut self, handle: GpuHandle) -> Option<HandleEntry>;
    fn handles_of_kind(&self, kind: ResourceKind) -> Vec<GpuHandle>;
    fn size(&self) -> usize;
    fn clear(&mut self);
}

/// Table-backed [`NativeGpu`]. Dawn C slots stay 0 until a consume lane dlopens.
#[derive(Debug, Default)]
pub struct NativeGpuHost {
    table: HandleTable,
}

impl NativeGpuHost {
    pub fn new() -> Self {
        Self {
            table: HandleTable::new(),
        }
    }
}

impl NativeGpu for NativeGpuHost {
    fn insert(&mut self, kind: ResourceKind, dawn: DawnSlot) -> GpuHandle {
        self.table.insert(kind, dawn)
    }

    fn contains(&self, handle: GpuHandle) -> bool {
        self.table.contains(handle)
    }

    fn get(&self, handle: GpuHandle, kind: ResourceKind) -> Result<&HandleEntry, NativeGpuError> {
        self.table.get(handle, kind)
    }

    fn drop_handle(&mut self, handle: GpuHandle) -> Result<HandleEntry, NativeGpuError> {
        self.table.drop_handle(handle)
    }

    fn try_drop(&mut self, handle: GpuHandle) -> Option<HandleEntry> {
        self.table.try_drop(handle)
    }

    fn handles_of_kind(&self, kind: ResourceKind) -> Vec<GpuHandle> {
        self.table.handles_of_kind(kind)
    }

    fn size(&self) -> usize {
        self.table.size()
    }

    fn clear(&mut self) {
        self.table.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_no_jni_in_this_module() {
        // Compile-time: this file has no `use jni`. Runtime smoke: insert/drop
        // never constructs a GlobalRef / JVM attach.
        let _gpu: NativeGpuHost = NativeGpuHost::new();
    }

    #[test]
    fn insert_drop_does_not_jni() {
        assert_no_jni_in_this_module();
        let mut gpu: NativeGpuHost = NativeGpuHost::new();
        let h = NativeGpu::insert(&mut gpu, ResourceKind::Buffer, 0);
        assert_ne!(h.raw(), GpuHandle::NULL);
        assert!(gpu.contains(h));
        assert_eq!(gpu.size(), 1);
        let entry = NativeGpu::drop_handle(&mut gpu, h).expect("drop live handle");
        assert_eq!(entry.kind, ResourceKind::Buffer);
        assert_eq!(entry.dawn, 0);
        assert_eq!(gpu.size(), 0);
        assert!(NativeGpu::try_drop(&mut gpu, h).is_none());
    }

    #[test]
    fn handle_zero_reserved() {
        assert!(GpuHandle::from_raw(0).is_err());
        let mut table = HandleTable::new();
        let h = table.insert(ResourceKind::Adapter, 0);
        assert_eq!(h.raw(), 1);
        assert_ne!(h.raw(), GpuHandle::NULL);
    }

    #[test]
    fn kind_mismatch_and_unknown() {
        let mut gpu = NativeGpuHost::new();
        let h = gpu.insert(ResourceKind::Device, 0);
        let err = gpu.get(h, ResourceKind::Queue).unwrap_err();
        assert!(matches!(
            err,
            NativeGpuError::KindMismatch {
                expected: ResourceKind::Queue,
                found: ResourceKind::Device,
                ..
            }
        ));
        gpu.drop_handle(h).unwrap();
        assert!(matches!(
            gpu.drop_handle(h),
            Err(NativeGpuError::InvalidHandle { .. })
        ));
    }

    #[test]
    fn handles_of_kind_and_clear() {
        let mut gpu = NativeGpuHost::new();
        let a = gpu.insert(ResourceKind::Adapter, 0);
        let _d = gpu.insert(ResourceKind::Device, 0);
        let adapters = gpu.handles_of_kind(ResourceKind::Adapter);
        assert_eq!(adapters, vec![a]);
        gpu.clear();
        assert_eq!(gpu.size(), 0);
        assert!(gpu.handles_of_kind(ResourceKind::Adapter).is_empty());
    }

    #[test]
    fn all_kotlin_resource_kinds_exist() {
        // Keep in lockstep with Handles.kt ResourceKind.
        let kinds = [
            ResourceKind::Adapter,
            ResourceKind::Device,
            ResourceKind::Buffer,
            ResourceKind::ShaderModule,
            ResourceKind::BindGroupLayout,
            ResourceKind::BindGroup,
            ResourceKind::PipelineLayout,
            ResourceKind::ComputePipeline,
            ResourceKind::CommandEncoder,
            ResourceKind::ComputePassEncoder,
            ResourceKind::CommandBuffer,
            ResourceKind::Queue,
            ResourceKind::Surface,
            ResourceKind::CanvasContext,
            ResourceKind::Texture,
            ResourceKind::TextureView,
            ResourceKind::Sampler,
            ResourceKind::RenderPipeline,
            ResourceKind::RenderPassEncoder,
            ResourceKind::QuerySet,
            ResourceKind::RenderBundleEncoder,
            ResourceKind::RenderBundle,
        ];
        assert_eq!(kinds.len(), 22);
        let mut gpu = NativeGpuHost::new();
        for kind in kinds {
            let _ = gpu.insert(kind, 0);
        }
        assert_eq!(gpu.size(), 22);
    }
}
