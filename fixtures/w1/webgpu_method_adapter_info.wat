;; S6+: get-adapter + [method]gpu-adapter.info
;; WIT: info: func() -> gpu-adapter-info. Drop own; harness 1.
;; L2 unused (lift-only). get-adapter is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-adapter" (type $gpu-adapter (sub resource)))
    (export "gpu-adapter-info" (type $gpu-adapter-info (sub resource)))
    (type $borrow-adapter (borrow $gpu-adapter))
    (type $own-info (own $gpu-adapter-info))
    (type $ty (func (param "self" $borrow-adapter) (result $own-info)))
    (export "[method]gpu-adapter.info" (func (type $ty)))
    (type $own-adapter (own $gpu-adapter))
    (export "get-adapter" (func (result $own-adapter)))
  ))
  (alias export $webgpu "gpu-adapter-info" (type $gpu-adapter-info))
  (alias export $webgpu "get-adapter" (func $get-adapter))
  (alias export $webgpu "[method]gpu-adapter.info" (func $info))

  (core func $ga_lower (canon lower (func $get-adapter)))
  (core func $i_lower (canon lower (func $info)))
  (core func $di_lower (canon resource.drop $gpu-adapter-info))

  (core module $m
    (import "" "get-adapter" (func $get-adapter (result i32)))
    (import "" "info" (func $info (param i32) (result i32)))
    (import "" "drop-info" (func $drop-info (param i32)))
    (func (export "run") (result i32)
      (call $drop-info (call $info (call $get-adapter)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-adapter" (func $ga_lower))
      (export "info" (func $i_lower))
      (export "drop-info" (func $di_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
