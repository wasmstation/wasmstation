use std::{fs, path::PathBuf};

use argh::FromArgs;
use wasmstation::backend::WasmerBackend;

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
    display_scale: u32,
}

fn main() {
    let args: Args = argh::from_env();
    let wasm_bytes = fs::read(&args.path).expect("failed to read game");

    let title = args
        .path
        .file_name()
        .map(|t| t.to_str().unwrap_or("wasmstation"))
        .unwrap_or("wasmstation")
        .split('.')
        .next()
        .unwrap_or("wasmstation")
        .replace("-", " ");

    pretty_env_logger::init();

    wasmstation::launch(
        WasmerBackend::new(&wasm_bytes).unwrap(),
        &title,
        args.display_scale,
    )
    .unwrap();
}
