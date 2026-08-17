;; S6+: wasi:webgpu/webgpu@0.3.0-rc.2 get-buffer +
;; [method]gpu-buffer.map-async
;; WIT: map-async: async func(mode: gpu-map-mode, offset: option<gpu-size64>,
;;      size: option<gpu-size64>) -> result<_, map-async-error>
;; Guest passes mode=READ, offset/size=none; drops nothing (void ok); run returns
;; harness 1. L2 still host-fixed MAP_READ buffer. True CM async.
;; get-buffer is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $mode (flags "read" "write"))
    (export "gpu-map-mode" (type (eq $mode)))
    (type $opt-u64 (option u64))
    (type $kind (variant
      (case "operation-error")
      (case "range-error")
      (case "abort-error")
    ))
    (export "map-async-error-kind" (type (eq $kind)))
    (type $err (record (field "kind" 4) (field "message" string)))
    (export "map-async-error" (type (eq $err)))
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (type $borrow-buf (borrow $gpu-buffer))
    (type $result-unit (result (error 6)))
    (type $map-ty (func async
      (param "self" $borrow-buf)
      (param "mode" 1)
      (param "offset" $opt-u64)
      (param "size" $opt-u64)
      (result $result-unit)))
    (export "[method]gpu-buffer.map-async" (func (type $map-ty)))
    (type $own-buf (own $gpu-buffer))
    (export "get-buffer" (func (result $own-buf)))
  ))
  (alias export $webgpu "get-buffer" (func $get-buffer))
  (alias export $webgpu "[method]gpu-buffer.map-async" (func $map-async))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gb_lower (canon lower (func $get-buffer)))
  (core func $ma_lower
    (canon lower (func $map-async)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-buffer" (func $get-buffer (result i32)))
    (import "" "map-async"
      (func $map-async
        (param i32 i32 i32 i64 i32 i64 i32)))
    (func (export "run") (result i32)
      (local $buf i32)
      (local $retptr i32)
      (local.set $retptr (i32.const 0))
      (local.set $buf (call $get-buffer))
      (call $map-async
        (local.get $buf)
        (i32.const 1)
        (i32.const 0) (i64.const 0)
        (i32.const 0) (i64.const 0)
        (local.get $retptr))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-buffer" (func $gb_lower))
      (export "map-async" (func $ma_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
