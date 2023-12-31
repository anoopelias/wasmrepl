# Features

Supported features from [Wasm spec](https://webassembly.github.io/spec/core/syntax/instructions.html) are:
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
        - [x] Tests (`eqz`)
        - [x] Comparisons (`eq`, `ne`..)
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
        - [x] `nop`
        - [ ] `unreachable`
        - [x] `block` .. `end`
        - [x] `loop` .. `end`
        - [x] `if` .. `else` .. `end`
        - [x] `br`
        - [ ] `br_if`
        - [ ] `br_table`
        - [x] `return`
        - [x] `call`
        - [ ] `call_indirect`
- [ ] Modules
    - [ ] types
    - [x] funcs
    - [ ] tables
    - [ ] mems
    - [ ] globals
    - [ ] elems
    - [ ] datas
    - [ ] start
    - [ ] imports
    - [ ] exports
