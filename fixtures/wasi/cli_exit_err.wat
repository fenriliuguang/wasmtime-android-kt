;; L-CMD-EXIT: wasi:cli/exit@0.3.0#exit (err).
;; Guest `exit` does not return; host maps CliExit(err) to the run harness 1.
;; Must not kill the ART process. `exit-with-code` is not this lane.
(component
  (import "wasi:cli/exit@0.3.0" (instance $exit
    (export "exit" (func (param "status" (result))))
  ))
  (alias export $exit "exit" (func $exit))

  (core module $m
    (import "" "exit" (func $exit (param i32)))
    (func (export "run") (result i32)
      (call $exit (i32.const 1))
      unreachable)
  )

  (core func $exit_lower (canon lower (func $exit)))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "exit" (func $exit_lower))
    ))
  ))

  (func (export "run") (result u32)
    (canon lift (core func $i "run")))
  (func $command-run (result (result))
    (canon lift (core func $i "run")))
  (instance $cli-run
    (export "run" (func $command-run)))
  (export "wasi:cli/run@0.3.0" (instance $cli-run))
)
