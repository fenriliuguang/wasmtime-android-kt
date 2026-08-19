;; S6+: get-texture + [method]gpu-texture.destroy
;; WIT: destroy: func(). Guest constructs texture then destroys; harness 1.
;; L2 unused (lift-only). get-texture is a test constructor (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-texture" (type $gpu-texture (sub resource)))
    (type $borrow-tex (borrow $gpu-texture))
    (type $destroy-ty (func (param "self" $borrow-tex)))
    (export "[method]gpu-texture.destroy" (func (type $destroy-ty)))
    (type $own-tex (own $gpu-texture))
    (export "get-texture" (func (result $own-tex)))
  ))
  (alias export $webgpu "get-texture" (func $get-texture))
  (alias export $webgpu "[method]gpu-texture.destroy" (func $destroy))

  (core func $gt_lower (canon lower (func $get-texture)))
  (core func $d_lower (canon lower (func $destroy)))

  (core module $m
    (import "" "get-texture" (func $get-texture (result i32)))
    (import "" "destroy" (func $destroy (param i32)))
    (func (export "run") (result i32)
      (call $destroy (call $get-texture))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-texture" (func $gt_lower))
      (export "destroy" (func $d_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
