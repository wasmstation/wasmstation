//! Implementations of WASM-4 Engine Functions

mod audio;
mod framebuffer;
mod trace;

use core::cell::Cell;

use audio::{AudioInterface, AudioState};
pub use framebuffer::*;
pub use trace::tracef;

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
            return Some(self.save_cache.get());
        } else {
            return None;
        }
    }
}
