;; S6+: get-encoder + get-buffer + [method]gpu-command-encoder.clear-buffer
;; WIT: clear-buffer: func(buffer: borrow<gpu-buffer>, offset: option<u64>, size: option<u64>)
;; Guest passes buffer, offset/size none; drops own; run returns harness 1.
;; L2 still host-fixed 4-byte buffer copy.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (export "gpu-command-encoder" (type $gpu-command-encoder (sub resource)))
    (type $borrow-buf (borrow $gpu-buffer))
    (type $opt-u64 (option u64))
    (type $borrow-enc (borrow $gpu-command-encoder))
    (type $clear-ty (func
      (param "self" $borrow-enc)
      (param "buffer" $borrow-buf)
      (param "offset" $opt-u64)
      (param "size" $opt-u64)))
    (export "[method]gpu-command-encoder.clear-buffer" (func (type $clear-ty)))
    (type $own-enc (own $gpu-command-encoder))
    (export "get-encoder" (func (result $own-enc)))
    (type $own-buf (own $gpu-buffer))
    (export "get-buffer" (func (result $own-buf)))
  ))
  (alias export $webgpu "gpu-buffer" (type $gpu-buffer))
  (alias export $webgpu "get-encoder" (func $get-encoder))
  (alias export $webgpu "get-buffer" (func $get-buffer))
  (alias export $webgpu "[method]gpu-command-encoder.clear-buffer" (func $clear))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $gb_lower (canon lower (func $get-buffer)))
  (core func $c_lower (canon lower (func $clear)))
  (core func $db_lower (canon resource.drop $gpu-buffer))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "get-buffer" (func $get-buffer (result i32)))
    (import "" "clear"
      (func $clear (param i32 i32 i32 i64 i32 i64)))
    (import "" "drop-buffer" (func $drop-buffer (param i32)))
    (func (export "run") (result i32)
      (local $enc i32)
      (local $buf i32)
      (local.set $enc (call $get-encoder))
      (local.set $buf (call $get-buffer))
      (call $clear
        (local.get $enc)
        (local.get $buf)
        (i32.const 0) (i64.const 0)
        (i32.const 0) (i64.const 0))
      (call $drop-buffer (local.get $buf))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-encoder" (func $ge_lower))
      (export "get-buffer" (func $gb_lower))
      (export "clear" (func $c_lower))
      (export "drop-buffer" (func $db_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
