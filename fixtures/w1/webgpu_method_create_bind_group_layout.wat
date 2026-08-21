;; P2: wasi:webgpu/webgpu@0.3.0-rc.2 get-device +
;; [method]gpu-device.create-bind-group-layout
;; Guest passes two buffer entries (binding=0 uniform, binding=1 storage,
;; visibility=compute); label=none; drops own; run returns harness 1.
;; get-device is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $buf-ty (enum "uniform" "storage" "read-only-storage"))
    (export "gpu-buffer-binding-type" (type (eq $buf-ty)))
    (type $samp-ty (enum "filtering" "non-filtering" "comparison"))
    (export "gpu-sampler-binding-type" (type (eq $samp-ty)))
    (type $sample-ty (enum "float" "unfilterable-float" "depth" "sint" "uint"))
    (export "gpu-texture-sample-type" (type (eq $sample-ty)))
    (type $st-access (enum "write-only" "read-only" "read-write"))
    (export "gpu-storage-texture-access" (type (eq $st-access)))
    (type $fmt (enum "r8unorm" "r8snorm" "r8uint" "r8sint" "r16unorm" "r16snorm" "r16uint" "r16sint" "r16float" "rg8unorm" "rg8snorm" "rg8uint" "rg8sint" "r32uint" "r32sint" "r32float" "rg16unorm" "rg16snorm" "rg16uint" "rg16sint" "rg16float" "rgba8unorm" "rgba8unorm-srgb" "rgba8snorm" "rgba8uint" "rgba8sint" "bgra8unorm" "bgra8unorm-srgb" "rgb9e5ufloat" "rgb10a2uint" "rgb10a2unorm" "rg11b10ufloat" "rg32uint" "rg32sint" "rg32float" "rgba16unorm" "rgba16snorm" "rgba16uint" "rgba16sint" "rgba16float" "rgba32uint" "rgba32sint" "rgba32float" "stencil8" "depth16unorm" "depth24plus" "depth24plus-stencil8" "depth32float" "depth32float-stencil8" "bc1-rgba-unorm" "bc1-rgba-unorm-srgb" "bc2-rgba-unorm" "bc2-rgba-unorm-srgb" "bc3-rgba-unorm" "bc3-rgba-unorm-srgb" "bc4-r-unorm" "bc4-r-snorm" "bc5-rg-unorm" "bc5-rg-snorm" "bc6h-rgb-ufloat" "bc6h-rgb-float" "bc7-rgba-unorm" "bc7-rgba-unorm-srgb" "etc2-rgb8unorm" "etc2-rgb8unorm-srgb" "etc2-rgb8a1unorm" "etc2-rgb8a1unorm-srgb" "etc2-rgba8unorm" "etc2-rgba8unorm-srgb" "eac-r11unorm" "eac-r11snorm" "eac-rg11unorm" "eac-rg11snorm" "astc4x4-unorm" "astc4x4-unorm-srgb" "astc5x4-unorm" "astc5x4-unorm-srgb" "astc5x5-unorm" "astc5x5-unorm-srgb" "astc6x5-unorm" "astc6x5-unorm-srgb" "astc6x6-unorm" "astc6x6-unorm-srgb" "astc8x5-unorm" "astc8x5-unorm-srgb" "astc8x6-unorm" "astc8x6-unorm-srgb" "astc8x8-unorm" "astc8x8-unorm-srgb" "astc10x5-unorm" "astc10x5-unorm-srgb" "astc10x6-unorm" "astc10x6-unorm-srgb" "astc10x8-unorm" "astc10x8-unorm-srgb" "astc10x10-unorm" "astc10x10-unorm-srgb" "astc12x10-unorm" "astc12x10-unorm-srgb" "astc12x12-unorm" "astc12x12-unorm-srgb"))
    (export "gpu-texture-format" (type (eq $fmt)))
    (type $viewdim (enum "d1" "d2" "d2-array" "cube" "cube-array" "d3"))
    (export "gpu-texture-view-dimension" (type (eq $viewdim)))
    (type $stage (flags "vertex" "fragment" "compute"))
    (export "gpu-shader-stage" (type (eq $stage)))
    (type $opt-buf-ty (option 1))
    (type $opt-bool (option bool))
    (type $opt-u64 (option u64))
    (type $buf-layout (record
      (field "type" $opt-buf-ty)
      (field "has-dynamic-offset" $opt-bool)
      (field "min-binding-size" $opt-u64)
    ))
    (export "gpu-buffer-binding-layout" (type (eq $buf-layout)))
    (type $opt-samp-ty (option 3))
    (type $samp-layout (record
      (field "type" $opt-samp-ty)
    ))
    (export "gpu-sampler-binding-layout" (type (eq $samp-layout)))
    (type $opt-sample-ty (option 5))
    (type $opt-viewdim (option 11))
    (type $tex-layout (record
      (field "sample-type" $opt-sample-ty)
      (field "view-dimension" $opt-viewdim)
      (field "multisampled" $opt-bool)
    ))
    (export "gpu-texture-binding-layout" (type (eq $tex-layout)))
    (type $opt-st-access (option 7))
    (type $st-layout (record
      (field "access" $opt-st-access)
      (field "format" 9)
      (field "view-dimension" $opt-viewdim)
    ))
    (export "gpu-storage-texture-binding-layout" (type (eq $st-layout)))
    (type $opt-buf-layout (option 18))
    (type $opt-samp-layout (option 21))
    (type $opt-tex-layout (option 25))
    (type $opt-st-layout (option 28))
    (type $entry (record
      (field "binding" u32)
      (field "visibility" 13)
      (field "buffer" $opt-buf-layout)
      (field "sampler" $opt-samp-layout)
      (field "texture" $opt-tex-layout)
      (field "storage-texture" $opt-st-layout)
    ))
    (export "gpu-bind-group-layout-entry" (type (eq $entry)))
    (type $list-entry (list 34))
    (type $opt-str (option string))
    (type $desc-def (record
      (field "entries" $list-entry)
      (field "label" $opt-str)
    ))
    (export "gpu-bind-group-layout-descriptor" (type (eq $desc-def)))
    (export "gpu-bind-group-layout" (type $gpu-bind-group-layout (sub resource)))
    (export "gpu-device" (type $gpu-device (sub resource)))
    (type $borrow-device (borrow $gpu-device))
    (type $own-bgl (own $gpu-bind-group-layout))
    (type $create-ty (func
      (param "self" $borrow-device)
      (param "descriptor" 38)
      (result $own-bgl)))
    (export "[method]gpu-device.create-bind-group-layout" (func (type $create-ty)))
    (type $own-device (own $gpu-device))
    (export "get-device" (func (result $own-device)))
  ))
  (alias export $webgpu "gpu-bind-group-layout" (type $gpu-bind-group-layout))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.create-bind-group-layout" (func $create-bgl))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $cbgl_lower
    (canon lower (func $create-bgl)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $dbgl_lower (canon resource.drop $gpu-bind-group-layout))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "create-bgl"
      (func $create-bgl
        (param i32 i32 i32 i32 i32 i32)
        (result i32)))
    (import "" "drop-bgl" (func $drop-bgl (param i32)))
    (func (export "run") (result i32)
      (local $device i32)
      (local $bgl i32)
      (local.set $device (call $get-device))
      ;; Two BGL entries, size 56 each (record+option<u64> aligns to 8).
      ;; Entry 0 at 256: binding=0, visibility=COMPUTE, buffer=uniform.
      (i32.store (i32.const 256) (i32.const 0))
      (i32.store (i32.const 260) (i32.const 4))
      (i32.store (i32.const 264) (i32.const 1))
      (i32.store (i32.const 272) (i32.const 1))
      (i32.store (i32.const 276) (i32.const 0))
      ;; Entry 1 at 312: binding=1, visibility=COMPUTE, buffer=storage.
      ;; type option disc+enum share the i32 at payload start (0x01 | 0x01<<8).
      (i32.store (i32.const 312) (i32.const 1))
      (i32.store (i32.const 316) (i32.const 4))
      (i32.store (i32.const 320) (i32.const 1))
      (i32.store (i32.const 328) (i32.const 0x0101))
      (local.set $bgl
        (call $create-bgl
          (local.get $device)
          (i32.const 256)
          (i32.const 2)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)))
      (call $drop-bgl (local.get $bgl))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-device" (func $gd_lower))
      (export "create-bgl" (func $cbgl_lower))
      (export "drop-bgl" (func $dbgl_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
