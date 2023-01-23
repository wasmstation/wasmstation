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

    pretty_env_logger::init();

    wasmstation::launch(
        WasmerBackend::new(&wasm_bytes).unwrap(),
        &args.path,
        args.display_scale,
    )
    .unwrap();
}
