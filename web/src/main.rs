#![no_std]

fn main() {
    wasmstation_web::launch_web("wasmstation", include_bytes!("basic.wasm"));
}
