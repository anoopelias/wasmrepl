;; Throws error because br 1 is trying to break out of function
;; Fix by changing to br 0
(module
  (import "console" "log" (func $log (param i32)))
  (func $main
    call $f
    call $log ;; log the result
  )
  (func $f (result i32)
      i32.const 2
      br 1
      drop
      i32.const 3
  )
  (start $main)
)

;; prints 4
;; Branch needs to break from if first
(module
  (import "console" "log" (func $log (param i32)))
  (func $main
    call $fname
    call $log ;; log the result
  )
  (func $fname (result i32)
    
    (block $bname (result i32)
      i32.const 1
      (if $ifname (result i32) (then (i32.const 2) (br 0))
        (else (i32.const 3))
      )
      drop
      i32.const 4
    )

    ;; drop
    ;; i32.const 5
  )
  (start $main)
)

;; Throws error
;; Cannot branch from function explicitly by id
(module
  (import "console" "log" (func $log (param i32)))
  (func $main
    call $fname
    call $log ;; log the result
  )
  (func $fname (result i32)
    
    (block $bname (result i32)
      i32.const 1
      (if $ifname (result i32) (then (i32.const 2) (br $fname))
        (else (i32.const 3))
      )
      drop
      i32.const 4
    )

    drop
    i32.const 5
  )
  (start $main)
)

;; Throws error
;; Cannot branch out from within function, neither by id nor by num
(module
  (import "console" "log" (func $log (param i32)))
  (func $main
    (block $outer (result i32)
      call $fname
    )
    
    call $log ;; log the result
  )
  (func $fname (result i32)
    
    (block $bname (result i32)
      i32.const 1
      (if $ifname (result i32) (then (i32.const 2) (br $outer))
        (else (i32.const 3))
      )
      drop
      i32.const 4
    )

    drop
    i32.const 5
  )
  (start $main)
)

;; print 9
;; branch statement will ignore multiple values on stack and will
;; only return the tip (Similar to return statement)
(module
  (import "console" "log" (func $log (param i32)))
  (func $main
    call $fname
    call $log
  )
  (func $fname (result i32)
    (block $bname (result i32)
      i32.const 1
      (if $ifname (result i32)
        (then
          (i32.const 2)
          (i32.const 5)
          (i32.const 6)
          (i64.const 8)
          (i32.const 9)
          (br 0)
          (i32.const 7))
        (else (i32.const 3))
      )
    )
  )
  (start $main)
)