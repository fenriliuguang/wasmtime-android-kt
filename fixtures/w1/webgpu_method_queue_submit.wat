;; W3: get-queue + [method]gpu-queue.submit (self, commands u32; void). Returns 19.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-queue" (type $gpu-queue (sub resource)))
    (export "get-queue" (func (result (own $gpu-queue))))
    (export "[method]gpu-queue.submit"
      (func (param "self" (borrow $gpu-queue)) (param "commands" u32)))
  ))
  (alias export $webgpu "get-queue" (func $get-queue))
  (alias export $webgpu "[method]gpu-queue.submit" (func $submit))

  (core module $m
    (import "" "get-queue" (func $get-queue (result i32)))
    (import "" "submit" (func $submit (param i32 i32)))
    (func (export "run") (result i32)
      (local $queue i32)
      (local.set $queue (call $get-queue))
      (call $submit (local.get $queue) (i32.const 19))
      (i32.const 19)
    )
  )
  (core func $gq_lower (canon lower (func $get-queue)))
  (core func $s_lower (canon lower (func $submit)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-queue" (func $gq_lower))
      (export "submit" (func $s_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
