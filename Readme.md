
# wasmrepl

Since Web Assembly's text format is close to Lisp, maybe an attempt to a REPL prompt is in order!

Demo:

[![demo](https://asciinema.org/a/qcNk4yG7emT52iSlOBzvLCbE9.svg)](https://asciinema.org/a/qcNk4yG7emT52iSlOBzvLCbE9?autoplay=1)

## Caveats

- This REPL is not strictly built according to Wasm spec. Some rules are relaxed to make it easy to use within a REPL prompt. For example, the prompt acts as the inside of a Wasm `func`, so you can do, say an `(i32.const 12)`. However, unlike the inside of a Wasm `func`, you can also define a new `func` on the prompt.
- We don’t have full coverage of all features of Wasm yet. What is covered is documented [here](./Features.md). If you would like to see a particular feature implemented, please feel free to open an issue. Or a PR.

## Installation

To install, follow the below command,

```
$ cargo install wasmrepl
```

To check if installation is complete,

```
$ wasmrepl
>>
```

It should give you the REPL prompt. To exit the prompt, use Ctrl+D.

## How to use

Some examples on how to use this is added in the blog post [here](https://anoopelias.github.io/posts/intro-to-wasm).
