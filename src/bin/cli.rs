use std::{path::PathBuf, fs};

use argh::FromArgs;

#[derive(FromArgs)]
#[argh(description = "Run wasm4 compatible games.")]
struct Args {
    #[argh(positional)]
    path: PathBuf,
}

fn main() {
    let args: Args = argh::from_env();

    let wasm_bytes = fs::read(&args.path).expect("failed to read game");

    // TODO: open window, handle input, etc.
}
