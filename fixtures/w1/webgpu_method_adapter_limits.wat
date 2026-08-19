;; S6+: get-adapter + [method]gpu-adapter.limits
;; WIT: limits: func() -> gpu-supported-limits. Drop own; harness 1.
;; L2 unused (lift-only). get-adapter is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-adapter" (type $gpu-adapter (sub resource)))
    (export "gpu-supported-limits" (type $gpu-supported-limits (sub resource)))
    (type $borrow-adapter (borrow $gpu-adapter))
    (type $own-lim (own $gpu-supported-limits))
    (type $ty (func (param "self" $borrow-adapter) (result $own-lim)))
    (export "[method]gpu-adapter.limits" (func (type $ty)))
    (type $own-adapter (own $gpu-adapter))
    (export "get-adapter" (func (result $own-adapter)))
  ))
  (alias export $webgpu "gpu-supported-limits" (type $gpu-supported-limits))
  (alias export $webgpu "get-adapter" (func $get-adapter))
  (alias export $webgpu "[method]gpu-adapter.limits" (func $limits))

  (core func $ga_lower (canon lower (func $get-adapter)))
  (core func $l_lower (canon lower (func $limits)))
  (core func $dl_lower (canon resource.drop $gpu-supported-limits))

  (core module $m
    (import "" "get-adapter" (func $get-adapter (result i32)))
    (import "" "limits" (func $limits (param i32) (result i32)))
    (import "" "drop-lim" (func $drop-lim (param i32)))
    (func (export "run") (result i32)
      (call $drop-lim (call $limits (call $get-adapter)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-adapter" (func $ga_lower))
      (export "limits" (func $l_lower))
      (export "drop-lim" (func $dl_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
