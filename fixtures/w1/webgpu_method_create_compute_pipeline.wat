;; S6+: wasi:webgpu/webgpu@0.3.0-rc.2 get-device + get-shader-module +
;; [method]gpu-device.create-compute-pipeline
;; WIT: create-compute-pipeline: func(descriptor: gpu-compute-pipeline-descriptor)
;;      -> gpu-compute-pipeline
;; Guest passes shader borrow, entry-point="main", constants none, layout=auto, label="l2";
;; drops own pipeline; run returns harness 1. L2 described shader/entry/layout/label.
;; get-device / get-shader-module are test constructors only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-shader-module" (type $gpu-shader-module (sub resource)))
    (export "gpu-pipeline-layout" (type $gpu-pipeline-layout (sub resource)))
    (export "record-gpu-pipeline-constant-value" (type $record-const (sub resource)))
    (type $borrow-pll (borrow $gpu-pipeline-layout))
    (type $layout-mode-def (variant
      (case "specific" $borrow-pll)
      (case "auto")
    ))
    (export "gpu-layout-mode" (type $gpu-layout-mode (eq $layout-mode-def)))
    (type $borrow-shader (borrow $gpu-shader-module))
    (type $opt-str (option string))
    (type $own-const (own $record-const))
    (type $opt-const (option $own-const))
    (type $stage-def (record
      (field "module" $borrow-shader)
      (field "entry-point" $opt-str)
      (field "constants" $opt-const)
    ))
    (export "gpu-programmable-stage" (type $gpu-programmable-stage (eq $stage-def)))
    (type $desc-def (record
      (field "compute" $gpu-programmable-stage)
      (field "layout" $gpu-layout-mode)
      (field "label" $opt-str)
    ))
    (export "gpu-compute-pipeline-descriptor" (type $gpu-compute-pipeline-descriptor (eq $desc-def)))
    (export "gpu-compute-pipeline" (type $gpu-compute-pipeline (sub resource)))
    (export "gpu-device" (type $gpu-device (sub resource)))
    (type $borrow-device (borrow $gpu-device))
    (type $own-pipeline (own $gpu-compute-pipeline))
    (type $create-ty (func
      (param "self" $borrow-device)
      (param "descriptor" $gpu-compute-pipeline-descriptor)
      (result $own-pipeline)))
    (export "[method]gpu-device.create-compute-pipeline" (func (type $create-ty)))
    (type $own-device (own $gpu-device))
    (export "get-device" (func (result $own-device)))
    (type $own-shader (own $gpu-shader-module))
    (export "get-shader-module" (func (result $own-shader)))
  ))
  (alias export $webgpu "gpu-compute-pipeline" (type $gpu-compute-pipeline))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "get-shader-module" (func $get-shader))
  (alias export $webgpu "[method]gpu-device.create-compute-pipeline" (func $create-cp))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $gs_lower (canon lower (func $get-shader)))
  (core func $cc_lower
    (canon lower (func $create-cp)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $dp_lower (canon resource.drop $gpu-compute-pipeline))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "get-shader" (func $get-shader (result i32)))
    (import "" "create-cp"
      (func $create-cp
        (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
        (result i32)))
    (import "" "drop-pipeline" (func $drop-pipeline (param i32)))
    (data (i32.const 32) "main")
    (data (i32.const 48) "l2")
    (func (export "run") (result i32)
      (local $device i32)
      (local $shader i32)
      (local $pipeline i32)
      (local.set $device (call $get-device))
      (local.set $shader (call $get-shader))
      (local.set $pipeline
        (call $create-cp
          (local.get $device)
          (local.get $shader)
          (i32.const 1)
          (i32.const 32)
          (i32.const 4)
          (i32.const 0)
          (i32.const 0)
          (i32.const 1)
          (i32.const 0)
          (i32.const 1)
          (i32.const 48)
          (i32.const 2)))
      (call $drop-pipeline (local.get $pipeline))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-device" (func $gd_lower))
      (export "get-shader" (func $gs_lower))
      (export "create-cp" (func $cc_lower))
      (export "drop-pipeline" (func $dp_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
