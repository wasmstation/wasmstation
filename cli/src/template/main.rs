use std::{env, PathBuf};
use wasmstation::{backend::WasmerBackend, console::Console};

mod disk;

const WASM_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/wasm.module"));

fn main() {
    let save_path = PathBuf::new("{crate_name}.disk");
    
    wasmstation::launch(
        WasmerBackend::precompiled(WASM_BYTES, &Console::new()).unwrap(),
        disk::write(&save_path),
        disk::read(&save_path),
        {window_scale},
        "{crate_name}",
    )
    .unwrap();
}
