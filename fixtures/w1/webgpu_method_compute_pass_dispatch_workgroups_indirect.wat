;; S6+: get-compute-pass + get-buffer +
;; [method]gpu-compute-pass-encoder.dispatch-workgroups-indirect
;; WIT: dispatch-workgroups-indirect: func(indirect-buffer: borrow, indirect-offset: u64)
;; Guest passes borrow buffer, offset=0; drops buffer; run returns harness 1.
;; L2 still host-fixed 1×1×1 dispatch.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (export "gpu-compute-pass-encoder" (type $gpu-compute-pass-encoder (sub resource)))
    (type $borrow-buf (borrow $gpu-buffer))
    (type $borrow-pass (borrow $gpu-compute-pass-encoder))
    (type $dispatch-ty (func
      (param "self" $borrow-pass)
      (param "indirect-buffer" $borrow-buf)
      (param "indirect-offset" u64)))
    (export "[method]gpu-compute-pass-encoder.dispatch-workgroups-indirect" (func (type $dispatch-ty)))
    (type $own-pass (own $gpu-compute-pass-encoder))
    (export "get-compute-pass" (func (result $own-pass)))
    (type $own-buf (own $gpu-buffer))
    (export "get-buffer" (func (result $own-buf)))
  ))
  (alias export $webgpu "gpu-buffer" (type $gpu-buffer))
  (alias export $webgpu "get-compute-pass" (func $get-compute-pass))
  (alias export $webgpu "get-buffer" (func $get-buffer))
  (alias export $webgpu "[method]gpu-compute-pass-encoder.dispatch-workgroups-indirect" (func $dispatch))

  (core func $gp_lower (canon lower (func $get-compute-pass)))
  (core func $gb_lower (canon lower (func $get-buffer)))
  (core func $d_lower (canon lower (func $dispatch)))
  (core func $db_lower (canon resource.drop $gpu-buffer))

  (core module $m
    (import "" "get-compute-pass" (func $get-compute-pass (result i32)))
    (import "" "get-buffer" (func $get-buffer (result i32)))
    (import "" "dispatch" (func $dispatch (param i32 i32 i64)))
    (import "" "drop-buffer" (func $drop-buffer (param i32)))
    (func (export "run") (result i32)
      (local $pass i32)
      (local $buf i32)
      (local.set $pass (call $get-compute-pass))
      (local.set $buf (call $get-buffer))
      (call $dispatch (local.get $pass) (local.get $buf) (i64.const 0))
      (call $drop-buffer (local.get $buf))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-compute-pass" (func $gp_lower))
      (export "get-buffer" (func $gb_lower))
      (export "dispatch" (func $d_lower))
      (export "drop-buffer" (func $db_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
