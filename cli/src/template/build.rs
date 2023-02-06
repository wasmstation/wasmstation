use std::{env, fs};
use wasmer::{Module, Store};

fn main() {
    fs::write(
        format!("{}/wasm.module", env::var("OUT_DIR").unwrap()),
        Module::new(&Store::default(), include_bytes!("cart.wasm"))
            .unwrap()
            .serialize()
            .unwrap(),
    )
    .unwrap();
}
