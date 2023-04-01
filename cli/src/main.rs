use std::{env, ffi::OsStr, fs, path::PathBuf, str::FromStr};

use argh::FromArgs;
use wasmstation_core::Console;
use wasmstation_wasmer::WasmerBackend;

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

    wasmstation_desktop::launch(
        WasmerBackend::new(&wasm_bytes, &Console::new()).unwrap(),
        &args.path,
        args.display_scale,
    )
    .unwrap();
}

/// Create a Rust project for wrapping a WASM-4 game into a native executable in the current directory.
#[derive(FromArgs)]
#[argh(subcommand, name = "create")]
struct Create {
    /// path leading to the game's cartridge file (e.g. /path/to/cart.wasm)
    #[argh(positional, from_str_fn(validate_wasm_path))]
    cart: PathBuf,
    /// default scale factor of the window
    #[argh(option, short = 's', default = "3")]
    display_scale: u32,
}

fn create(args: Create) {
    // args.cart is guaranteed to be a file ideally so this shouldn't panic.
    let name: String = args
        .cart
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or("cart".to_string());

    let base_dir = env::current_dir()
        .expect("get current directory")
        .join(&name);

    let cargo_toml = include_str!("template/Cargo.toml").replace("{crate_name}", &name);
    let build_rs = include_str!("template/build.rs").replace("{cart_name}", &name);
    let main_rs =
        include_str!("template/main.rs").replace("{window_scale}", &args.display_scale.to_string());

    fs::create_dir_all(base_dir.join("src")).expect("create main directory");
    fs::write(base_dir.join("Cargo.toml"), cargo_toml).expect("create Cargo.toml");
    fs::write(base_dir.join("build.rs"), build_rs).expect("create build.rs");
    fs::write(base_dir.join("src").join("main.rs"), main_rs).expect("create main.rs");
    fs::copy(&args.cart, base_dir.join(format!("{name}.wasm"))).expect("copy wasm cart");

    println!(
        "Created your wasmstation project at {}\n\nBuild the cart by running:\n    $ cd {name}\n    $ cargo build --release",
        base_dir.to_string_lossy()
    );
}

fn validate_wasm_path(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from_str(path).map_err(|err| err.to_string())?;

    if path.file_name().is_none() {
        return Err("must be a file".to_string());
    }

    if path.extension() != Some(OsStr::new("wasm")) {
        return Err("file extension should be .wasm".to_string());
    }

    Ok(path)
}
