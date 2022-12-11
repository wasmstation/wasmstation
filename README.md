# wasmstation

A work-in-progress runtime for [wasm4](https://github.com/aduros/wasm4).

## Project Structure

 - `/Cargo.toml` - Repository is a workspace
 - `/wasmstation` - Main crate which is home to common traits, constants, and functions. Also reexports backends/renderers when features are set.
 - `/wasmstation-backend-*` - Backend implementations (wasmer, [wasm-micro-runtime](https://github.com/bytecodealliance/wasm-micro-runtime)?)
 - `/wasmstation-renderer-*` - Renderer implementations (sdl2, wgpu, [embedded-graphics](https://crates.io/crates/embedded-graphics)?)
 - `/cli` - The main way the library will be used, also a demo implementation.
