use std::{fs, path::PathBuf};

use argh::FromArgs;
use wasmstation::{utils, wasm4, Backend, Renderer, WasmerBackend, WgpuRenderer};

#[derive(FromArgs)]
#[argh(description = "Run wasm4 compatible games.")]
struct Args {
    #[argh(positional)]
    path: PathBuf,
    #[argh(
        option,
        short = 's',
        default = "3",
        description = "scale factor for the window"
    )]
    display_scale: u8,
}

fn main() {
    let args: Args = argh::from_env();
    let wasm_bytes = fs::read(&args.path).expect("failed to read game");

    wasmstation::launch(
        WasmerBackend::new(&wasm_bytes).unwrap(),
        WgpuRenderer::new_blocking(args.display_scale as u32).unwrap(),
    );
}
