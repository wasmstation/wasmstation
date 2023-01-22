

use self::audio::{AudioState, AudioInterface};

pub(crate) mod fb;
pub(crate) mod audio;

pub struct Console {
    audio_state: AudioState
}

impl Console {

    pub fn new() -> Self {
        Self { audio_state: AudioState::new() }
    }

    pub fn create_api(&self) -> Api {
        Api {
            audio_api: self.audio_state.api().clone()
        }
    }

    pub fn update(&self) {
        self.audio_state.api().update()
    }

}

#[derive(Clone)]
pub struct Api {
    audio_api: AudioInterface
}

impl Api {
    pub fn tone(&self, frequency: u32, duration: u32, volume: u32, flags: u32) {
        self.audio_api.tone(frequency, duration, volume, flags)
    }
}