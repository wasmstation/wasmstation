use wasmstation::{gpu_renderer, Console, WasmiBackend};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(msg: &str);
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_log::init_with_level(log::Level::Info).expect("error initializing logger");

    #[cfg(target_arch = "wasm32")]
    let print = Box::new(|msg: &str| log(msg));
    #[cfg(not(target_arch = "wasm32"))]
    let print = Box::new(|msg: &str| println!("{msg}"));

    gpu_renderer::launch(
        WasmiBackend::from_bytes(include_bytes!(env!("CART")), &Console::new(print)).unwrap(),
        "wasmstation",
        3,
    )
    .unwrap();
}