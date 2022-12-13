use std::{fs, path::PathBuf};

use argh::FromArgs;
use wasmstation::{backend::WasmerBackend, renderer::{WgpuRenderer, Sdl2Renderer}};

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
        Sdl2Renderer {
            //display_scale: args.display_scale as u32,
        },
    );
}
