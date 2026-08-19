;; S6+: get-device + [method]gpu-device.create-render-bundle-encoder
;; WIT: create-render-bundle-encoder: func(descriptor) -> gpu-render-bundle-encoder
;; Guest passes empty color-formats, other fields none; drops own; harness 1.
;; L2 unused (lift-only). get-device is a test constructor (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $fmt-def (enum "r8unorm" "r8snorm" "r8uint" "r8sint" "r16unorm" "r16snorm" "r16uint" "r16sint" "r16float" "rg8unorm" "rg8snorm" "rg8uint" "rg8sint" "r32uint" "r32sint" "r32float" "rg16unorm" "rg16snorm" "rg16uint" "rg16sint" "rg16float" "rgba8unorm" "rgba8unorm-srgb" "rgba8snorm" "rgba8uint" "rgba8sint" "bgra8unorm" "bgra8unorm-srgb" "rgb9e5ufloat" "rgb10a2uint" "rgb10a2unorm" "rg11b10ufloat" "rg32uint" "rg32sint" "rg32float" "rgba16unorm" "rgba16snorm" "rgba16uint" "rgba16sint" "rgba16float" "rgba32uint" "rgba32sint" "rgba32float" "stencil8" "depth16unorm" "depth24plus" "depth24plus-stencil8" "depth32float" "depth32float-stencil8" "bc1-rgba-unorm" "bc1-rgba-unorm-srgb" "bc2-rgba-unorm" "bc2-rgba-unorm-srgb" "bc3-rgba-unorm" "bc3-rgba-unorm-srgb" "bc4-r-unorm" "bc4-r-snorm" "bc5-rg-unorm" "bc5-rg-snorm" "bc6h-rgb-ufloat" "bc6h-rgb-float" "bc7-rgba-unorm" "bc7-rgba-unorm-srgb" "etc2-rgb8unorm" "etc2-rgb8unorm-srgb" "etc2-rgb8a1unorm" "etc2-rgb8a1unorm-srgb" "etc2-rgba8unorm" "etc2-rgba8unorm-srgb" "eac-r11unorm" "eac-r11snorm" "eac-rg11unorm" "eac-rg11snorm" "astc4x4-unorm" "astc4x4-unorm-srgb" "astc5x4-unorm" "astc5x4-unorm-srgb" "astc5x5-unorm" "astc5x5-unorm-srgb" "astc6x5-unorm" "astc6x5-unorm-srgb" "astc6x6-unorm" "astc6x6-unorm-srgb" "astc8x5-unorm" "astc8x5-unorm-srgb" "astc8x6-unorm" "astc8x6-unorm-srgb" "astc8x8-unorm" "astc8x8-unorm-srgb" "astc10x5-unorm" "astc10x5-unorm-srgb" "astc10x6-unorm" "astc10x6-unorm-srgb" "astc10x8-unorm" "astc10x8-unorm-srgb" "astc10x10-unorm" "astc10x10-unorm-srgb" "astc12x10-unorm" "astc12x10-unorm-srgb" "astc12x12-unorm" "astc12x12-unorm-srgb"))
    (export "gpu-texture-format" (type $gpu-texture-format (eq $fmt-def)))
    (type $opt-bool (option bool))
    (type $opt-fmt (option $gpu-texture-format))
    (type $list-opt-fmt (list $opt-fmt))
    (type $opt-u32 (option u32))
    (type $opt-str (option string))
    (type $desc-def (record
      (field "depth-read-only" $opt-bool)
      (field "stencil-read-only" $opt-bool)
      (field "color-formats" $list-opt-fmt)
      (field "depth-stencil-format" $opt-fmt)
      (field "sample-count" $opt-u32)
      (field "label" $opt-str)
    ))
    (export "gpu-render-bundle-encoder-descriptor" (type $desc (eq $desc-def)))
    (export "gpu-render-bundle-encoder" (type $gpu-render-bundle-encoder (sub resource)))
    (export "gpu-device" (type $gpu-device (sub resource)))
    (type $borrow-device (borrow $gpu-device))
    (type $own-enc (own $gpu-render-bundle-encoder))
    (type $create-ty (func
      (param "self" $borrow-device)
      (param "descriptor" $desc)
      (result $own-enc)))
    (export "[method]gpu-device.create-render-bundle-encoder" (func (type $create-ty)))
    (type $own-device (own $gpu-device))
    (export "get-device" (func (result $own-device)))
  ))
  (alias export $webgpu "gpu-render-bundle-encoder" (type $gpu-render-bundle-encoder))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.create-render-bundle-encoder" (func $create-rbe))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $c_lower
    (canon lower (func $create-rbe)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $de_lower (canon resource.drop $gpu-render-bundle-encoder))

  (core module $m
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "create-rbe"
      (func $create-rbe
        (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
        (result i32)))
    (import "" "drop-enc" (func $drop-enc (param i32)))
    (func (export "run") (result i32)
      (local $enc i32)
      (local.set $enc
        (call $create-rbe
          (call $get-device)
          (i32.const 0) (i32.const 0)
          (i32.const 0) (i32.const 0)
          (i32.const 0) (i32.const 0)
          (i32.const 0) (i32.const 0)
          (i32.const 0) (i32.const 0)
          (i32.const 0) (i32.const 0) (i32.const 0)))
      (call $drop-enc (local.get $enc))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-device" (func $gd_lower))
      (export "create-rbe" (func $c_lower))
      (export "drop-enc" (func $de_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
