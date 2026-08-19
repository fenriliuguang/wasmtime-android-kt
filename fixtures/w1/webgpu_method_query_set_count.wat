;; S6+: get-query-set + [method]gpu-query-set.count
;; WIT: count: func() -> gpu-size32-out. Host returns 1; harness 1.
;; L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-query-set" (type $gpu-query-set (sub resource)))
    (type $borrow-qs (borrow $gpu-query-set))
    (type $count-ty (func (param "self" $borrow-qs) (result u32)))
    (export "[method]gpu-query-set.count" (func (type $count-ty)))
    (type $own-qs (own $gpu-query-set))
    (export "get-query-set" (func (result $own-qs)))
  ))
  (alias export $webgpu "get-query-set" (func $get-qs))
  (alias export $webgpu "[method]gpu-query-set.count" (func $count))

  (core func $gq_lower (canon lower (func $get-qs)))
  (core func $c_lower (canon lower (func $count)))

  (core module $m
    (import "" "get-qs" (func $get-qs (result i32)))
    (import "" "count" (func $count (param i32) (result i32)))
    (func (export "run") (result i32)
      (drop (call $count (call $get-qs)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-qs" (func $gq_lower))
      (export "count" (func $c_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
