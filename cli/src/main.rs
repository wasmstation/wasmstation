use std::env;
use std::{fs, path::PathBuf};

use argh::FromArgs;
use wasmstation::{backend::WasmerBackend, console::Console};

#[derive(FromArgs)]
#[argh(description = "Run wasm4 compatible games.")]
struct Args {
    #[argh(subcommand)]
    subcommand: Subcommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Subcommand {
    Run(Run),
    Create(Create),
}

fn main() {
    let args: Args = argh::from_env();

    pretty_env_logger::init();

    match args.subcommand {
        Subcommand::Run(args) => run(args),
        Subcommand::Create(args) => create(args),
    }
}

/// Run a WASM-4 game.
#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
struct Run {
    #[argh(positional)]
    path: PathBuf,
    /// default scale factor for the window
    #[argh(option, short = 's', default = "3")]
    display_scale: u32,
}

fn run(args: Run) {
    let wasm_bytes = fs::read(&args.path).expect("failed to read game");

    wasmstation::launch(
        WasmerBackend::new(&wasm_bytes, &Console::new()).unwrap(),
        &args.path,
        args.display_scale,
    )
    .unwrap();
}

/// Create an executable from a WASM-4 game.
#[derive(FromArgs)]
#[argh(subcommand, name = "create")]
struct Create {
    #[argh(positional)]
    path: PathBuf,
    /// scale factor of the window
    #[argh(option, short = 's', default = "3")]
    display_scale: u32,
    /// name of the game
    #[argh(option, short = 't', default = "\"cart\".to_string()")]
    name: String,
}

fn create(args: Create) {
    let base_dir = env::current_dir()
        .expect("get current dir")
        .join(&args.name);

    let cargo_toml = include_str!("template/Cargo.toml").replace("cart", &args.name);
    let build_rs = include_str!("template/build.rs");
    let main_rs = include_str!("template/main.rs").replace("3", &args.display_scale.to_string());

    fs::create_dir_all(&base_dir.join("src")).expect("create main directory");
    fs::write(base_dir.join("Cargo.toml"), cargo_toml).expect("create Cargo.toml");
    fs::write(base_dir.join("build.rs"), build_rs).expect("create build.rs");
    fs::write(base_dir.join("src").join("main.rs"), main_rs).expect("create main.rs");
    fs::copy(&args.path, base_dir.join("cart.wasm")).expect("copy wasm cart");

    println!(
        "Created your wasmstation project at {}\n",
        base_dir.to_string_lossy()
    );
    println!(
        "Build the cart by running:\n    $ cd {}\n    $ cargo build --release",
        &args.name
    );
}
