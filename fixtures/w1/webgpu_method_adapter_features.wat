;; S6+: get-adapter + [method]gpu-adapter.features
;; WIT: features: func() -> gpu-supported-features. Drop own; harness 1.
;; L2 unused (lift-only). get-adapter is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-adapter" (type $gpu-adapter (sub resource)))
    (export "gpu-supported-features" (type $gpu-supported-features (sub resource)))
    (type $borrow-adapter (borrow $gpu-adapter))
    (type $own-feat (own $gpu-supported-features))
    (type $ty (func (param "self" $borrow-adapter) (result $own-feat)))
    (export "[method]gpu-adapter.features" (func (type $ty)))
    (type $own-adapter (own $gpu-adapter))
    (export "get-adapter" (func (result $own-adapter)))
  ))
  (alias export $webgpu "gpu-supported-features" (type $gpu-supported-features))
  (alias export $webgpu "get-adapter" (func $get-adapter))
  (alias export $webgpu "[method]gpu-adapter.features" (func $features))

  (core func $ga_lower (canon lower (func $get-adapter)))
  (core func $f_lower (canon lower (func $features)))
  (core func $df_lower (canon resource.drop $gpu-supported-features))

  (core module $m
    (import "" "get-adapter" (func $get-adapter (result i32)))
    (import "" "features" (func $features (param i32) (result i32)))
    (import "" "drop-feat" (func $drop-feat (param i32)))
    (func (export "run") (result i32)
      (call $drop-feat (call $features (call $get-adapter)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-adapter" (func $ga_lower))
      (export "features" (func $f_lower))
      (export "drop-feat" (func $df_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
