;; S6+: [constructor]record-gpu-pipeline-constant-value + [method]record-gpu-pipeline-constant-value.add
;; Guest constructs record then add (empty key, 0.0); harness 1. L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "record-gpu-pipeline-constant-value" (type $rec (sub resource)))
    (type $own-rec (own $rec))
    (export "[constructor]record-gpu-pipeline-constant-value" (func (result $own-rec)))
    (type $borrow-rec (borrow $rec))
    (type $meth-ty (func (param "self" $borrow-rec) (param "key" string) (param "value" f64)))
    (export "[method]record-gpu-pipeline-constant-value.add" (func (type $meth-ty)))
  ))
  (alias export $webgpu "[constructor]record-gpu-pipeline-constant-value" (func $ctor))
  (alias export $webgpu "[method]record-gpu-pipeline-constant-value.add" (func $meth))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $c_lower (canon lower (func $ctor)))
  (core func $m_lower
    (canon lower (func $meth)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "ctor" (func $ctor (result i32)))
    (import "" "meth" (func $meth (param i32 i32 i32 f64)))
    (func (export "run") (result i32)
      (call $meth (call $ctor) (i32.const 0) (i32.const 0) (f64.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "ctor" (func $c_lower))
      (export "meth" (func $m_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)