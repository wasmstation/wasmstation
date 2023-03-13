use std::{env, ffi::OsStr, fs, path::PathBuf, process, str::FromStr};

use argh::FromArgs;
use log::error;
use wasmstation::{backend::WasmerBackend, console::Console, renderer::LaunchConfig};

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
    #[argh(option, short = 'd', default = "3")]
    display_scale: u32,

    /// path to the save file used by the game
    #[argh(option, short = 's', from_str_fn(validate_save_path))]
    save_path: Option<PathBuf>,
}

fn run(args: Run) {
    let wasm_bytes = match fs::read(&args.path) {
        Ok(bytes) => bytes,
        Err(err) => {
            error!("Failed to read the game cart: {err}");
            process::exit(1);
        }
    };

    let save_path = args.save_path.unwrap_or(save_path_from_cart(&args.path));
    let title = args
        .path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or("wasmstation".to_string());

    wasmstation::launch(
        WasmerBackend::new(&wasm_bytes, &Console::new()).unwrap(),
        LaunchConfig::from_savefile(save_path, args.display_scale, &title),
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
    #[argh(option, short = 'd', default = "3")]
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
    let main_rs = include_str!("template/main.rs")
        .replace("{window_scale}", &args.display_scale.to_string())
        .replace("{crate_name}", &name);

    fs::create_dir_all(&base_dir.join("src")).expect("create main directory");
    fs::write(base_dir.join("Cargo.toml"), cargo_toml).expect("create Cargo.toml");
    fs::write(base_dir.join("build.rs"), build_rs).expect("create build.rs");
    fs::write(base_dir.join("src").join("main.rs"), main_rs).expect("create main.rs");
    fs::copy(&args.cart, base_dir.join(&format!("{name}.wasm"))).expect("copy wasm cart");

    println!(
        "Created your wasmstation project at {}\n\nBuild the cart by running:\n    $ cd {name}\n    $ cargo build --release",
        base_dir.to_string_lossy()
    );
}

/// Verify that the user's `/path/to/cart.wasm` is a file with a .wasm extension.
fn validate_wasm_path(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from_str(path).map_err(|err| err.to_string())?;

    if path.file_name() == None {
        return Err("must be a file".to_string());
    }

    if path.extension() != Some(OsStr::new("wasm")) {
        return Err("file extension should be .wasm".to_string());
    }

    Ok(path)
}

/// Verify that the user's `/path/to/cart.disk` is a file.
fn validate_save_path(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from_str(path).map_err(|err| err.to_string())?;

    if path.file_name().is_none() {
        return Err("must be a file".to_string());
    }

    Ok(path)
}

/// Generate a save path `/path/to/cart.disk` from `/path/to/cart.wasm`.
fn save_path_from_cart(path: &PathBuf) -> PathBuf {
    let mut path = path.clone();

    path.set_extension("disk");
    path
}
