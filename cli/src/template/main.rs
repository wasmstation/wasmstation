use std::{env, path::PathBuf, str::FromStr};
use wasmstation::{backend::WasmerBackend, console::Console, renderer::LaunchConfig};

const WASM_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/wasm.module"));

fn main() {
    let save_path = PathBuf::from_str("{crate_name}.disk").expect("create save file path");

    let mut config = LaunchConfig::from_path(&save_path);
    config.display_scale = {window_scale};
    config.title = "{crate_name}";

    wasmstation::launch(
        WasmerBackend::precompiled(&WASM_BYTES, &Console::new()).unwrap(),
        config,
    )
    .unwrap();
}
