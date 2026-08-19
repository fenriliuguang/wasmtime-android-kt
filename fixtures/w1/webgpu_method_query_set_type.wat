;; S6+: get-query-set + [method]gpu-query-set.type
;; WIT: type: func() -> gpu-query-type. Host returns occlusion; harness 1.
;; L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $qt (enum "occlusion" "timestamp"))
    (export "gpu-query-type" (type $gpu-query-type (eq $qt)))
    (export "gpu-query-set" (type $gpu-query-set (sub resource)))
    (type $borrow-qs (borrow $gpu-query-set))
    (type $ty-fn (func (param "self" $borrow-qs) (result $gpu-query-type)))
    (export "[method]gpu-query-set.type" (func (type $ty-fn)))
    (type $own-qs (own $gpu-query-set))
    (export "get-query-set" (func (result $own-qs)))
  ))
  (alias export $webgpu "get-query-set" (func $get-qs))
  (alias export $webgpu "[method]gpu-query-set.type" (func $ty))

  (core func $gq_lower (canon lower (func $get-qs)))
  (core func $t_lower (canon lower (func $ty)))

  (core module $m
    (import "" "get-qs" (func $get-qs (result i32)))
    (import "" "ty" (func $ty (param i32) (result i32)))
    (func (export "run") (result i32)
      (drop (call $ty (call $get-qs)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-qs" (func $gq_lower))
      (export "ty" (func $t_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
