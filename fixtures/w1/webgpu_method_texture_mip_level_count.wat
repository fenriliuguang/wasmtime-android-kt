
;; S6+: get-texture + [method]gpu-texture.mip-level-count
;; WIT: mip-level-count: func() -> u32. Host returns 1; harness 1.
;; L2 described texture handle → mip-level-count (stub 1×1 when get-texture rep=0).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-texture" (type $gpu-texture (sub resource)))
    (type $borrow-tex (borrow $gpu-texture))
    (type $ty (func (param "self" $borrow-tex) (result u32)))
    (export "[method]gpu-texture.mip-level-count" (func (type $ty)))
    (type $own-tex (own $gpu-texture))
    (export "get-texture" (func (result $own-tex)))
  ))
  (alias export $webgpu "get-texture" (func $get-tex))
  (alias export $webgpu "[method]gpu-texture.mip-level-count" (func $meth))

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