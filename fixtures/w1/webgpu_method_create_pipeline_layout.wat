;; S6+: wasi:webgpu/webgpu@0.3.0-rc.2 get-device +
;; [method]gpu-device.create-pipeline-layout
;; WIT: create-pipeline-layout: func(descriptor: gpu-pipeline-layout-descriptor)
;;      -> gpu-pipeline-layout
;; Guest passes empty bind-group-layouts, immediate-size=none, label=none;
;; drops own; run returns harness 1. L2 still host-fixed empty layouts.
;; get-device is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-bind-group-layout" (type $gpu-bind-group-layout (sub resource)))
    (type $borrow-bgl (borrow $gpu-bind-group-layout))
    (type $opt-bgl (option $borrow-bgl))
    (type $list-opt-bgl (list $opt-bgl))
    (type $opt-u32 (option u32))
    (type $opt-str (option string))
    (type $desc-def (record
      (field "bind-group-layouts" $list-opt-bgl)
      (field "immediate-size" $opt-u32)
      (field "label" $opt-str)
    ))
    (export "gpu-pipeline-layout-descriptor" (type (eq $desc-def)))
    (export "gpu-pipeline-layout" (type $gpu-pipeline-layout (sub resource)))
    (export "gpu-device" (type $gpu-device (sub resource)))
    (type $borrow-device (borrow $gpu-device))
    (type $own-pll (own $gpu-pipeline-layout))
    (type $create-ty (func
      (param "self" $borrow-device)
      (param "descriptor" 7)
      (result $own-pll)))
    (export "[method]gpu-device.create-pipeline-layout" (func (type $create-ty)))
    (type $own-device (own $gpu-device))
    (export "get-device" (func (result $own-device)))
  ))
  (alias export $webgpu "gpu-pipeline-layout" (type $gpu-pipeline-layout))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.create-pipeline-layout" (func $create-pll))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $cpl_lower
    (canon lower (func $create-pll)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $dpl_lower (canon resource.drop $gpu-pipeline-layout))

  (core module $m
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "create-pll"
      (func $create-pll
        (param i32 i32 i32 i32 i32 i32 i32 i32)
        (result i32)))
    (import "" "drop-pll" (func $drop-pll (param i32)))
    (func (export "run") (result i32)
      (local $device i32)
      (local $pll i32)
      (local.set $device (call $get-device))
      (local.set $pll
        (call $create-pll
          (local.get $device)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)))
      (call $drop-pll (local.get $pll))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-device" (func $gd_lower))
      (export "create-pll" (func $cpl_lower))
      (export "drop-pll" (func $dpl_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
