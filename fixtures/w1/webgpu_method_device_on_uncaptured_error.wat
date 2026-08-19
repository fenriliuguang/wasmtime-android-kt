;; S6+: get-device + [method]gpu-device.on-uncaptured-error
;; WIT: on-uncaptured-error: func() -> stream<gpu-error>. Drop readable; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-device" (type $gpu-device (sub resource)))
    (export "gpu-error" (type $gpu-error (sub resource)))
    (type $borrow-dev (borrow $gpu-device))
    (type $own-err (own $gpu-error))
    (type $stream-err (stream $own-err))
    (type $ty (func (param "self" $borrow-dev) (result $stream-err)))
    (export "[method]gpu-device.on-uncaptured-error" (func (type $ty)))
    (type $own-dev (own $gpu-device))
    (export "get-device" (func (result $own-dev)))
  ))
  (alias export $webgpu "gpu-error" (type $gpu-error))
  (type $own-err (own $gpu-error))
  (type $st (stream $own-err))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.on-uncaptured-error" (func $on-error))

  (core module $builtins
    (memory (export "mem") 1)
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $oe_lower (canon lower (func $on-error) (memory $builtins "mem")))
  (core func $stream.drop-readable (canon stream.drop-readable $st))

  (core module $m
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "on-error" (func $on-error (param i32) (result i32)))
    (import "" "stream.drop-readable" (func $stream.drop-readable (param i32)))
    (func (export "run") (result i32)
      (local $dev i32)
      (local $stream i32)
      (local.set $dev (call $get-device))
      (local.set $stream (call $on-error (local.get $dev)))
      (call $stream.drop-readable (local.get $stream))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-device" (func $gd_lower))
      (export "on-error" (func $oe_lower))
      (export "stream.drop-readable" (func $stream.drop-readable))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
