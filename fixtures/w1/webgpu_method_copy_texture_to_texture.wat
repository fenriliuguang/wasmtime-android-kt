;; L2: get-encoder + two get-texture +
;; [method]gpu-command-encoder.copy-texture-to-texture
;; Guest passes two texture borrows, mip/origin/aspect none, size 1×1×1;
;; drops owns; run returns harness 1. Flattened params exceed 16 (spill).
;; Native forwards encoder/src/dst texture reps + extent into described JNI.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-texture" (type $gpu-texture (sub resource)))
    (export "gpu-command-encoder" (type $gpu-command-encoder (sub resource)))
    (type $aspect (enum "all" "stencil-only" "depth-only"))
    (export "gpu-texture-aspect" (type (eq $aspect)))
    (type $opt-u32 (option u32))
    (type $origin (record
      (field "x" $opt-u32)
      (field "y" $opt-u32)
      (field "z" $opt-u32)
    ))
    (export "gpu-origin3-d" (type (eq $origin)))
    (type $borrow-tex (borrow $gpu-texture))
    (type $opt-origin (option 6))
    (type $opt-aspect (option 3))
    (type $info (record
      (field "texture" $borrow-tex)
      (field "mip-level" $opt-u32)
      (field "origin" $opt-origin)
      (field "aspect" $opt-aspect)
    ))
    (export "gpu-texel-copy-texture-info" (type (eq $info)))
    (type $extent (record
      (field "width" u32)
      (field "height" $opt-u32)
      (field "depth-or-array-layers" $opt-u32)
    ))
    (export "gpu-extent3-d" (type (eq $extent)))
    (type $borrow-enc (borrow $gpu-command-encoder))
    (type $copy-ty (func
      (param "self" $borrow-enc)
      (param "source" 11)
      (param "destination" 11)
      (param "copy-size" 13)))
    (export "[method]gpu-command-encoder.copy-texture-to-texture" (func (type $copy-ty)))
    (type $own-enc (own $gpu-command-encoder))
    (export "get-encoder" (func (result $own-enc)))
    (type $own-tex (own $gpu-texture))
    (export "get-texture" (func (result $own-tex)))
  ))
  (alias export $webgpu "gpu-texture" (type $gpu-texture))
  (alias export $webgpu "get-encoder" (func $get-encoder))
  (alias export $webgpu "get-texture" (func $get-texture))
  (alias export $webgpu "[method]gpu-command-encoder.copy-texture-to-texture" (func $copy))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 256)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $gt_lower (canon lower (func $get-texture)))
  (core func $c_lower
    (canon lower (func $copy)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $dt_lower (canon resource.drop $gpu-texture))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "get-texture" (func $get-texture (result i32)))
    (import "" "copy" (func $copy (param i32)))
    (import "" "drop-texture" (func $drop-texture (param i32)))
    (func (export "run") (result i32)
      (local $enc i32)
      (local $src i32)
      (local $dst i32)
      (local.set $enc (call $get-encoder))
      (local.set $src (call $get-texture))
      (local.set $dst (call $get-texture))
      ;; Memory tuple: encoder@0, src info@4 (tex@4), dst info@48 (tex@48),
      ;; extent@92
      (i32.store (i32.const 0) (local.get $enc))
      (i32.store (i32.const 4) (local.get $src))
      (i32.store (i32.const 48) (local.get $dst))
      (i32.store (i32.const 92) (i32.const 1))
      (i32.store (i32.const 96) (i32.const 1))
      (i32.store (i32.const 100) (i32.const 1))
      (i32.store (i32.const 104) (i32.const 1))
      (i32.store (i32.const 108) (i32.const 1))
      (call $copy (i32.const 0))
      (call $drop-texture (local.get $src))
      (call $drop-texture (local.get $dst))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-encoder" (func $ge_lower))
      (export "get-texture" (func $gt_lower))
      (export "copy" (func $c_lower))
      (export "drop-texture" (func $dt_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
