;; S6+: get-render-bundle-encoder + get-bind-group +
;; [method]gpu-render-bundle-encoder.set-bind-group
;; WIT: set-bind-group: func(index, option<borrow<gpu-bind-group>>,
;;      option<list<u32>>, option<u64>, option<u32>) -> result<_, set-bind-group-error>
;; Guest passes index=0, bind-group=some, offsets none; result ok; drops own;
;; run returns harness 1. L2 unused.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-bind-group" (type $gpu-bind-group (sub resource)))
    (export "gpu-render-bundle-encoder" (type $gpu-render-bundle-encoder (sub resource)))
    (type $kind (variant (case "range-error")))
    (export "set-bind-group-error-kind" (type (eq $kind)))
    (type $err (record (field "kind" 3) (field "message" string)))
    (export "set-bind-group-error" (type (eq $err)))
    (type $borrow-bg (borrow $gpu-bind-group))
    (type $opt-bg (option $borrow-bg))
    (type $list-u32 (list u32))
    (type $opt-list (option $list-u32))
    (type $opt-u64 (option u64))
    (type $opt-u32 (option u32))
    (type $borrow-encoder (borrow $gpu-render-bundle-encoder))
    (type $result-unit (result (error 5)))
    (type $set-ty (func
      (param "self" $borrow-encoder)
      (param "index" u32)
      (param "bind-group" $opt-bg)
      (param "dynamic-offsets-data" $opt-list)
      (param "dynamic-offsets-data-start" $opt-u64)
      (param "dynamic-offsets-data-length" $opt-u32)
      (result $result-unit)))
    (export "[method]gpu-render-bundle-encoder.set-bind-group" (func (type $set-ty)))
    (type $own-encoder (own $gpu-render-bundle-encoder))
    (export "get-render-bundle-encoder" (func (result $own-encoder)))
    (type $own-bg (own $gpu-bind-group))
    (export "get-bind-group" (func (result $own-bg)))
  ))
  (alias export $webgpu "gpu-bind-group" (type $gpu-bind-group))
  (alias export $webgpu "get-render-bundle-encoder" (func $get-encoder))
  (alias export $webgpu "get-bind-group" (func $get-bind-group))
  (alias export $webgpu "[method]gpu-render-bundle-encoder.set-bind-group" (func $set-bind-group))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $gbg_lower (canon lower (func $get-bind-group)))
  (core func $sbg_lower
    (canon lower (func $set-bind-group)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $dbg_lower (canon resource.drop $gpu-bind-group))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "get-bind-group" (func $get-bind-group (result i32)))
    (import "" "set-bind-group"
      (func $set-bind-group
        (param i32 i32 i32 i32 i32 i32 i32 i32 i64 i32 i32 i32)))
    (import "" "drop-bind-group" (func $drop-bind-group (param i32)))
    (func (export "run") (result i32)
      (local $encoder i32)
      (local $bg i32)
      (local $retptr i32)
      (local.set $retptr (i32.const 0))
      (local.set $encoder (call $get-encoder))
      (local.set $bg (call $get-bind-group))
      (call $set-bind-group
        (local.get $encoder)
        (i32.const 0)
        (i32.const 1) (local.get $bg)
        (i32.const 0) (i32.const 0) (i32.const 0)
        (i32.const 0) (i64.const 0)
        (i32.const 0) (i32.const 0)
        (local.get $retptr))
      (call $drop-bind-group (local.get $bg))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-encoder" (func $ge_lower))
      (export "get-bind-group" (func $gbg_lower))
      (export "set-bind-group" (func $sbg_lower))
      (export "drop-bind-group" (func $dbg_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
