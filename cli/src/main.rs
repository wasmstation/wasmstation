use std::{env, process::{self, Command}};
use std::{ffi::OsStr, fs, path::PathBuf, str::FromStr};

use argh::FromArgs;
use wasmstation::{backend::WasmerBackend, console::Console};

const CARGO_TOML_TEMPLATE: &str = include_str!("template/Cargo.toml");
const MAIN_RS_TEMPLATE: &str = include_str!("template/main.rs");

#[derive(FromArgs)]
/// Run WASM-4 compatible games.
struct Args {
    #[argh(subcommand)]
    subcommand: Subcommands,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Subcommands {
    Run(Run),
    Build(Build),
}

fn main() {
    let args: Args = argh::from_env();

    pretty_env_logger::init();

    match args.subcommand {
        Subcommands::Run(args) => run(args),
        Subcommands::Build(args) => build(args),
    }
}

#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
/// Run a WASM-4 cart.
struct Run {
    #[argh(positional)]
    cart: PathBuf,

    /// default window scale factor
    #[argh(option, short = 's', default = "3")]
    display_scale: u32,
}

fn run(args: Run) {
    let wasm_bytes = fs::read(&args.cart).expect("read game file");
    let console = Console::new();

    let file_name = &args.cart.file_name().unwrap_or(OsStr::new("cart"));

    let title = file_name
        .to_str()
        .unwrap_or("cart")
        .split('.')
        .next()
        .unwrap_or("cart")
        .replace('-', " ")
        .replace('_', " ");

    let mut save_file = env::current_dir().expect("current directory");
    save_file.push(&title);
    save_file.set_extension("disk");

    wasmstation::launch(
        WasmerBackend::new(&wasm_bytes, &console).unwrap(),
        &title,
        args.display_scale,
        &save_file,
    )
    .unwrap();
}

#[derive(FromArgs)]
/// Build a native binary for a WASM-4 game.
#[argh(subcommand, name = "build")]
struct Build {
    /// path to wasm input
    #[argh(positional)]
    cart: PathBuf,

    /// path to executable output
    #[argh(
        option,
        short = 'o',
        long = "out",
        default = "PathBuf::from_str(\"output.cart\").unwrap()"
    )]
    out: PathBuf,

    /// default window scale factor
    #[argh(option, short = 's', default = "3")]
    display_scale: u32,

    /// name of the game on the window
    #[argh(option, short = 't', default = "\"cart\".to_string()")]
    title: String,
}

fn build(args: Build) {
    let tempdir = tempfile::tempdir().expect("get temporary directory");

    fs::write(tempdir.path().join("Cargo.toml"), CARGO_TOML_TEMPLATE).expect("write Cargo.toml");
    fs::create_dir_all(tempdir.path().join("src")).expect("create src dir");
    fs::write(tempdir.path().join("src/main.rs"), MAIN_RS_TEMPLATE).expect("write main.rs");

    let savefile = args.cart.file_name().unwrap_or(OsStr::new("savefile"));
    let display_scale = &args.display_scale.to_string();

    let env = [
        ("CART_SAVEFILE_NAME", savefile),
        ("CART_PATH", &args.cart.as_os_str()),
        ("CART_TITLE", OsStr::new(&args.title)),
        ("CART_DISPLAY_SCALE", OsStr::new(display_scale)),
    ];

    // if cfg!(target_os = "windows") {
    //     Command::new("cmd")
    //         .current_dir(tempdir.path())
    //         .args(["/C", "cargo build --release"])
    //         .envs(env)
    //         .output()
    //         .expect("run cargo build");

    //     fs::copy(tempdir.path().join("\\target\\release\\cart.exe"), args.out)
    //         .expect("copy resulting executable to out directory");
    // } else {
    let status = Command::new("sh")
        .current_dir(tempdir.path())
        .args(["-c", "cargo build --release"])
        .envs(env)
        .status()
        .expect("run cargo build");

    if !status.success() {
        process::exit(1);
    }

    fs::copy(tempdir.path().join("target/release/cart"), args.out)
        .expect("copy resulting executable to out directory");
    // }
}
