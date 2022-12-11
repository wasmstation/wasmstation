#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![allow(unused_variables, dead_code)]

use std::error::Error;

pub mod wasm4;
pub mod utils;

/// Wgpu renderer.
#[cfg(feature = "wgpu-renderer")]
mod wgpu_renderer;
#[cfg(feature = "wgpu-renderer")]
pub use wgpu_renderer::WgpuRenderer;

/// Wasmer backend.
#[cfg(feature = "wasmer-backend")]
mod wasmer_backend;
#[cfg(feature = "wasmer-backend")]
pub use wasmer_backend::WasmerBackend;

/// Common trait for game renderers.
pub trait Renderer {
    fn render(
        &mut self,
        framebuffer: [u8; wasm4::FRAMEBUFFER_SIZE],
        palette: [u8; 16],
    ) -> Result<(), Box<dyn Error>>;
}

/// Common trait for webassembly backends.
pub trait Backend {
    fn call_update(&mut self);
    fn call_start(&mut self);
    fn read_screen(&self, framebuffer: &mut [u8; wasm4::FRAMEBUFFER_SIZE], palette: &mut [u8; 16]);
    fn set_gamepad(gamepad: u32);
    fn set_mouse(x: i16, y: i16, buttons: u8);
}

/// Common trait for reading from game memory.
pub trait Source<T>
where
    T: Copy,
{
    fn item_at(&self, offset: usize) -> T;
}

/// Common trait for writing to game memory.
pub trait Sink<T>
where
    T: Copy,
{
    fn set_item_at(&mut self, offset: usize, item: T);
}
