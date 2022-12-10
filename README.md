# wasmstation

A work-in-progress runtime for [wasm4](https://github.com/aduros/wasm4).

## Traits
```rust
trait Renderer {
    // with only one method for the renderer, the user of the library
    // would have to manage window management/resizing.
    fn render(&mut self, framebuffer: [u8; 6400], palette: [u32; 4]) -> Result<(), Box<dyn Error>>;
}

trait Backend {
    fn call_update(&mut self);
    fn call_start(&mut self);

    // all data required to draw the state of the game.
    fn get_screen(&self) -> ([u8; 6400], [u32; 4]);

    // it's best that game input is separate if we want this to run on embedded.
    fn set_gamepad_state(gamepad: u32);
}
```

## Project Structure

Here's my idea:

 - `/Cargo.toml` - Repository is a workspace
 - `/wasmstation-backend-*` - Backend implementations (wasmer, [wasm-micro-runtime](https://github.com/bytecodealliance/wasm-micro-runtime)?)
 - `/wasmstation-renderer-*` - Renderer implementations (sdl2, wgpu, [embedded-graphics](https://crates.io/crates/embedded-graphics)?)
 - `/wasmstation` - Main crate which is home to the common traits. Also reexports backends/renderers when features are set.
 - `/cli` - The main way (I imagine) the library will be used.

I also had an idea about a python (?) script that automatically builds an executable from the wasm cart like `w4 bundle`. That'll be much further into the development, though.
