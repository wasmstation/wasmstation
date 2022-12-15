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
    #[argh(option, short = 'r', description = "renderer to use")]
    renderer: Option<String>,
}

fn main() {
    let args: Args = argh::from_env();
    let renderer_name = args
        .renderer
        .as_ref()
        .map(|s| String::from(s))
        .unwrap_or_else(|| String::from(ProxyRenderer::default_name()));
    let wasm_bytes = fs::read(&args.path).expect("failed to read game");

    let renderer = match ProxyRenderer::from_name(&renderer_name, args.display_scale as u32) {
        Ok(r) => r,
        Err(_) => {
            eprintln!(
                "renderer '{}' is unknown, supported renderers are: {:?}",
                &renderer_name,
                ProxyRenderer::names()
            );
            std::process::exit(1)
        }
    };

    wasmstation::launch(WasmerBackend::new(&wasm_bytes).unwrap(), renderer);
}
