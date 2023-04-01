use std::env;

use wasmstation_wasmer::WasmerBackend;
use wasmstation_core::Console;

const WASM_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/wasm.module"));

fn main() {
    wasmstation_desktop::launch(
        WasmerBackend::precompiled(WASM_BYTES, &Console::new()).unwrap(),
        &env::current_dir().unwrap(),
        {window_scale},
    )
    .unwrap();
}
