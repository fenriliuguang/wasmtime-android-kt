;; S6+: get-query-set + [method]gpu-query-set.destroy
;; WIT: destroy: func(). Guest constructs query-set then destroys; harness 1.
;; L2 described query-set handle → destroy (stub occlusion count 1 when get-query-set rep=0).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-query-set" (type $gpu-query-set (sub resource)))
    (type $borrow-qs (borrow $gpu-query-set))
    (type $destroy-ty (func (param "self" $borrow-qs)))
    (export "[method]gpu-query-set.destroy" (func (type $destroy-ty)))
    (type $own-qs (own $gpu-query-set))
    (export "get-query-set" (func (result $own-qs)))
  ))
  (alias export $webgpu "get-query-set" (func $get-qs))
  (alias export $webgpu "[method]gpu-query-set.destroy" (func $destroy))

  (core func $gq_lower (canon lower (func $get-qs)))
  (core func $d_lower (canon lower (func $destroy)))

  (core module $m
    (import "" "get-qs" (func $get-qs (result i32)))
    (import "" "destroy" (func $destroy (param i32)))
    (func (export "run") (result i32)
      (call $destroy (call $get-qs))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-qs" (func $gq_lower))
      (export "destroy" (func $d_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
