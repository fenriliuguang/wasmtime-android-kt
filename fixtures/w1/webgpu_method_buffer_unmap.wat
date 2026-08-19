;; S6+: get-buffer + [method]gpu-buffer.unmap
;; WIT: unmap: func() -> result<_, unmap-error>
;; Guest unmaps; result ok; run returns harness 1.
;; L2 described JNI forwards buffer rep (0 → stub create). get-buffer is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $kind (variant (case "abort-error")))
    (export "unmap-error-kind" (type (eq $kind)))
    (type $err (record (field "kind" 1) (field "message" string)))
    (export "unmap-error" (type (eq $err)))
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (type $borrow-buf (borrow $gpu-buffer))
    (type $result-unit (result (error 3)))
    (type $unmap-ty (func
      (param "self" $borrow-buf)
      (result $result-unit)))
    (export "[method]gpu-buffer.unmap" (func (type $unmap-ty)))
    (type $own-buf (own $gpu-buffer))
    (export "get-buffer" (func (result $own-buf)))
  ))
  (alias export $webgpu "get-buffer" (func $get-buffer))
  (alias export $webgpu "[method]gpu-buffer.unmap" (func $unmap))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gb_lower (canon lower (func $get-buffer)))
  (core func $um_lower
    (canon lower (func $unmap)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-buffer" (func $get-buffer (result i32)))
    (import "" "unmap" (func $unmap (param i32 i32)))
    (func (export "run") (result i32)
      (local $buf i32)
      (local $retptr i32)
      (local.set $retptr (i32.const 0))
      (local.set $buf (call $get-buffer))
      (call $unmap (local.get $buf) (local.get $retptr))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-buffer" (func $gb_lower))
      (export "unmap" (func $um_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
