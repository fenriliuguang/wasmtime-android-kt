;; W3+: get-queue + [method]gpu-queue.write-texture (self, texture u32; void). Returns 37.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-queue" (type $gpu-queue (sub resource)))
    (export "get-queue" (func (result (own $gpu-queue))))
    (export "[method]gpu-queue.write-texture"
      (func (param "self" (borrow $gpu-queue)) (param "texture" u32)))
  ))
  (alias export $webgpu "get-queue" (func $get-queue))
  (alias export $webgpu "[method]gpu-queue.write-texture" (func $write-texture))

  (core module $m
    (import "" "get-queue" (func $get-queue (result i32)))
    (import "" "write-texture" (func $write-texture (param i32 i32)))
    (func (export "run") (result i32)
      (local $queue i32)
      (local.set $queue (call $get-queue))
      (call $write-texture (local.get $queue) (i32.const 37))
      (i32.const 37)
    )
  )
  (core func $gq_lower (canon lower (func $get-queue)))
  (core func $wt_lower (canon lower (func $write-texture)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-queue" (func $gq_lower))
      (export "write-texture" (func $wt_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
