;; L2: get-encoder + get-buffer + get-buffer +
;; [method]gpu-command-encoder.copy-buffer-to-buffer
;; WIT: copy-buffer-to-buffer: func(source: borrow<gpu-buffer>,
;;      source-offset: option<u64>, destination: borrow<gpu-buffer>,
;;      destination-offset: option<u64>, size: option<u64>)
;; Guest passes two buffers, offsets some(0), size some(4); drops owns; run returns 1.
;; get-encoder / get-buffer are test constructors (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (export "gpu-command-encoder" (type $gpu-command-encoder (sub resource)))
    (type $borrow-buf (borrow $gpu-buffer))
    (type $opt-u64 (option u64))
    (type $borrow-enc (borrow $gpu-command-encoder))
    (type $copy-ty (func
      (param "self" $borrow-enc)
      (param "source" $borrow-buf)
      (param "source-offset" $opt-u64)
      (param "destination" $borrow-buf)
      (param "destination-offset" $opt-u64)
      (param "size" $opt-u64)))
    (export "[method]gpu-command-encoder.copy-buffer-to-buffer" (func (type $copy-ty)))
    (type $own-enc (own $gpu-command-encoder))
    (export "get-encoder" (func (result $own-enc)))
    (type $own-buf (own $gpu-buffer))
    (export "get-buffer" (func (result $own-buf)))
  ))
  (alias export $webgpu "gpu-buffer" (type $gpu-buffer))
  (alias export $webgpu "get-encoder" (func $get-encoder))
  (alias export $webgpu "get-buffer" (func $get-buffer))
  (alias export $webgpu "[method]gpu-command-encoder.copy-buffer-to-buffer" (func $copy))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $gb_lower (canon lower (func $get-buffer)))
  (core func $c_lower (canon lower (func $copy)))
  (core func $db_lower (canon resource.drop $gpu-buffer))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "get-buffer" (func $get-buffer (result i32)))
    (import "" "copy"
      (func $copy (param i32 i32 i32 i64 i32 i32 i64 i32 i64)))
    (import "" "drop-buffer" (func $drop-buffer (param i32)))
    (func (export "run") (result i32)
      (local $enc i32)
      (local $src i32)
      (local $dst i32)
      (local.set $enc (call $get-encoder))
      (local.set $src (call $get-buffer))
      (local.set $dst (call $get-buffer))
      (call $copy
        (local.get $enc)
        (local.get $src)
        (i32.const 1) (i64.const 0)
        (local.get $dst)
        (i32.const 1) (i64.const 0)
        (i32.const 1) (i64.const 4))
      (call $drop-buffer (local.get $src))
      (call $drop-buffer (local.get $dst))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-encoder" (func $ge_lower))
      (export "get-buffer" (func $gb_lower))
      (export "copy" (func $c_lower))
      (export "drop-buffer" (func $db_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
