#![no_main]
#![no_std]

use wasmstation_web::{wasm_bindgen::prelude::*, launch_canvas};
use wasm_logger::Config;

#[wasm_bindgen(start)]
pub fn start() {
    wasm_logger::init(Config::default());

    launch_canvas(include_bytes!("hello.wasm"), "canvas");
}