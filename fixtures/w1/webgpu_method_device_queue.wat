;; S1: wasi:webgpu/webgpu@0.3.0-rc.2 get-device + [method]gpu-device.queue
;; WIT: queue: func() -> gpu-queue  ⇒  (borrow gpu-device) -> own gpu-queue
;; get-device is a test constructor only (not product WIT).
;; Guest run drops the own handle and returns 1 (harness u32, not the method shape).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-device" (type $gpu-device (sub resource)))
    (export "gpu-queue" (type $gpu-queue (sub resource)))
    (export "get-device" (func (result (own $gpu-device))))
    (export "[method]gpu-device.queue"
      (func (param "self" (borrow $gpu-device)) (result (own $gpu-queue))))
  ))
  (alias export $webgpu "gpu-queue" (type $gpu-queue))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.queue" (func $queue))

  (core module $m
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "queue" (func $queue (param i32) (result i32)))
    (import "" "drop-queue" (func $drop-queue (param i32)))
    (func (export "run") (result i32)
      (local $device i32)
      (local $queue i32)
      (local.set $device (call $get-device))
      (local.set $queue (call $queue (local.get $device)))
      (call $drop-queue (local.get $queue))
      (i32.const 1)
    )
  )
  (core func $gd_lower (canon lower (func $get-device)))
  (core func $q_lower (canon lower (func $queue)))
  (core func $dq_lower (canon resource.drop $gpu-queue))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-device" (func $gd_lower))
      (export "queue" (func $q_lower))
      (export "drop-queue" (func $dq_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
