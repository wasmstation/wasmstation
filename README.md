# wasmstation

A work-in-progress runtime for [wasm4](https://github.com/aduros/wasm4).


## Short Term Goals

* Run W4 games (carts) on desktop platforms
* Offer support for different renderers (wgpu, sdl2)
* Driver infrastructure for input
* Factor Abstractions into design for mid and long term goals


## Mid Term Goals

* Run W4 games on mobile and embedded platforms
* Ability to compile games to run standalone without a WASM runtime (?)
* allow input from non-standard sources via driver infrastructure (i2c devices on embeded)
* Multi-app runtime support
* Have 'privileged' menu app (like Xbox guide) that allows downloading apps


## Long Term Goals
* Offer support for runtimes other than wasmer (wasm3, etc...)

