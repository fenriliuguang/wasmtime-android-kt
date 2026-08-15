;; W3+: get-queue + [method]gpu-queue.write-buffer (self, buffer u32; void). Returns 31.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-queue" (type $gpu-queue (sub resource)))
    (export "get-queue" (func (result (own $gpu-queue))))
    (export "[method]gpu-queue.write-buffer"
      (func (param "self" (borrow $gpu-queue)) (param "buffer" u32)))
  ))
  (alias export $webgpu "get-queue" (func $get-queue))
  (alias export $webgpu "[method]gpu-queue.write-buffer" (func $write-buffer))

  (core module $m
    (import "" "get-queue" (func $get-queue (result i32)))
    (import "" "write-buffer" (func $write-buffer (param i32 i32)))
    (func (export "run") (result i32)
      (local $queue i32)
      (local.set $queue (call $get-queue))
      (call $write-buffer (local.get $queue) (i32.const 31))
      (i32.const 31)
    )
  )
  (core func $gq_lower (canon lower (func $get-queue)))
  (core func $wb_lower (canon lower (func $write-buffer)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-queue" (func $gq_lower))
      (export "write-buffer" (func $wb_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
