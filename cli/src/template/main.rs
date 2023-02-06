use std::env;

use wasmstation::{backend::WasmerBackend, console::Console};

const WASM_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/wasm.module"));

fn main() {
    wasmstation::launch(
        WasmerBackend::precompiled(WASM_BYTES, &Console::new()).unwrap(),
        &env::current_dir().unwrap(),
        3,
    )
    .unwrap();
}
