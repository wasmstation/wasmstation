use std::{env, ffi::OsStr, fs, path::PathBuf, process, str::FromStr};

use argh::FromArgs;
use wasmstation::{gpu_renderer, sdl2_renderer, Console, WasmerBackend, WasmiBackend};

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

    if let Err(err) = match args.subcommand {
        Subcommand::Run(args) => run(args),
        Subcommand::Create(args) => create(args),
    } {
        log::error!("Runtime Error: {err}");
        process::exit(1);
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
    /// webassembly backend used for executing the cart
    #[argh(option, short = 'b', default = "BackendType::default()")]
    backend: BackendType,
    /// renderer used for the window
    #[argh(option, short = 'r', default = "RendererType::default()")]
    renderer: RendererType,
}

fn run(args: Run) -> anyhow::Result<()> {
    let wasm_bytes = fs::read(&args.path)?;
    let console = Console::new(Box::new(|s| println!("{s}")));

    match args.renderer {
        RendererType::Sdl2 => match args.backend {
            BackendType::Wasmer => sdl2_renderer::launch_desktop(
                WasmerBackend::from_bytes(&wasm_bytes, &console)?,
                &args.path,
                args.display_scale,
            ),
            BackendType::Wasmi => sdl2_renderer::launch_desktop(
                WasmiBackend::from_bytes(&wasm_bytes, &console)?,
                &args.path,
                args.display_scale,
            ),
        },
        RendererType::Gpu => match args.backend {
            BackendType::Wasmer => gpu_renderer::launch_desktop(
                WasmerBackend::from_bytes(&wasm_bytes, &console)?,
                "Wasmstation CLI",
                args.display_scale,
            ),
            BackendType::Wasmi => gpu_renderer::launch_desktop(
                WasmiBackend::from_bytes(&wasm_bytes, &console)?,
                "Wasmstation CLI",
                args.display_scale,
            ),
        },
    }
}

#[derive(Copy, Clone, Default)]
enum BackendType {
    #[default]
    Wasmer,
    Wasmi,
}

impl FromStr for BackendType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "wasmi" => Ok(Self::Wasmi),
            "wasmer" => Ok(Self::Wasmer),
            _ => Err("backend type must be 'wasmi' or 'wasmer'".to_string()),
        }
    }
}

#[derive(Copy, Clone, Default)]
enum RendererType {
    #[default]
    Sdl2,
    Gpu,
}

impl FromStr for RendererType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sdl2" => Ok(Self::Sdl2),
            "sdl" => Ok(Self::Sdl2),
            "gpu" => Ok(Self::Gpu),
            "pixels" => Ok(Self::Gpu),
            _ => Err("backend type must be 'wasmi' or 'wasmer'".to_string()),
        }
    }
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

fn create(args: Create) -> anyhow::Result<()> {
    // args.cart is guaranteed to be a file ideally so this shouldn't panic.
    let name: String = args
        .cart
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .ok_or(anyhow::anyhow!("wasm name must have a file name"))?;

    let base_dir = env::current_dir()?.join(&name);

    let cargo_toml = include_str!("../../template/Cargo.toml").replace("{crate_name}", &name);
    let build_rs = include_str!("../../template/build.rs").replace("{cart_name}", &name);
    let main_rs = include_str!("../../template/main.rs")
        .replace("{window_scale}", &args.display_scale.to_string());

    fs::create_dir_all(base_dir.join("src"))?;
    fs::write(base_dir.join("Cargo.toml"), cargo_toml)?;
    fs::write(base_dir.join("build.rs"), build_rs)?;
    fs::write(base_dir.join("src").join("main.rs"), main_rs)?;
    fs::copy(&args.cart, base_dir.join(format!("{name}.wasm")))?;

    println!(
        "Created your wasmstation project at {}\n\nBuild the cart by running:\n    $ cd {name}\n    $ cargo build --release",
        base_dir.to_string_lossy()
    );

    Ok(())
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
