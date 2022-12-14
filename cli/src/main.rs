use std::{fs, path::PathBuf};

use argh::FromArgs;
use wasmstation::{backend::WasmerBackend, renderer::ProxyRenderer};

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
    #[argh(
        option,
        short = 'r',
        description = "which renderer to use",
        default = "ProxyRenderer::default()"
    )]
    renderer: ProxyRenderer,
}

fn main() {
    let args: Args = argh::from_env();
    let wasm_bytes = fs::read(&args.path).expect("failed to read game");

    let mut renderer = args.renderer;
    renderer.set_display_scale(args.display_scale as u32);

    wasmstation::launch(WasmerBackend::new(&wasm_bytes).unwrap(), renderer);
}
