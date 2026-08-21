### Code — L2 create-bind-group guest entries (2026-08-21)

- `[method]gpu-device.create-bind-group` forwards guest `entries` (`binding` + `gpu-binding-resource`) through described JNI (`bindings`/`kinds`/`handles` arrays); empty list stays valid
- Fixture `webgpu_method_create_bind_group` asserts one `gpu-buffer` entry at binding 0; host builds `BindGroupDescriptor.entries` (not `emptyList()`)
- kind 0=buffer, 1=sampler, 2=texture-view; buffer `rep==0` still stubs via existing create-buffer
