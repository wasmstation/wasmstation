use std::{env, path::PathBuf, str::FromStr};
use wasmstation::{backend::WasmerBackend, console::Console, renderer::LaunchConfig};

const WASM_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/wasm.module"));

fn main() {
    let save_path = PathBuf::from_str("{crate_name}.disk").expect("create save file path");

    wasmstation::launch(
        WasmerBackend::precompiled(&WASM_BYTES, &Console::new()).unwrap(),
        LaunchConfig::from_savefile(save_path, {window_scale}, "{crate_name}"),
    )
    .unwrap();
}
