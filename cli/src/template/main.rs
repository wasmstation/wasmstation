use std::{env, path::PathBuf, str::FromStr};
use wasmstation::{backend::WasmerBackend, console::Console, renderer::LaunchConfig};

mod disk;

const WASM_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/wasm.module"));

fn main() {
    let save_path = PathBuf::from_str("{crate_name}.disk").expect("create save file path");

    wasmstation::launch(
        WasmerBackend::new(&wasm_bytes, &Console::new()).unwrap(),
        LaunchConfig::from_savefile(save_path, args.display_scale, &title),
    )
    .unwrap();
}
