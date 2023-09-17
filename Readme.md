
# wasmrepl

Since Web Assembly's text format is close to Lisp, it is well suited for a REPL as well!

Demo:

[![asciicast](https://asciinema.org/a/608816.png)](https://asciinema.org/a/608816)

## Notes
- The goal of this project is _not_ to provide a fully compatible Wasm interpreter, rather help everyone understand and visualize how Wasm in general works.
- Wasm features that are supported as of now are,
    - [ ] Types
        - [x] Number Types (i32, i64, f32, f64)
        - [ ] Vector Types (v128)
        - [ ] Reference Types
        - [x] Result Types
        - [x] Function Types
        - [ ] Memory Types
        - [ ] Table Types
        - [ ] Global Types
        - [ ] External Types
    - [ ] Instructions
        - [ ] Numeric Instructions
            - [x] Constant ops (`i32.const`, `i64.const` ...)
            - [x] Integer Unary ops (`clz`, `ctz` ...)
            - [x] Integer Binary ops (`add`, `sub` ...)
            - [x] Float Unary ops (`abs`, `neg` ...)
            - [x] Float Binary ops (`add`, `sub` ...)
            - [ ] Tests (`eqz`)
            - [ ] Comparisons (`eq`, `ne`..)
            - [ ] Conversions (`extend8_s`, `extend16_s` ...)
        - [ ] Vector Instructions
        - [ ] Reference Instructions
        - [ ] Parametric Instructions
            - [x] Drop
            - [ ] Select
        - [ ] Variable Instructions
            - [x] `local.set`
            - [x] `local.get`
            - [x] `local.tee`
            - [ ] `global.set`
            - [ ] `global.get`
        - [ ] Table Instructions
        - [ ] Memory Instructions
        - [ ] Control Instructions
            - [ ] `nop`
            - [ ] `unreachable`
            - [ ] `block` .. `end`
            - [ ] `loop` .. `end`
            - [ ] `if` .. `else` .. `end`
            - [ ] `br`
            - [ ] `br_if`
            - [ ] `br_table`
            - [x] `return`
            - [x] `call`
            - [ ] `call_indirect`

