
;; S6+: get-texture + [method]gpu-texture.usage
;; WIT: usage: func() -> gpu-texture-usage. Host usage bits; harness 1.
;; L2 described texture handle → usage (stub 1×1 when get-texture rep=0).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $usage (flags "copy-src" "copy-dst" "texture-binding" "storage-binding" "render-attachment" "transient-attachment"))
    (export "gpu-texture-usage" (type $gpu-texture-usage (eq $usage)))
    (export "gpu-texture" (type $gpu-texture (sub resource)))
    (type $borrow-tex (borrow $gpu-texture))
    (type $usage-fn (func (param "self" $borrow-tex) (result $gpu-texture-usage)))
    (export "[method]gpu-texture.usage" (func (type $usage-fn)))
    (type $own-tex (own $gpu-texture))
    (export "get-texture" (func (result $own-tex)))
  ))
  (alias export $webgpu "get-texture" (func $get-tex))
  (alias export $webgpu "[method]gpu-texture.usage" (func $meth))

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