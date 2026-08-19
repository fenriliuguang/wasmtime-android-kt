
;; S6+: get-texture + [method]gpu-texture.format
;; WIT: format: func() -> gpu-texture-format. Host returns rgba8unorm; harness 1.
;; L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $fmt (enum "r8unorm" "r8snorm" "r8uint" "r8sint" "r16unorm" "r16snorm" "r16uint" "r16sint" "r16float" "rg8unorm" "rg8snorm" "rg8uint" "rg8sint" "r32uint" "r32sint" "r32float" "rg16unorm" "rg16snorm" "rg16uint" "rg16sint" "rg16float" "rgba8unorm" "rgba8unorm-srgb" "rgba8snorm" "rgba8uint" "rgba8sint" "bgra8unorm" "bgra8unorm-srgb" "rgb9e5ufloat" "rgb10a2uint" "rgb10a2unorm" "rg11b10ufloat" "rg32uint" "rg32sint" "rg32float" "rgba16unorm" "rgba16snorm" "rgba16uint" "rgba16sint" "rgba16float" "rgba32uint" "rgba32sint" "rgba32float" "stencil8" "depth16unorm" "depth24plus" "depth24plus-stencil8" "depth32float" "depth32float-stencil8" "bc1-rgba-unorm" "bc1-rgba-unorm-srgb" "bc2-rgba-unorm" "bc2-rgba-unorm-srgb" "bc3-rgba-unorm" "bc3-rgba-unorm-srgb" "bc4-r-unorm" "bc4-r-snorm" "bc5-rg-unorm" "bc5-rg-snorm" "bc6h-rgb-ufloat" "bc6h-rgb-float" "bc7-rgba-unorm" "bc7-rgba-unorm-srgb" "etc2-rgb8unorm" "etc2-rgb8unorm-srgb" "etc2-rgb8a1unorm" "etc2-rgb8a1unorm-srgb" "etc2-rgba8unorm" "etc2-rgba8unorm-srgb" "eac-r11unorm" "eac-r11snorm" "eac-rg11unorm" "eac-rg11snorm" "astc4x4-unorm" "astc4x4-unorm-srgb" "astc5x4-unorm" "astc5x4-unorm-srgb" "astc5x5-unorm" "astc5x5-unorm-srgb" "astc6x5-unorm" "astc6x5-unorm-srgb" "astc6x6-unorm" "astc6x6-unorm-srgb" "astc8x5-unorm" "astc8x5-unorm-srgb" "astc8x6-unorm" "astc8x6-unorm-srgb" "astc8x8-unorm" "astc8x8-unorm-srgb" "astc10x5-unorm" "astc10x5-unorm-srgb" "astc10x6-unorm" "astc10x6-unorm-srgb" "astc10x8-unorm" "astc10x8-unorm-srgb" "astc10x10-unorm" "astc10x10-unorm-srgb" "astc12x10-unorm" "astc12x10-unorm-srgb" "astc12x12-unorm" "astc12x12-unorm-srgb"))
    (export "gpu-texture-format" (type (eq $fmt)))
    (export "gpu-texture" (type $gpu-texture (sub resource)))
    (type $borrow-tex (borrow $gpu-texture))
    (type $ty-fn (func (param "self" $borrow-tex) (result 1)))
    (export "[method]gpu-texture.format" (func (type $ty-fn)))
    (type $own-tex (own $gpu-texture))
    (export "get-texture" (func (result $own-tex)))
  ))
  (alias export $webgpu "get-texture" (func $get-tex))
  (alias export $webgpu "[method]gpu-texture.format" (func $meth))

  (core func $gt_lower (canon lower (func $get-tex)))
  (core func $m_lower (canon lower (func $meth)))

  (core module $m
    (import "" "get-tex" (func $get-tex (result i32)))
    (import "" "meth" (func $meth (param i32) (result i32)))
    (func (export "run") (result i32)
      (drop (call $meth (call $get-tex)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-tex" (func $gt_lower))
      (export "meth" (func $m_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)