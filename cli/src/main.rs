use argh::FromArgs;
use std::{fs, path::PathBuf};
use wasmstation::{backend::WasmerBackend, renderer::DefaultRenderer};

#[derive(FromArgs)]
#[argh(description = "Run wasm4 compatible games.")]
struct Args {
    #[argh(positional)]
    path: PathBuf,
}

fn main() {
    let args: Args = argh::from_env();
    let wasm_bytes = fs::read(&args.path).expect("failed to read game");

    wasmstation::launch(
        WasmerBackend::new(&wasm_bytes).unwrap(),
        DefaultRenderer::default(),
    );
}
