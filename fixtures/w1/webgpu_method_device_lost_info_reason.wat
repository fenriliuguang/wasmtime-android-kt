;; S6+: get-device-lost-info + [method]gpu-device-lost-info.reason
;; WIT: reason: func() -> gpu-device-lost-reason. Host unknown; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $reason (enum "unknown" "destroyed"))
    (export "gpu-device-lost-reason" (type $gpu-device-lost-reason (eq $reason)))
    (export "gpu-device-lost-info" (type $gpu-device-lost-info (sub resource)))
    (type $borrow-info (borrow $gpu-device-lost-info))
    (type $ty (func (param "self" $borrow-info) (result $gpu-device-lost-reason)))
    (export "[method]gpu-device-lost-info.reason" (func (type $ty)))
    (type $own-info (own $gpu-device-lost-info))
    (export "get-device-lost-info" (func (result $own-info)))
  ))
  (alias export $webgpu "get-device-lost-info" (func $get-info))
  (alias export $webgpu "[method]gpu-device-lost-info.reason" (func $reason))

  (core func $gi_lower (canon lower (func $get-info)))
  (core func $r_lower (canon lower (func $reason)))

  (core module $m
    (import "" "get-info" (func $get-info (result i32)))
    (import "" "reason" (func $reason (param i32) (result i32)))
    (func (export "run") (result i32)
      (drop (call $reason (call $get-info)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-info" (func $gi_lower))
      (export "reason" (func $r_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
