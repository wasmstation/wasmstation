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
        default = "String::from(\"wgpu\")",
        description = "renderer to use"
    )]
    renderer: String,
}

fn main() {
    let args: Args = argh::from_env();
    let wasm_bytes = fs::read(&args.path).expect("failed to read game");

    let renderer = match ProxyRenderer::from_name(&args.renderer, args.display_scale as u32) {
        Ok(r) => r,
        Err(_) => {
            eprintln!("renderer '{}' is unknown, supported renderers are: {:?}", args.renderer, ProxyRenderer::names());
            std::process::exit(1)
        }
    };
    
    wasmstation::launch(
        WasmerBackend::new(&wasm_bytes).unwrap(),
        renderer
    );
}
