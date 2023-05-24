use wasm_bindgen::prelude::*;
use wasmstation::{Console, WasmiBackend};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(msg: &str);
}

fn main() {
    console_log::init_with_level(log::Level::Info).expect("error initializing logger");

    wasmstation::gpu_renderer::launch_web(
        WasmiBackend::from_bytes(
            include_bytes!("../cart.wasm"),
            &Console::new(Box::new(|msg| log(msg))),
        )
        .unwrap(),
        "wasmstation",
        3,
    ).unwrap();
}
