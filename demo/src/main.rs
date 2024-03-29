use wasmstation::{gpu_renderer, Console, WasmiBackend};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_namespace = console)]
extern "C" {
    fn log(msg: &str);
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).expect("error initializing logger");
        console_error_panic_hook::set_once();
    }

    gpu_renderer::launch(
        WasmiBackend::from_bytes(include_bytes!(env!("CART")), &Console::default()).unwrap(),
        "wasmstation",
        3,
    )
    .unwrap();
}
