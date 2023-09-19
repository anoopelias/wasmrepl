
# wasmrepl

Since Web Assembly's text format is close to Lisp, it is well suited for a REPL as well!

Demo:

[![demo](https://asciinema.org/a/608816.svg)](https://asciinema.org/a/608816?autoplay=1)

## Notes
- The goal of this project is _not_ to provide a fully compatible Wasm interpreter, rather help everyone understand and visualize how Wasm in general works.
- Wasm features that are supported as of now are documented [here](./Features.md).

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

It should give you the REPL prompt. To exit the prompt, use Ctrl+D or Ctrl+C
