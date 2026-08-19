;; L2: get-queue + get-buffer + [method]gpu-queue.write-buffer-with-copy
;; WIT: write-buffer-with-copy: func(buffer, buffer-offset, data, data-offset, size)
;;      -> result<_, write-buffer-error>
;; Guest passes borrow buffer, offset=0, 4-byte data "l2\00\00", offset/size none;
;; result ok; drops buffer; run returns harness 1.
;; get-queue / get-buffer are test constructors (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (export "gpu-queue" (type $gpu-queue (sub resource)))
    (type $kind (variant (case "operation-error")))
    (export "write-buffer-error-kind" (type (eq $kind)))
    (type $err (record (field "kind" 3) (field "message" string)))
    (export "write-buffer-error" (type (eq $err)))
    (type $borrow-buf (borrow $gpu-buffer))
    (type $list-u8 (list u8))
    (type $opt-u64 (option u64))
    (type $borrow-q (borrow $gpu-queue))
    (type $result-unit (result (error 5)))
    (type $write-ty (func
      (param "self" $borrow-q)
      (param "buffer" $borrow-buf)
      (param "buffer-offset" u64)
      (param "data" $list-u8)
      (param "data-offset" $opt-u64)
      (param "size" $opt-u64)
      (result $result-unit)))
    (export "[method]gpu-queue.write-buffer-with-copy" (func (type $write-ty)))
    (type $own-q (own $gpu-queue))
    (export "get-queue" (func (result $own-q)))
    (type $own-buf (own $gpu-buffer))
    (export "get-buffer" (func (result $own-buf)))
  ))
  (alias export $webgpu "gpu-buffer" (type $gpu-buffer))
  (alias export $webgpu "get-queue" (func $get-queue))
  (alias export $webgpu "get-buffer" (func $get-buffer))
  (alias export $webgpu "[method]gpu-queue.write-buffer-with-copy" (func $write))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gq_lower (canon lower (func $get-queue)))
  (core func $gb_lower (canon lower (func $get-buffer)))
  (core func $w_lower
    (canon lower (func $write)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $db_lower (canon resource.drop $gpu-buffer))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-queue" (func $get-queue (result i32)))
    (import "" "get-buffer" (func $get-buffer (result i32)))
    (import "" "write"
      (func $write
        (param i32 i32 i64 i32 i32 i32 i64 i32 i64 i32)))
    (import "" "drop-buffer" (func $drop-buffer (param i32)))
    (data (i32.const 32) "\6c\32\00\00")
    (func (export "run") (result i32)
      (local $queue i32)
      (local $buf i32)
      (local $retptr i32)
      (local.set $retptr (i32.const 0))
      (local.set $queue (call $get-queue))
      (local.set $buf (call $get-buffer))
      (call $write
        (local.get $queue)
        (local.get $buf)
        (i64.const 0)
        (i32.const 32) (i32.const 4)
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
      (export "get-queue" (func $gq_lower))
      (export "get-buffer" (func $gb_lower))
      (export "write" (func $w_lower))
      (export "drop-buffer" (func $db_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
