use std::{fs, path::PathBuf};

use argh::FromArgs;
use wasmstation::backend::WasmerBackend;
use wasmstation::console::Console;

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

    pretty_env_logger::init();

    let wasm_bytes = fs::read(&args.path).expect("failed to read game");
    let console = Console::new();

    wasmstation::launch(
        WasmerBackend::new(&wasm_bytes, &console).unwrap(),
        &args.path,
        args.display_scale,
    )
    .unwrap();
}
