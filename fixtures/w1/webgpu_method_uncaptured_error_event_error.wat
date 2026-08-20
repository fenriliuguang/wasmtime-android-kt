;; S6+: get-uncaptured-error-event + [method]gpu-uncaptured-error-event.error
;; L2 described device handle → own gpu-error with device rep; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-error" (type $gpu-error (sub resource)))
    (export "gpu-uncaptured-error-event" (type $gpu-uncaptured-error-event (sub resource)))
    (type $borrow-event (borrow $gpu-uncaptured-error-event))
    (type $own-err (own $gpu-error))
    (type $ty (func (param "self" $borrow-event) (result $own-err)))
    (export "[method]gpu-uncaptured-error-event.error" (func (type $ty)))
    (type $own-event (own $gpu-uncaptured-error-event))
    (export "get-uncaptured-error-event" (func (result $own-event)))
  ))
  (alias export $webgpu "gpu-error" (type $gpu-error))
  (alias export $webgpu "get-uncaptured-error-event" (func $get-event))
  (alias export $webgpu "[method]gpu-uncaptured-error-event.error" (func $error))

  (core func $ge_lower (canon lower (func $get-event)))
  (core func $err_lower (canon lower (func $error)))
  (core func $de_lower (canon resource.drop $gpu-error))

  (core module $m
    (import "" "get-event" (func $get-event (result i32)))
    (import "" "error" (func $error (param i32) (result i32)))
    (import "" "drop-err" (func $drop-err (param i32)))
    (func (export "run") (result i32)
      (call $drop-err (call $error (call $get-event)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-event" (func $ge_lower))
      (export "error" (func $err_lower))
      (export "drop-err" (func $de_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
