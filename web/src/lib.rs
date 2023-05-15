#![no_std]

use wasm_bindgen::prelude::*;

use core::f64::consts::PI;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(msg: &str);
}

pub fn launch_web(id: &str, wasm: &[u8]) {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(id).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    context.begin_path();

    // Draw the outer circle.
    context.arc(75.0, 75.0, 50.0, 0.0, PI * 2.0).unwrap();

    // Draw the mouth.
    context.move_to(110.0, 75.0);
    context.arc(75.0, 75.0, 35.0, 0.0, PI).unwrap();

    // Draw the left eye.
    context.move_to(65.0, 65.0);
    context.arc(60.0, 65.0, 5.0, 0.0, PI * 2.0).unwrap();

    // Draw the right eye.
    context.move_to(95.0, 65.0);
    context.arc(90.0, 65.0, 5.0, 0.0, PI * 2.0).unwrap();

    context.stroke();
}
