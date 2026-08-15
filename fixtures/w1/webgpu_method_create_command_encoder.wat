;; W3: wasi:webgpu/webgpu@0.3.0-rc.2 get-device +
;; [method]gpu-device.create-command-encoder (resource self; sync).
;; Transitional: method returns u32 (no option<descriptor>). Flat
;; device-create-command-encoder stays registered.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-device" (type $gpu-device (sub resource)))
    (export "get-device" (func (result (own $gpu-device))))
    (export "[method]gpu-device.create-command-encoder"
      (func (param "self" (borrow $gpu-device)) (result u32)))
  ))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.create-command-encoder" (func $create-encoder))

  (core module $m
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "create-encoder" (func $create-encoder (param i32) (result i32)))
    (func (export "run") (result i32)
      (local $device i32)
      (local.set $device (call $get-device))
      (call $create-encoder (local.get $device))
    )
  )
  (core func $gd_lower (canon lower (func $get-device)))
  (core func $ce_lower (canon lower (func $create-encoder)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-device" (func $gd_lower))
      (export "create-encoder" (func $ce_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
