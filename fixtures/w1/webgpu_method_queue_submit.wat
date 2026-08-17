;; S5: wasi:webgpu/webgpu@0.3.0-rc.2 get-queue + get-command-buffer +
;; [method]gpu-queue.submit
;; WIT: submit: func(command-buffers: list<borrow<gpu-command-buffer>>)
;; Guest passes a one-element list; drops owns; run returns harness 1.
;; get-queue / get-command-buffer are test constructors (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-queue" (type $gpu-queue (sub resource)))
    (export "gpu-command-buffer" (type $gpu-command-buffer (sub resource)))
    (type $borrow-queue (borrow $gpu-queue))
    (type $borrow-cb (borrow $gpu-command-buffer))
    (type $list-cb (list $borrow-cb))
    (type $submit-ty (func
      (param "self" $borrow-queue)
      (param "command-buffers" $list-cb)))
    (export "[method]gpu-queue.submit" (func (type $submit-ty)))
    (type $own-queue (own $gpu-queue))
    (export "get-queue" (func (result $own-queue)))
    (type $own-cb (own $gpu-command-buffer))
    (export "get-command-buffer" (func (result $own-cb)))
  ))
  (alias export $webgpu "gpu-queue" (type $gpu-queue))
  (alias export $webgpu "gpu-command-buffer" (type $gpu-command-buffer))
  (alias export $webgpu "get-queue" (func $get-queue))
  (alias export $webgpu "get-command-buffer" (func $get-command-buffer))
  (alias export $webgpu "[method]gpu-queue.submit" (func $submit))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gq_lower (canon lower (func $get-queue)))
  (core func $gcb_lower (canon lower (func $get-command-buffer)))
  (core func $s_lower
    (canon lower (func $submit)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $dq_lower (canon resource.drop $gpu-queue))
  (core func $dcb_lower (canon resource.drop $gpu-command-buffer))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-queue" (func $get-queue (result i32)))
    (import "" "get-command-buffer" (func $get-command-buffer (result i32)))
    (import "" "submit" (func $submit (param i32 i32 i32)))
    (import "" "drop-queue" (func $drop-queue (param i32)))
    (import "" "drop-command-buffer" (func $drop-command-buffer (param i32)))
    (func (export "run") (result i32)
      (local $queue i32)
      (local $cb i32)
      (local.set $queue (call $get-queue))
      (local.set $cb (call $get-command-buffer))
      (i32.store (i32.const 16) (local.get $cb))
      (call $submit (local.get $queue) (i32.const 16) (i32.const 1))
      (call $drop-command-buffer (local.get $cb))
      (call $drop-queue (local.get $queue))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-queue" (func $gq_lower))
      (export "get-command-buffer" (func $gcb_lower))
      (export "submit" (func $s_lower))
      (export "drop-queue" (func $dq_lower))
      (export "drop-command-buffer" (func $dcb_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
