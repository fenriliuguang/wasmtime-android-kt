;; S6+: get-buffer + [method]gpu-buffer.destroy
;; WIT: destroy: func(). Guest constructs buffer then destroys; harness 1.
;; L2 described buffer handle → destroy (stub 4-byte buffer when get-buffer rep=0).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (type $borrow-buf (borrow $gpu-buffer))
    (type $destroy-ty (func (param "self" $borrow-buf)))
    (export "[method]gpu-buffer.destroy" (func (type $destroy-ty)))
    (type $own-buf (own $gpu-buffer))
    (export "get-buffer" (func (result $own-buf)))
  ))
  (alias export $webgpu "get-buffer" (func $get-buffer))
  (alias export $webgpu "[method]gpu-buffer.destroy" (func $destroy))

  (core func $gb_lower (canon lower (func $get-buffer)))
  (core func $d_lower (canon lower (func $destroy)))

  (core module $m
    (import "" "get-buffer" (func $get-buffer (result i32)))
    (import "" "destroy" (func $destroy (param i32)))
    (func (export "run") (result i32)
      (call $destroy (call $get-buffer))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-buffer" (func $gb_lower))
      (export "destroy" (func $d_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
