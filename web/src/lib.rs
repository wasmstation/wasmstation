use wasm_logger::Config;

pub use wasm_bindgen;

pub fn launch_canvas(bytes: &[u8], id: &str) {
    wasm_logger::init(Config::default());

    log::info!("starting backend...");
    let mut backend = WebBackend::from_bytes(bytes);
}

struct WebBackend {
    // instance: Instance,
}

impl WebBackend {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            
        }
    }
    
}