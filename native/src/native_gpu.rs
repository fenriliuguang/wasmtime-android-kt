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
#[derive(Debug)]
pub struct HandleTable {
    next_id: u32,
    entries: HashMap<u32, HandleEntry>,
}

impl Default for HandleTable {
    fn default() -> Self {
        Self::new()
    }
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
    AdapterUnavailable,
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
            NativeGpuError::AdapterUnavailable => {
                write!(f, "NativeGpu request-adapter returned none")
            }
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

    fn request_adapter(&mut self, options: &NativeRequestAdapterOptions<'_>) -> Option<GpuHandle>;
    fn request_device(
        &mut self,
        adapter: GpuHandle,
        desc: &NativeRequestDeviceDescriptor<'_>,
    ) -> Result<GpuHandle, NativeGpuError>;
    fn device_queue(&mut self, device: GpuHandle) -> Result<GpuHandle, NativeGpuError>;
    fn adapter_info(&self, adapter: GpuHandle) -> Result<NativeAdapterInfo, NativeGpuError>;
    fn adapter_has_feature(&self, adapter: GpuHandle, name: &str) -> Result<bool, NativeGpuError>;
}

/// Packed `[method]gpu.request-adapter` options (cm.rs lowering).
#[derive(Clone, Debug, Default)]
pub struct NativeRequestAdapterOptions<'a> {
    pub feature_level: &'a str,
    /// 0 = none, 1 = low-power, 2 = high-performance.
    pub power_preference: i32,
    pub force_fallback_adapter: bool,
    pub xr_compatible: Option<bool>,
}

/// Packed `[method]gpu-adapter.request-device` descriptor (cm.rs lowering).
#[derive(Clone, Debug, Default)]
pub struct NativeRequestDeviceDescriptor<'a> {
    pub required_features: &'a [i32],
    pub required_limits_rep: i32,
    pub label: &'a str,
    pub default_queue_label: &'a str,
}

/// Table-backed adapter info until Dawn C `wgpuAdapterGetInfo`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeAdapterInfo {
    pub vendor: String,
    pub architecture: String,
    pub device: String,
    pub description: String,
    pub subgroup_min_size: u32,
    pub subgroup_max_size: u32,
    pub is_fallback_adapter: bool,
}

/// Shader-module `compilation-hints` leftover. Dawn C has no ctor slot — Record only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeShaderHints {
    pub entries: String,
    pub layouts: Vec<i32>,
}

/// Pipeline `constants` map copied from WIT `record-gpu-pipeline-constant-value`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NativePipelineConstants {
    pub compute: Vec<(String, f64)>,
    pub vertex: Vec<(String, f64)>,
    pub fragment: Vec<(String, f64)>,
}

impl Default for NativeAdapterInfo {
    fn default() -> Self {
        Self {
            vendor: String::new(),
            architecture: String::new(),
            device: "native-gpu".into(),
            description: "table-backed NativeGpu (Dawn C slot 0)".into(),
            subgroup_min_size: 4,
            subgroup_max_size: 128,
            is_fallback_adapter: false,
        }
    }
}

/// Table-backed [`NativeGpu`]. Dawn C slots stay 0 until a consume lane dlopens.
#[derive(Debug)]
pub struct NativeGpuHost {
    table: HandleTable,
    interned_queues: HashMap<u32, u32>,
    adapter_info: HashMap<u32, NativeAdapterInfo>,
    /// Shader `compilation-hints` Record leftover (Dawn C has no slot).
    shader_hints: HashMap<u32, NativeShaderHints>,
    /// WIT `record-gpu-pipeline-constant-value` maps keyed by resource `rep`.
    pipeline_constant_records: HashMap<u32, Vec<(String, f64)>>,
    /// Constants copied onto compute/render pipelines at create (Dawn slot still 0).
    pipeline_constants: HashMap<u32, NativePipelineConstants>,
}

impl Default for NativeGpuHost {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeGpuHost {
    pub fn new() -> Self {
        Self {
            table: HandleTable::new(),
            interned_queues: HashMap::new(),
            adapter_info: HashMap::new(),
            shader_hints: HashMap::new(),
            pipeline_constant_records: HashMap::new(),
            pipeline_constants: HashMap::new(),
        }
    }

    fn forget_side(&mut self, handle: GpuHandle, kind: ResourceKind) {
        match kind {
            ResourceKind::Adapter => {
                self.adapter_info.remove(&handle.raw());
            }
            ResourceKind::Device => {
                self.interned_queues.remove(&handle.raw());
            }
            ResourceKind::ShaderModule => {
                self.shader_hints.remove(&handle.raw());
            }
            ResourceKind::ComputePipeline | ResourceKind::RenderPipeline => {
                self.pipeline_constants.remove(&handle.raw());
            }
            _ => {}
        }
    }

    /// `rep == 0` is the fixture `get-adapter` stub; otherwise a live table id.
    pub fn resolve_adapter(&mut self, adapter_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if adapter_rep == GpuHandle::NULL {
            self.request_adapter(&NativeRequestAdapterOptions::default())
                .ok_or(NativeGpuError::AdapterUnavailable)
        } else {
            let handle = GpuHandle::from_raw(adapter_rep)?;
            self.get(handle, ResourceKind::Adapter)?;
            Ok(handle)
        }
    }

    /// `rep == 0` is the fixture `get-device` stub; otherwise a live table id.
    pub fn resolve_device(&mut self, device_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if device_rep == GpuHandle::NULL {
            let adapter = self.resolve_adapter(GpuHandle::NULL)?;
            self.request_device(adapter, &NativeRequestDeviceDescriptor::default())
        } else {
            let handle = GpuHandle::from_raw(device_rep)?;
            self.get(handle, ResourceKind::Device)?;
            Ok(handle)
        }
    }

    pub fn request_adapter(
        &mut self,
        options: &NativeRequestAdapterOptions<'_>,
    ) -> Option<GpuHandle> {
        let mut info = NativeAdapterInfo::default();
        info.is_fallback_adapter = options.force_fallback_adapter;
        let _ = options.feature_level;
        let _ = options.power_preference;
        let _ = options.xr_compatible;
        let handle = self.table.insert(ResourceKind::Adapter, 0);
        self.adapter_info.insert(handle.raw(), info);
        Some(handle)
    }

    pub fn request_device(
        &mut self,
        adapter: GpuHandle,
        desc: &NativeRequestDeviceDescriptor<'_>,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(adapter, ResourceKind::Adapter)?;
        let _ = desc.required_features;
        let _ = desc.required_limits_rep;
        let _ = desc.label;
        let _ = desc.default_queue_label;
        Ok(self.table.insert(ResourceKind::Device, 0))
    }

    pub fn device_queue(&mut self, device: GpuHandle) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        if let Some(&raw) = self.interned_queues.get(&device.raw()) {
            if let Ok(existing) = GpuHandle::from_raw(raw) {
                if self.table.contains(existing) {
                    let _ = self.get(existing, ResourceKind::Queue)?;
                    return Ok(existing);
                }
            }
        }
        let queue = self.table.insert(ResourceKind::Queue, 0);
        self.interned_queues.insert(device.raw(), queue.raw());
        Ok(queue)
    }

    pub fn adapter_info(&self, adapter: GpuHandle) -> Result<NativeAdapterInfo, NativeGpuError> {
        self.get(adapter, ResourceKind::Adapter)?;
        Ok(self
            .adapter_info
            .get(&adapter.raw())
            .cloned()
            .unwrap_or_default())
    }

    pub fn adapter_has_feature(
        &self,
        adapter: GpuHandle,
        name: &str,
    ) -> Result<bool, NativeGpuError> {
        self.get(adapter, ResourceKind::Adapter)?;
        let _ = name;
        // Table-backed: no Dawn feature bits until a consume lane dlopens.
        Ok(false)
    }

    pub fn resolve_texture(&mut self, texture_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if texture_rep == GpuHandle::NULL {
            let device = self.resolve_device(GpuHandle::NULL)?;
            self.create_texture(device, 1, 1, 1, 0, 0, 1, 1, 2, &[], "")
        } else {
            let handle = GpuHandle::from_raw(texture_rep)?;
            self.get(handle, ResourceKind::Texture)?;
            Ok(handle)
        }
    }

    pub fn create_buffer(
        &mut self,
        device: GpuHandle,
        size: u64,
        usage: u32,
        mapped_at_creation: i32,
        label: &str,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let _ = (size, usage, mapped_at_creation, label);
        Ok(self.table.insert(ResourceKind::Buffer, 0))
    }

    pub fn create_texture(
        &mut self,
        device: GpuHandle,
        width: u32,
        height: u32,
        depth: u32,
        format: u32,
        usage: u32,
        mip: u32,
        sample: u32,
        dimension: u32,
        view_formats: &[i32],
        label: &str,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let _ = (
            width,
            height,
            depth,
            format,
            usage,
            mip,
            sample,
            dimension,
            view_formats,
            label,
        );
        Ok(self.table.insert(ResourceKind::Texture, 0))
    }

    pub fn create_sampler(
        &mut self,
        device: GpuHandle,
        mag_filter: u32,
        min_filter: u32,
        address_mode_u: u32,
        address_mode_v: u32,
        address_mode_w: u32,
        mipmap_filter: u32,
        compare: u32,
        has_lod_min: i32,
        lod_min: f32,
        has_lod_max: i32,
        lod_max: f32,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let _ = (
            mag_filter,
            min_filter,
            address_mode_u,
            address_mode_v,
            address_mode_w,
            mipmap_filter,
            compare,
            has_lod_min,
            lod_min,
            has_lod_max,
            lod_max,
        );
        Ok(self.table.insert(ResourceKind::Sampler, 0))
    }

    pub fn create_shader_module(
        &mut self,
        device: GpuHandle,
        code: &str,
        label: &str,
        hint_layouts: &[i32],
        hint_entries: &str,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let _ = (code, label);
        let handle = self.table.insert(ResourceKind::ShaderModule, 0);
        if !hint_layouts.is_empty() || !hint_entries.is_empty() {
            self.shader_hints.insert(
                handle.raw(),
                NativeShaderHints {
                    entries: hint_entries.to_string(),
                    layouts: hint_layouts.to_vec(),
                },
            );
        }
        Ok(handle)
    }

    pub fn shader_compilation_hints(
        &self,
        shader: GpuHandle,
    ) -> Result<Option<&NativeShaderHints>, NativeGpuError> {
        self.get(shader, ResourceKind::ShaderModule)?;
        Ok(self.shader_hints.get(&shader.raw()))
    }

    pub fn create_texture_view(
        &mut self,
        texture: GpuHandle,
        dimension: u32,
        aspect: u32,
        format: u32,
        base_mip: i32,
        mip_count: i32,
        base_layer: i32,
        layer_count: i32,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(texture, ResourceKind::Texture)?;
        let _ = (
            dimension,
            aspect,
            format,
            base_mip,
            mip_count,
            base_layer,
            layer_count,
        );
        Ok(self.table.insert(ResourceKind::TextureView, 0))
    }

    pub fn resolve_shader(&mut self, shader_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if shader_rep == GpuHandle::NULL {
            let device = self.resolve_device(GpuHandle::NULL)?;
            self.create_shader_module(device, "", "", &[], "")
        } else {
            let handle = GpuHandle::from_raw(shader_rep)?;
            self.get(handle, ResourceKind::ShaderModule)?;
            Ok(handle)
        }
    }

    pub fn resolve_bind_group_layout(
        &mut self,
        layout_rep: u32,
    ) -> Result<GpuHandle, NativeGpuError> {
        if layout_rep == GpuHandle::NULL {
            let device = self.resolve_device(GpuHandle::NULL)?;
            self.create_bind_group_layout(device, &[], &[], &[], &[], &[])
        } else {
            let handle = GpuHandle::from_raw(layout_rep)?;
            self.get(handle, ResourceKind::BindGroupLayout)?;
            Ok(handle)
        }
    }

    pub fn resolve_buffer(&mut self, buffer_rep: u32) -> Result<GpuHandle, NativeGpuError> {
        if buffer_rep == GpuHandle::NULL {
            let device = self.resolve_device(GpuHandle::NULL)?;
            self.create_buffer(device, 0, 0, -1, "")
        } else {
            let handle = GpuHandle::from_raw(buffer_rep)?;
            self.get(handle, ResourceKind::Buffer)?;
            Ok(handle)
        }
    }

    pub fn create_bind_group_layout(
        &mut self,
        device: GpuHandle,
        bindings: &[i32],
        visibilities: &[i32],
        buffer_types: &[i32],
        sampler_types: &[i32],
        texture_sample_types: &[i32],
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let _ = (
            bindings,
            visibilities,
            buffer_types,
            sampler_types,
            texture_sample_types,
        );
        Ok(self.table.insert(ResourceKind::BindGroupLayout, 0))
    }

    pub fn create_pipeline_layout(
        &mut self,
        device: GpuHandle,
        bind_group_layouts: &[i32],
        label: &str,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        for &raw in bind_group_layouts {
            if raw > 0 {
                let h = GpuHandle::from_raw(raw as u32)?;
                self.get(h, ResourceKind::BindGroupLayout)?;
            }
        }
        let _ = label;
        Ok(self.table.insert(ResourceKind::PipelineLayout, 0))
    }

    pub fn create_bind_group(
        &mut self,
        device: GpuHandle,
        layout: GpuHandle,
        label: &str,
        bindings: &[i32],
        kinds: &[i32],
        handles: &[i32],
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        self.get(layout, ResourceKind::BindGroupLayout)?;
        let _ = (label, bindings, kinds, handles);
        Ok(self.table.insert(ResourceKind::BindGroup, 0))
    }

    fn copy_constant_record(&self, rep: i32) -> Vec<(String, f64)> {
        if rep <= 0 {
            return Vec::new();
        }
        self.pipeline_constant_records
            .get(&(rep as u32))
            .cloned()
            .unwrap_or_default()
    }

    pub fn pipeline_constant_add(&mut self, handle: u32, key: String, value: f64) {
        let rec = self.pipeline_constant_records.entry(handle).or_default();
        if let Some(slot) = rec.iter_mut().find(|(k, _)| *k == key) {
            slot.1 = value;
        } else {
            rec.push((key, value));
        }
    }

    pub fn pipeline_constant_get(&self, handle: u32, key: &str) -> Option<f64> {
        self.pipeline_constant_records
            .get(&handle)
            .and_then(|rec| rec.iter().find(|(k, _)| k == key).map(|(_, v)| *v))
    }

    pub fn pipeline_constant_has(&self, handle: u32, key: &str) -> bool {
        self.pipeline_constant_get(handle, key).is_some()
    }

    pub fn pipeline_constant_remove(&mut self, handle: u32, key: &str) {
        if let Some(rec) = self.pipeline_constant_records.get_mut(&handle) {
            rec.retain(|(k, _)| k != key);
        }
    }

    pub fn pipeline_constant_keys(&self, handle: u32) -> Vec<String> {
        self.pipeline_constant_records
            .get(&handle)
            .map(|rec| rec.iter().map(|(k, _)| k.clone()).collect())
            .unwrap_or_default()
    }

    pub fn pipeline_constant_values(&self, handle: u32) -> Vec<f64> {
        self.pipeline_constant_records
            .get(&handle)
            .map(|rec| rec.iter().map(|(_, v)| *v).collect())
            .unwrap_or_default()
    }

    pub fn pipeline_constant_entries(&self, handle: u32) -> Vec<(String, f64)> {
        self.pipeline_constant_records
            .get(&handle)
            .cloned()
            .unwrap_or_default()
    }

    pub fn create_compute_pipeline(
        &mut self,
        device: GpuHandle,
        shader_rep: u32,
        entry_point: &str,
        layout_rep: i32,
        label: &str,
        constants_rep: i32,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let _ = self.resolve_shader(shader_rep)?;
        if layout_rep > 0 {
            let h = GpuHandle::from_raw(layout_rep as u32)?;
            self.get(h, ResourceKind::PipelineLayout)?;
        }
        let _ = (entry_point, label);
        let handle = self.table.insert(ResourceKind::ComputePipeline, 0);
        self.pipeline_constants.insert(
            handle.raw(),
            NativePipelineConstants {
                compute: self.copy_constant_record(constants_rep),
                ..NativePipelineConstants::default()
            },
        );
        Ok(handle)
    }

    pub fn create_render_pipeline(
        &mut self,
        device: GpuHandle,
        vertex_shader: u32,
        vertex_entry: &str,
        fragment_shader: i32,
        fragment_entry: &str,
        format: i32,
        layout_rep: i32,
        label: &str,
        vertex_constants: i32,
        fragment_constants: i32,
    ) -> Result<GpuHandle, NativeGpuError> {
        self.get(device, ResourceKind::Device)?;
        let _ = self.resolve_shader(vertex_shader)?;
        if fragment_shader > 0 {
            let _ = self.resolve_shader(fragment_shader as u32)?;
        }
        if layout_rep > 0 {
            let h = GpuHandle::from_raw(layout_rep as u32)?;
            self.get(h, ResourceKind::PipelineLayout)?;
        }
        let _ = (vertex_entry, fragment_entry, format, label);
        let handle = self.table.insert(ResourceKind::RenderPipeline, 0);
        self.pipeline_constants.insert(
            handle.raw(),
            NativePipelineConstants {
                vertex: self.copy_constant_record(vertex_constants),
                fragment: self.copy_constant_record(fragment_constants),
                ..NativePipelineConstants::default()
            },
        );
        Ok(handle)
    }

    pub fn pipeline_constants(
        &self,
        pipeline: GpuHandle,
    ) -> Result<Option<&NativePipelineConstants>, NativeGpuError> {
        let entry = self
            .table
            .get(pipeline, ResourceKind::ComputePipeline)
            .or_else(|_| self.table.get(pipeline, ResourceKind::RenderPipeline))?;
        let _ = entry;
        Ok(self.pipeline_constants.get(&pipeline.raw()))
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
        let entry = self.table.drop_handle(handle)?;
        self.forget_side(handle, entry.kind);
        Ok(entry)
    }

    fn try_drop(&mut self, handle: GpuHandle) -> Option<HandleEntry> {
        let entry = self.table.try_drop(handle)?;
        self.forget_side(handle, entry.kind);
        Some(entry)
    }

    fn handles_of_kind(&self, kind: ResourceKind) -> Vec<GpuHandle> {
        self.table.handles_of_kind(kind)
    }

    fn size(&self) -> usize {
        self.table.size()
    }

    fn clear(&mut self) {
        self.table.clear();
        self.interned_queues.clear();
        self.adapter_info.clear();
        self.shader_hints.clear();
        self.pipeline_constant_records.clear();
        self.pipeline_constants.clear();
    }

    fn request_adapter(&mut self, options: &NativeRequestAdapterOptions<'_>) -> Option<GpuHandle> {
        NativeGpuHost::request_adapter(self, options)
    }

    fn request_device(
        &mut self,
        adapter: GpuHandle,
        desc: &NativeRequestDeviceDescriptor<'_>,
    ) -> Result<GpuHandle, NativeGpuError> {
        NativeGpuHost::request_device(self, adapter, desc)
    }

    fn device_queue(&mut self, device: GpuHandle) -> Result<GpuHandle, NativeGpuError> {
        NativeGpuHost::device_queue(self, device)
    }

    fn adapter_info(&self, adapter: GpuHandle) -> Result<NativeAdapterInfo, NativeGpuError> {
        NativeGpuHost::adapter_info(self, adapter)
    }

    fn adapter_has_feature(&self, adapter: GpuHandle, name: &str) -> Result<bool, NativeGpuError> {
        NativeGpuHost::adapter_has_feature(self, adapter, name)
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

    #[test]
    fn request_adapter_device_queue_boot_no_jni() {
        let mut gpu = NativeGpuHost::new();
        let adapter = gpu
            .request_adapter(&NativeRequestAdapterOptions {
                xr_compatible: Some(true),
                ..Default::default()
            })
            .expect("table-backed adapter");
        assert_ne!(adapter.raw(), GpuHandle::NULL);
        let device = gpu
            .request_device(
                adapter,
                &NativeRequestDeviceDescriptor {
                    label: "l2",
                    default_queue_label: "l2",
                    required_features: &[0, 1],
                    required_limits_rep: 0,
                },
            )
            .expect("table-backed device");
        let q1 = gpu.device_queue(device).expect("queue");
        let q2 = gpu.device_queue(device).expect("interned queue");
        assert_eq!(q1, q2, "device.queue interned like DawnWasiWebGpuHost");
        let info = gpu.adapter_info(adapter).expect("info");
        assert_eq!(info.device, "native-gpu");
        assert!(!info.is_fallback_adapter);
        assert!(!gpu.adapter_has_feature(adapter, "timestamp-query").unwrap());
        let via_zero = gpu.resolve_device(0).expect("fixture get-device");
        assert_ne!(via_zero.raw(), GpuHandle::NULL);
    }

    #[test]
    fn handle_table_default_skips_null() {
        let mut table = HandleTable::default();
        let h = table.insert(ResourceKind::Queue, 0);
        assert_ne!(h.raw(), GpuHandle::NULL);
    }

    #[test]
    fn create_resources_and_shader_hints_record_no_jni() {
        let mut gpu = NativeGpuHost::new();
        let device = gpu.resolve_device(0).expect("boot device");
        let buf = gpu.create_buffer(device, 4, 0x28, 1, "l2").expect("buffer");
        let tex = gpu
            .create_texture(device, 1, 1, 1, 0, 0, 2, 1, 2, &[1], "l2")
            .expect("texture");
        let samp = gpu
            .create_sampler(device, 0, 0, 0, 0, 0, 0, 0, 0, 0.0, 0, 0.0)
            .expect("sampler");
        let shader = gpu
            .create_shader_module(device, "fn l2() {}", "l2", &[-1], "l2")
            .expect("shader");
        let view = gpu
            .create_texture_view(tex, 0, 0, 0, 0, 1, 0, 1)
            .expect("view");
        assert_ne!(buf.raw(), GpuHandle::NULL);
        assert_ne!(samp.raw(), GpuHandle::NULL);
        assert_ne!(view.raw(), GpuHandle::NULL);
        let entry = gpu.get(shader, ResourceKind::ShaderModule).unwrap();
        assert_eq!(entry.dawn, 0, "compilation-hints stay Record, not Dawn C");
        let hints = gpu
            .shader_compilation_hints(shader)
            .unwrap()
            .expect("hints recorded");
        assert_eq!(hints.entries, "l2");
        assert_eq!(hints.layouts, vec![-1]);
    }

    #[test]
    fn create_layouts_pipelines_and_constants_no_jni() {
        let mut gpu = NativeGpuHost::new();
        let device = gpu.resolve_device(0).expect("boot device");
        let bgl = gpu
            .create_bind_group_layout(device, &[0], &[4], &[0], &[-1], &[-1])
            .expect("bgl");
        let pl = gpu
            .create_pipeline_layout(device, &[bgl.raw() as i32], "l2")
            .expect("pipeline-layout");
        let bg = gpu
            .create_bind_group(device, bgl, "l2", &[0], &[0], &[0])
            .expect("bind-group");
        gpu.pipeline_constant_add(7, "c".into(), 1.0);
        assert!(gpu.pipeline_constant_has(7, "c"));
        assert_eq!(gpu.pipeline_constant_get(7, "c"), Some(1.0));
        let shader = gpu
            .create_shader_module(device, "fn main() {}", "", &[], "")
            .expect("shader");
        let compute = gpu
            .create_compute_pipeline(device, shader.raw(), "main", 0, "l2", 7)
            .expect("compute");
        let render = gpu
            .create_render_pipeline(device, shader.raw(), "vs_main", 0, "", 0, 0, "l2", 0, 0)
            .expect("render");
        assert_ne!(pl.raw(), GpuHandle::NULL);
        assert_ne!(bg.raw(), GpuHandle::NULL);
        let constants = gpu
            .pipeline_constants(compute)
            .unwrap()
            .expect("constants copied");
        assert_eq!(constants.compute, vec![("c".into(), 1.0)]);
        assert_eq!(
            gpu.get(render, ResourceKind::RenderPipeline).unwrap().dawn,
            0
        );
    }
}
