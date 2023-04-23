use std::env;

use wasmstation_core::Console;
use wasmstation_wasmer::WasmerBackend;

const WASM_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/wasm.module"));

fn main() {
    wasmstation_desktop::launch(
        WasmerBackend::precompiled(WASM_BYTES, &Console::new(|s| println!("{s}"))).unwrap(),
        &env::current_dir().unwrap(),
        { window_scale },
    )
    .unwrap();
}
