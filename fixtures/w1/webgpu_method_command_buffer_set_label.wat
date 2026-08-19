;; S6+: get-command-buffer + [method]gpu-command-buffer.set-label
;; Guest passes empty label; run returns harness 1. L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-command-buffer" (type $gpu-command-buffer (sub resource)))
    (type $borrow-buf (borrow $gpu-command-buffer))
    (type $set-ty (func
      (param "self" $borrow-buf)
      (param "label" string)))
    (export "[method]gpu-command-buffer.set-label" (func (type $set-ty)))
    (type $own-buf (own $gpu-command-buffer))
    (export "get-command-buffer" (func (result $own-buf)))
  ))
  (alias export $webgpu "get-command-buffer" (func $get-buf))
  (alias export $webgpu "[method]gpu-command-buffer.set-label" (func $set-label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gb_lower (canon lower (func $get-buf)))
  (core func $s_lower
    (canon lower (func $set-label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "get-buf" (func $get-buf (result i32)))
    (import "" "set-label" (func $set-label (param i32 i32 i32)))
    (func (export "run") (result i32)
      (call $set-label (call $get-buf) (i32.const 0) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-buf" (func $gb_lower))
      (export "set-label" (func $s_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
