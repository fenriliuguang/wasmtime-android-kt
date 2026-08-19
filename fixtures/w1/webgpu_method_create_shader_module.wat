;; L2: wasi:webgpu/webgpu@0.3.0-rc.2 get-device +
;; [method]gpu-device.create-shader-module
;; WIT: create-shader-module: func(descriptor: gpu-shader-module-descriptor)
;;      -> gpu-shader-module
;; Guest passes code="@compute @workgroup_size(1) fn l2() {}", hints/label none;
;; drops own; run returns harness 1.
;; get-device is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-pipeline-layout" (type $gpu-pipeline-layout (sub resource)))
    (type $borrow-pll (borrow $gpu-pipeline-layout))
    (type $layout-mode (variant
      (case "specific" $borrow-pll)
      (case "auto")
    ))
    (export "gpu-layout-mode" (type (eq $layout-mode)))
    (type $opt-layout (option 3))
    (type $hint (record
      (field "entry-point" string)
      (field "layout" $opt-layout)
    ))
    (export "gpu-shader-module-compilation-hint" (type (eq $hint)))
    (type $list-hint (list 6))
    (type $opt-list-hint (option $list-hint))
    (type $opt-str (option string))
    (type $desc-def (record
      (field "code" string)
      (field "compilation-hints" $opt-list-hint)
      (field "label" $opt-str)
    ))
    (export "gpu-shader-module-descriptor" (type (eq $desc-def)))
    (export "gpu-shader-module" (type $gpu-shader-module (sub resource)))
    (export "gpu-device" (type $gpu-device (sub resource)))
    (type $borrow-device (borrow $gpu-device))
    (type $own-shader (own $gpu-shader-module))
    (type $create-ty (func
      (param "self" $borrow-device)
      (param "descriptor" 11)
      (result $own-shader)))
    (export "[method]gpu-device.create-shader-module" (func (type $create-ty)))
    (type $own-device (own $gpu-device))
    (export "get-device" (func (result $own-device)))
  ))
  (alias export $webgpu "gpu-shader-module" (type $gpu-shader-module))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.create-shader-module" (func $create-shader))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $cs_lower
    (canon lower (func $create-shader)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $ds_lower (canon resource.drop $gpu-shader-module))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "create-shader"
      (func $create-shader
        (param i32 i32 i32 i32 i32 i32 i32 i32 i32)
        (result i32)))
    (import "" "drop-shader" (func $drop-shader (param i32)))
    (data (i32.const 64) "@compute @workgroup_size(1) fn l2() {}")
    (func (export "run") (result i32)
      (local $device i32)
      (local $shader i32)
      (local.set $device (call $get-device))
      (local.set $shader
        (call $create-shader
          (local.get $device)
          (i32.const 64)
          (i32.const 38)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)))
      (call $drop-shader (local.get $shader))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-device" (func $gd_lower))
      (export "create-shader" (func $cs_lower))
      (export "drop-shader" (func $ds_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
