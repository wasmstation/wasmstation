use std::env;

use wasmstation::{WasmerBackend, Console, sdl2_renderer::launch_desktop};

const WASM_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/wasm.module"));

fn main() {
    launch_desktop(
        WasmerBackend::precompiled(WASM_BYTES, &Console::default()).unwrap(),
        &env::current_dir().unwrap(),
        {window_scale},
    )
    .unwrap();
}
