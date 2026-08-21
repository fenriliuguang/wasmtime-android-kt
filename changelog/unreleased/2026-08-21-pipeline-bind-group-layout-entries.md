### Code — L2 create-bind-group-layout all entries (2026-08-21)

- `[method]gpu-device.create-bind-group-layout` forwards **all** guest `entries` (binding / visibility / buffer / sampler / texture options) through described JNI; empty list stays valid
- Fixture `webgpu_method_create_bind_group_layout` asserts two buffer entries (binding 0 uniform, binding 1 storage); sampler/texture/storage-texture absent → none
- JNI arrays: buffer 0=uniform/1=storage/2=read-only-storage, sampler 0=filtering/1=non-filtering/2=comparison, texture sample-type 0–4; `-1` = that option absent
