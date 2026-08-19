;; S6+: get-gpu-error + [method]gpu-error.kind
;; WIT: kind: func() -> gpu-error-kind. Host validation-error; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $kind (variant (case "validation-error") (case "out-of-memory-error") (case "internal-error")))
    (export "gpu-error-kind" (type $gpu-error-kind (eq $kind)))
    (export "gpu-error" (type $gpu-error (sub resource)))
    (type $borrow-err (borrow $gpu-error))
    (type $ty (func (param "self" $borrow-err) (result $gpu-error-kind)))
    (export "[method]gpu-error.kind" (func (type $ty)))
    (type $own-err (own $gpu-error))
    (export "get-gpu-error" (func (result $own-err)))
  ))
  (alias export $webgpu "get-gpu-error" (func $get-err))
  (alias export $webgpu "[method]gpu-error.kind" (func $kind-fn))

  (core func $ge_lower (canon lower (func $get-err)))
  (core func $k_lower (canon lower (func $kind-fn)))

  (core module $m
    (import "" "get-err" (func $get-err (result i32)))
    (import "" "kind" (func $kind (param i32) (result i32)))
    (func (export "run") (result i32)
      (drop (call $kind (call $get-err)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-err" (func $ge_lower))
      (export "kind" (func $k_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
