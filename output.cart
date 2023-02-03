use std::env;

use wasmstation::{launch, wasmer_precompiled};

fn main() {
    let mut save_file = env::current_dir().expect("get current directory");
    save_file.push(env!("CART_SAVEFILE_NAME"));
    save_file.set_extension(".disk");
    
    launch(
        wasmer_precompiled!(env!("CART_PATH")),
        env!("CART_TITLE"),
        env!("CART_DISPLAY_SCALE").parse::<u32>().unwrap(),
        save_file,
    ).expect("run game");
}
