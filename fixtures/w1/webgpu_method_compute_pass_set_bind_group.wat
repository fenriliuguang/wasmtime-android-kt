;; S6+: get-compute-pass + get-bind-group +
;; [method]gpu-compute-pass-encoder.set-bind-group
;; WIT: set-bind-group: func(index, option<borrow<gpu-bind-group>>,
;;      option<list<u32>>, option<u64>, option<u32>) -> result<_, set-bind-group-error>
;; Guest passes index=0, bind-group=some, offsets none; result ok; drops own;
;; run returns harness 1. L2 described JNI forwards pass/index/group (offsets none → empty).
;; get-compute-pass / get-bind-group are test constructors (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-bind-group" (type $gpu-bind-group (sub resource)))
    (export "gpu-compute-pass-encoder" (type $gpu-compute-pass-encoder (sub resource)))
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
    (type $borrow-pass (borrow $gpu-compute-pass-encoder))
    (type $result-unit (result (error 5)))
    (type $set-ty (func
      (param "self" $borrow-pass)
      (param "index" u32)
      (param "bind-group" $opt-bg)
      (param "dynamic-offsets-data" $opt-list)
      (param "dynamic-offsets-data-start" $opt-u64)
      (param "dynamic-offsets-data-length" $opt-u32)
      (result $result-unit)))
    (export "[method]gpu-compute-pass-encoder.set-bind-group" (func (type $set-ty)))
    (type $own-pass (own $gpu-compute-pass-encoder))
    (export "get-compute-pass" (func (result $own-pass)))
    (type $own-bg (own $gpu-bind-group))
    (export "get-bind-group" (func (result $own-bg)))
  ))
  (alias export $webgpu "gpu-bind-group" (type $gpu-bind-group))
  (alias export $webgpu "get-compute-pass" (func $get-compute-pass))
  (alias export $webgpu "get-bind-group" (func $get-bind-group))
  (alias export $webgpu "[method]gpu-compute-pass-encoder.set-bind-group" (func $set-bind-group))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gp_lower (canon lower (func $get-compute-pass)))
  (core func $gbg_lower (canon lower (func $get-bind-group)))
  (core func $sbg_lower
    (canon lower (func $set-bind-group)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $dbg_lower (canon resource.drop $gpu-bind-group))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-compute-pass" (func $get-compute-pass (result i32)))
    (import "" "get-bind-group" (func $get-bind-group (result i32)))
    (import "" "set-bind-group"
      (func $set-bind-group
        (param i32 i32 i32 i32 i32 i32 i32 i32 i64 i32 i32 i32)))
    (import "" "drop-bind-group" (func $drop-bind-group (param i32)))
    (func (export "run") (result i32)
      (local $pass i32)
      (local $bg i32)
      (local $retptr i32)
      (local.set $retptr (i32.const 0))
      (local.set $pass (call $get-compute-pass))
      (local.set $bg (call $get-bind-group))
      (call $set-bind-group
        (local.get $pass)
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
      (export "get-compute-pass" (func $gp_lower))
      (export "get-bind-group" (func $gbg_lower))
      (export "set-bind-group" (func $sbg_lower))
      (export "drop-bind-group" (func $dbg_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
