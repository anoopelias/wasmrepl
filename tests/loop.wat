;; Print 0
(module
  (import "console" "log" (func $log (param i32)))
  (func $main
    (local i32)
    (i32.const 10)
    (loop $bname (param i32) (result i32)
        (i32.const 1)
        (i32.sub)
        (local.tee 0)
        (local.get 0)
        (if (param i32) (result i32) (then br $bname))
    )
    call $log
  ) 
  (start $main)
)