#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use core::cell::Cell;

pub mod utils;
pub mod wasm4;
pub mod framebuffer;
pub mod trace;
mod audio;

use audio::{AudioInterface, AudioState};

#[doc(inline)]
pub use framebuffer::{text, line, hline, vline, oval, rect, blit_sub};
#[doc(inline)]
pub use trace::tracef;

/// Common trait for WebAssembly backends.
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
    /// Tell the renderer to save the cache to disk.
    fn write_save_cache(&mut self) -> Option<[u8; 1024]>;
    /// Set the backend's save cache.
    fn set_save_cache(&mut self, data: [u8; 1024]);
}

/// Common trait for reading from game memory.
///
/// A [`Source<T>`] reads from a cart's memory subregion that is
/// defined by the Source's provider. For instance, a [`Source<u8>`] provided
/// for reading the frame buffer will cover reading the frame buffer,
/// but no other regions, where offset 0 marks the first framebuffer byte.
pub trait Source<T>
where
    T: Copy,
{
    /// Read memory at the specified offset, relative to the start
    /// of the memory subregion the [`Source<T>`] covers.
    fn item_at(&self, offset: usize) -> Option<T>;

    /// Like [`items_at`](Source::items_at), but reads multiple values.
    fn items_at<const L: usize>(&self, offset: usize) -> Option<[T; L]>;
}

impl<T: Copy> Source<T> for Vec<T> {
    fn item_at(&self, offset: usize) -> Option<T> {
        self.get(offset).copied()
    }

    fn items_at<const L: usize>(&self, offset: usize) -> Option<[T; L]> {
        self.get(offset..(offset + L))
            .map(|s| s.try_into().unwrap())
    }
}

impl<const N: usize, T: Copy> Source<T> for [T; N] {
    fn item_at(&self, offset: usize) -> Option<T> {
        self.get(offset).copied()
    }

    fn items_at<const L: usize>(&self, offset: usize) -> Option<[T; L]> {
        self.get(offset..(offset + L))
            .map(|s| s.try_into().unwrap())
    }
}

/// Common trait for writing to game memory.
///
/// A [`Sink<T>`] writes to a cart's memory region that is defined by the Sink's provider.
/// Like [`Source<T>`], a [`Sink<T>`] may only cover a specific memory subregion.
pub trait Sink<T>
where
    T: Copy,
{
    /// Write memory at the specified offset, relative to the start
    /// of the memory subregion the [`Sink<T>`] covers.
    fn set_item_at(&mut self, offset: usize, item: T);

    /// Fill the entire memory subregion with values of T by
    /// cloning `item`
    fn fill(&mut self, item: T);
}

impl<T> Sink<T> for Vec<T>
where
    T: Copy,
{
    fn set_item_at(&mut self, offset: usize, item: T) {
        self[offset] = item
    }

    fn fill(&mut self, item: T) {
        <[T]>::fill(self, item)
    }
}

impl<const N: usize, T> Sink<T> for [T; N]
where
    T: Copy,
{
    fn set_item_at(&mut self, offset: usize, item: T) {
        self[offset] = item
    }

    fn fill(&mut self, item: T) {
        <[T]>::fill(self, item)
    }
}

pub struct Console {
    audio_state: AudioState,
}

impl Console {
    pub fn new() -> Self {
        Self {
            audio_state: AudioState::new(),
        }
    }

    pub fn create_api(&self) -> Api {
        Api {
            audio_api: self.audio_state.api().clone(),
            save_cache: Cell::new([0; 1024]),
            needs_write: Cell::new(false),
        }
    }

    pub fn update(&self) {
        self.audio_state.api().update()
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct Api {
    audio_api: AudioInterface,
    pub save_cache: Cell<[u8; 1024]>,
    pub needs_write: Cell<bool>,
}

impl Api {
    pub fn tone(&self, frequency: u32, duration: u32, volume: u32, flags: u32) {
        self.audio_api.tone(frequency, duration, volume, flags)
    }

    pub fn write_save(&self) -> Option<[u8; 1024]> {
        if self.needs_write.get() {
            self.needs_write.set(false);
            Some(self.save_cache.get())
        } else {
            None
        }
    }
}

