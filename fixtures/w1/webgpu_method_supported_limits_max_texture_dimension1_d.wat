;; L2: get-supported-limits + [method]gpu-supported-limits.max-texture-dimension1-d
;; WIT: max-texture-dimension1-d: func() -> u32. Host returns 1; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-supported-limits" (type $gpu-supported-limits (sub resource)))
    (type $borrow-limits (borrow $gpu-supported-limits))
    (type $ty (func (param "self" $borrow-limits) (result u32)))
    (export "[method]gpu-supported-limits.max-texture-dimension1-d" (func (type $ty)))
    (type $own-limits (own $gpu-supported-limits))
    (export "get-supported-limits" (func (result $own-limits)))
  ))
  (alias export $webgpu "get-supported-limits" (func $get-limits))
  (alias export $webgpu "[method]gpu-supported-limits.max-texture-dimension1-d" (func $meth))

  (core func $gl_lower (canon lower (func $get-limits)))
  (core func $m_lower (canon lower (func $meth)))

  (core module $m
    (import "" "get-limits" (func $get-limits (result i32)))
    (import "" "meth" (func $meth (param i32) (result i32)))
    (func (export "run") (result i32)
      (drop (call $meth (call $get-limits)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-limits" (func $gl_lower))
      (export "meth" (func $m_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)