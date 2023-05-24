use std::{env, fs};
use wasmstation::wasmer_backend::wasmer::{Module, Store};

fn main() {
    fs::write(
        format!("{}/wasm.module", env::var("OUT_DIR").unwrap()),
        Module::new(&Store::default(), include_bytes!("{cart_name}.wasm"))
            .unwrap()
            .serialize()
            .unwrap(),
    )
    .unwrap();
}