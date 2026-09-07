;; L-CMD-TERM: wasi:cli/terminal-{stdin,stdout,stderr}@0.3.0
;; Android: none is allowed. Not a fake TTY.
;; Guest: all three get-terminal-* must be none; harness 1.
(component
  (import "wasi:cli/terminal-input@0.3.0" (instance $term-in
    (export "terminal-input" (type $ti (sub resource)))
  ))
  (alias export $term-in "terminal-input" (type $ti))
  (import "wasi:cli/terminal-output@0.3.0" (instance $term-out
    (export "terminal-output" (type $to (sub resource)))
  ))
  (alias export $term-out "terminal-output" (type $to))

  (import "wasi:cli/terminal-stdin@0.3.0" (instance $term-stdin
    (export "terminal-input" (type (eq $ti)))
    (type $own-ti (own $ti))
    (export "get-terminal-stdin" (func (result (option $own-ti))))
  ))
  (import "wasi:cli/terminal-stdout@0.3.0" (instance $term-stdout
    (export "terminal-output" (type (eq $to)))
    (type $own-to (own $to))
    (export "get-terminal-stdout" (func (result (option $own-to))))
  ))
  (import "wasi:cli/terminal-stderr@0.3.0" (instance $term-stderr
    (export "terminal-output" (type (eq $to)))
    (type $own-to (own $to))
    (export "get-terminal-stderr" (func (result (option $own-to))))
  ))
  (alias export $term-stdin "get-terminal-stdin" (func $get-stdin))
  (alias export $term-stdout "get-terminal-stdout" (func $get-stdout))
  (alias export $term-stderr "get-terminal-stderr" (func $get-stderr))

  (core module $libc
    (memory (export "mem") 1)
    (global $last (mut i32) (i32.const 256))
    (func (export "realloc")
      (param $oldptr i32) (param $oldlen i32) (param $align i32) (param $newlen i32)
      (result i32)
      (local $ret i32)
      (local.set $ret (global.get $last))
      (global.set $last
        (i32.and
          (i32.add (i32.add (local.get $ret) (local.get $newlen)) (i32.const 7))
          (i32.const -8)))
      (local.get $ret)
    )
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-terminal-stdin" (func $get-stdin (param i32)))
    (import "" "get-terminal-stdout" (func $get-stdout (param i32)))
    (import "" "get-terminal-stderr" (func $get-stderr (param i32)))
    (func (export "run") (result i32)
      ;; option<own<resource>>: disc at retptr+0 (0=none), handle at +4
      (call $get-stdin (i32.const 64))
      (if (i32.ne (i32.load (i32.const 64)) (i32.const 0))
        (then (return (i32.const 0))))
      (call $get-stdout (i32.const 64))
      (if (i32.ne (i32.load (i32.const 64)) (i32.const 0))
        (then (return (i32.const 0))))
      (call $get-stderr (i32.const 64))
      (if (i32.ne (i32.load (i32.const 64)) (i32.const 0))
        (then (return (i32.const 0))))
      (i32.const 1)
    )
  )

  (core func $get_stdin_lower
    (canon lower (func $get-stdin)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $get_stdout_lower
    (canon lower (func $get-stdout)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $get_stderr_lower
    (canon lower (func $get-stderr)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "get-terminal-stdin" (func $get_stdin_lower))
      (export "get-terminal-stdout" (func $get_stdout_lower))
      (export "get-terminal-stderr" (func $get_stderr_lower))
    ))
  ))

  (func (export "run") (result u32)
    (canon lift (core func $i "run")))
)
