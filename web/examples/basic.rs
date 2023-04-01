#![no_main]

use wasmstation_web::{wasm_bindgen::prelude::*, launch_canvas};

#[wasm_bindgen(start)]
pub fn start() {
    launch_canvas(include_bytes!("watris.wasm"), "canvas");
}