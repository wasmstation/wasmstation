#![no_std]

use js_sys::{WebAssembly::{Instance, self}, Object, Reflect};
use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen_futures::JsFuture;
use wasm_logger::Config;

pub use wasm_bindgen;

pub async fn launch_canvas(bytes: &[u8], id: &str) -> Result<(), JsValue> {
    wasm_logger::init(Config::default());

    log::info!("starting backend...");
    let mut backend = WebBackend::from_bytes(bytes).await?;

    Ok(())
}

struct WebBackend {
    instance: Instance,
}

impl WebBackend {
    pub async fn from_bytes(bytes: &[u8]) -> Result<Self, JsValue> {
        log::debug!("instantiating from wasm buffer");
        let instantiate = JsFuture::from(WebAssembly::instantiate_buffer(bytes, &Object::new())).await?;
        let instance: Instance = Reflect::get(&instantiate, &"instance".into())?.dyn_into()?;
        
        Ok(Self {
            instance,
        })
    }
}