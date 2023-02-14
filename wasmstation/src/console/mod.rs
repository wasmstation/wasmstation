//! Implementations of WASM-4 Engine Functions

mod audio;
mod framebuffer;

use audio::{AudioInterface, AudioState};
pub use framebuffer::{
    blit_sub, clear, hline, line, oval, pixel_width_of_flags, rect, text, vline,
};

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
        }
    }

    pub fn update(&self) {
        self.audio_state.api().update()
    }
}

#[derive(Clone)]
pub struct Api {
    audio_api: AudioInterface,
}

impl Api {
    pub fn tone(&self, frequency: u32, duration: u32, volume: u32, flags: u32) {
        self.audio_api.tone(frequency, duration, volume, flags)
    }
}
