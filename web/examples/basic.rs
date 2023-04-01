#![no_main]
#![no_std]

use wasmstation_web::{wasm_bindgen::prelude::*, launch_canvas};

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    launch_canvas(include_bytes!("watris.wasm"), "canvas").await
}