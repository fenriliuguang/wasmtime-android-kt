;; S6+: wasi:webgpu/webgpu@0.3.0-rc.2 get-buffer +
;; [method]gpu-buffer.get-mapped-range-set-with-copy
;; WIT: get-mapped-range-set-with-copy: func(data: list<u8>, offset: option<gpu-size64>,
;;      size: option<gpu-size64>) -> result<_, get-mapped-range-error>
;; Guest passes empty data, offset/size=none; result ok; run returns harness 1.
;; L2 still host-fixed (no JNI). get-buffer is a test constructor only.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $kind (variant
      (case "operation-error")
      (case "range-error")
      (case "type-error")
    ))
    (export "get-mapped-range-error-kind" (type (eq $kind)))
    (type $err (record (field "kind" 1) (field "message" string)))
    (export "get-mapped-range-error" (type (eq $err)))
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (type $borrow-buf (borrow $gpu-buffer))
    (type $opt-u64 (option u64))
    (type $list-u8 (list u8))
    (type $result-unit (result (error 3)))
    (type $set-ty (func
      (param "self" $borrow-buf)
      (param "data" $list-u8)
      (param "offset" $opt-u64)
      (param "size" $opt-u64)
      (result $result-unit)))
    (export "[method]gpu-buffer.get-mapped-range-set-with-copy" (func (type $set-ty)))
    (type $own-buf (own $gpu-buffer))
    (export "get-buffer" (func (result $own-buf)))
  ))
  (alias export $webgpu "gpu-buffer" (type $gpu-buffer))
  (alias export $webgpu "get-buffer" (func $get-buffer))
  (alias export $webgpu "[method]gpu-buffer.get-mapped-range-set-with-copy" (func $set-range))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gb_lower (canon lower (func $get-buffer)))
  (core func $sr_lower
    (canon lower (func $set-range)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $db_lower (canon resource.drop $gpu-buffer))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-buffer" (func $get-buffer (result i32)))
    (import "" "set-range"
      (func $set-range
        (param i32 i32 i32 i32 i64 i32 i64 i32)))
    (import "" "drop-buffer" (func $drop-buffer (param i32)))
    (func (export "run") (result i32)
      (local $buf i32)
      (local $retptr i32)
      (local.set $retptr (i32.const 32))
      (local.set $buf (call $get-buffer))
      (call $set-range
        (local.get $buf)
        (i32.const 0) (i32.const 0)
        (i32.const 0) (i64.const 0)
        (i32.const 0) (i64.const 0)
        (local.get $retptr))
      (call $drop-buffer (local.get $buf))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-buffer" (func $gb_lower))
      (export "set-range" (func $sr_lower))
      (export "drop-buffer" (func $db_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
