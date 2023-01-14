#![doc = include_str!("../../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![allow(unused_variables, dead_code)]

pub mod utils;
pub mod wasm4;

pub mod backend;
mod console;
pub mod renderer;

pub use renderer::launch;

/// Common trait for webassembly backends.
pub trait Backend {
    /// Call the cart's `update()` function. 
    /// See [Callbacks](https://wasm4.org/docs/reference/functions#callbacks)
    fn call_update(&mut self);
    /// Call the cart's `start()` function. 
    /// See [Callbacks](https://wasm4.org/docs/reference/functions#callbacks)
    fn call_start(&mut self);
    /// Read the content of the [FRAMEBUFFER](https://wasm4.org/docs/reference/memory#framebuffer)
    /// memory region
    fn read_screen(&self, framebuffer: &mut [u8; wasm4::FRAMEBUFFER_SIZE], palette: &mut [u8; 16]);
    /// Provide the content of the [SYSTEM_FLAGS](https://wasm4.org/docs/reference/memory#system_flags) register. 
    fn read_system_flags(&self) -> u8;
    /// Set the [GAMEPADS](https://wasm4.org/docs/reference/memory#gamepads)
    /// register, where the cart will read gamepad input from.
    fn set_gamepad(&mut self, gamepad: u32);
    /// Set the [MOUSE_X](https://wasm4.org/docs/reference/memory#mouse_x),
    /// [MOUSE_Y](https://wasm4.org/docs/reference/memory#mouse_y) and
    /// [MOUSE_BUTTONS](https://wasm4.org/docs/reference/memory#mouse_buttons)
    /// registers, where the cart will read mouse input from.
    fn set_mouse(&mut self, x: i16, y: i16, buttons: u8);
    /// Tells the renderer to save the save cache to disk.
    fn write_save(&mut self) -> Option<[u8; 1024]>;
    /// Set the backend's save cache.
    fn set_save(&mut self, data: [u8; 1024]);
}

/// Common trait for reading from game memory. A `Source<T>` reads from
/// a cart's memory subregion that is defined by the Source's provider.
/// For instance, a Source<u8> provided for reading the frame buffer 
/// will cover reading the frame buffer, but no other regions, where
/// offset 0 marks the first framebuffer byte.
pub trait Source<T>
where
    T: Copy,
{
    /// Read memory at the specified offset, relative to the start
    /// of the memory subregion the `Source<T>` covers.
    fn item_at(&self, offset: usize) -> T;
}

/// Common trait for writing to game memory. A `Sink<T>` writes to
/// a cart's memory region that is defined by the Sink's provider.
/// Like [Source<T>], a `Sink<T>` may only cover a specific memory
/// subregion.

pub trait Sink<T>
where
    T: Copy,
{
    /// Write memory at the specified offset, relative to the start
    /// of the memory subregion the `Sink<T>` covers.
    fn set_item_at(&mut self, offset: usize, item: T);

    /// Fill the entire memory subregion with values of T by
    /// cloning `item`
    fn fill(&mut self, item: T);
}
