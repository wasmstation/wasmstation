# wasmstation

A work-in-progress runtime for [WASM-4](https://github.com/aduros/wasm4).

## Instructions

To test the CLI:
```shell
 $ cargo run --bin wasmstation --features="cli" -- run <WASM4 CART>
```

To install the CLI:
```shell
 $ cargo install --path . --features="cli" --bin wasmstation
```

To test the [web example](./examples/web-basic) with [trunk](https://trunkrs.dev/):
```shell
 $ trunk serve examples/web-basic/index.html
```

## Short Term Goals
* [X] Implement all WASM-4 functions
* [X] Run W4 games (carts) on desktop platforms
* [X] Embed wasmstation into standalone game executables
* [X] Offer support for different renderers (wgpu, sdl2)
* [ ] Driver infrastructure for input
* [ ] Factor Abstractions into design for mid and long term goals


## Mid Term Goals
* Run W4 games on mobile and embedded platforms
* Ability to compile games to run standalone without a WASM runtime (?)
* allow input from non-standard sources via driver infrastructure (i2c devices on embeded)
* Multi-app runtime support
* Have 'privileged' menu app (like Xbox guide) that allows downloading apps


## Long Term Goals
* [X] Offer support for runtimes other than wasmer (wasm3, etc...)

