# wasmstation

A work-in-progress runtime for [wasm4](https://github.com/aduros/wasm4).

## Traits
```rust
trait Renderer {
    fn render(
        &mut self,
        framebuffer: [u8; FRAMEBUFFER_SIZE],
        palette: [u8; 16],
    ) -> Result<(), Box<dyn Error>>;
}

trait Backend {
    fn call_update(&mut self);
    fn call_start(&mut self);
    fn read_screen(&self, framebuffer: &mut [u8; FRAMEBUFFER_SIZE], palette: &mut [u8; 16]);
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
