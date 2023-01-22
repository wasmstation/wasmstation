use cpal::{traits::{DeviceTrait, HostTrait}, Stream, OutputCallbackInfo};
use log::warn;
use num_traits::*;
use std::sync::mpsc;
use crate::wasm4;

use crate::{FrameCount, wasm4::{TONE_PAN_LEFT, TONE_PAN_RIGHT}};

const TARGET_FPS: u32 = 60;
const MAX_VOLUME: u16 = 100;


#[derive(Clone)]
pub (crate) struct AudioInterface {
    command_sender: Option<mpsc::Sender<AudioCommand>>,
}

type Sample = i32;

pub (crate) struct AudioState {
    output: Option<CpalOutput>,
    api: AudioInterface,
}


impl AudioState {
    pub (crate) fn new() -> Self { 
        let (tx,rx) = mpsc::channel();
        let api = AudioInterface { command_sender: None };
        let mut s = Self {output: None, api };

        s.output = match Self::mk_output(AudioProcessor::new(rx)) {
            Ok(o) => {
                s.api.command_sender = Some(tx);
                Some(o) 
            }
            Err(e) => {
                warn!("no audio device used: {}", e);
                None
            }
        };
        s
    }

    pub fn api(&self) -> &AudioInterface {
        &self.api
    }

    fn mk_output<P>(mut audio_processor: AudioProcessor<P>) -> anyhow::Result<CpalOutput> 
    where P: AudioCommandPoller + Send + 'static
    {
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            None => return Err(anyhow::Error::msg("no default output device present")),
            Some(d) => d
        };
        let supported_config = device.default_output_config()?;
        let config = supported_config.config();
        audio_processor.set_sample_rate(config.sample_rate.0 as i32);
        let data_callback = move |data: &mut [f32],info: &OutputCallbackInfo|{
            audio_processor.render_audio(config.channels, data);
            let t = info.timestamp();
            ()
        };
        let error_callback = move |err| {
            warn!("{}", err)
        };
        let stream = device.build_output_stream(&config, data_callback, error_callback)?;
        
        Ok(CpalOutput { stream })
    }
}

impl AudioInterface {
    fn do_send(&self, cmd: AudioCommand) {
        if let Some(tx) = &self.command_sender {
            if let Err(e) = tx.send(cmd) {
                warn!("sending command to audio processor failed ({})", e);
            }
        }
    }

    pub fn tone(&self, frequency: u32, duration: u32, volume: u32, flags: u32) {
       self.do_send(AudioCommand::Tone(ToneSpec{frequency, duration, volume, flags}));
    }

    pub fn update(&self) {
        self.do_send(AudioCommand::NextFrame);
    }
}

struct CpalOutput {
    stream: Stream,
}

#[derive(Debug)]
enum AudioCommand {
    Tone(ToneSpec),
    NextFrame,
}

#[derive(Debug)]
struct ToneSpec {
    frequency: u32,
    duration: u32,
    volume: u32,
    flags: u32
}

struct AudioProcessor<P: AudioCommandPoller> {
    channels: [AudioChannel; 4],
    command_receiver: P,
    current_frame: FrameCount,
}

impl <P: AudioCommandPoller> AudioProcessor<P> {

    const MAX_AMPLITUDE: i32 = 0xffff;

    fn render_audio(&mut self, audio_channels: u16, data: &mut [f32]) {
        // process commands and apply them 
        while let Some(cmd)=self.command_receiver.poll() {
            match cmd {
                AudioCommand::NextFrame => self.current_frame += 1,
                AudioCommand::Tone(spec) => self.apply_tone(&spec)
            }
        }

        let mut left_right = (0,0);
        let mut data_ch_idx = 0;
        for sample in data.iter_mut() {
            if data_ch_idx == 0 {
                left_right = (0,0);
                for channel in &mut self.channels {
                    let out = channel.next();
                    left_right.0 += out.0;
                    left_right.1 += out.1;
                }
            }

            *sample = match data_ch_idx {
                0 => left_right.0 as f32 / Self::MAX_AMPLITUDE as f32,
                1 => left_right.1 as f32 / Self::MAX_AMPLITUDE as f32,
                _ => 0.0
            };

            data_ch_idx = (data_ch_idx+1) % audio_channels;
        }
    }

    fn apply_tone(&mut self, tone_spec: &ToneSpec) {
        // find out which channel this ToneSpec applies to
        let channel_idx = (tone_spec.flags & 0b11) as usize;
        let channel = &mut  self.channels[channel_idx];
        let config = ToneConfiguration::from_tone_spec(&tone_spec, channel);

        channel.pending_config = Some(config);
    }

    fn new(command_receiver: P) -> AudioProcessor<P> {
        AudioProcessor { 
            channels: [
                AudioChannel {generator: AudioGenerator::Pulse,    ..AudioChannel::default()},
                AudioChannel {generator: AudioGenerator::Pulse,    ..AudioChannel::default()},
                AudioChannel {generator: AudioGenerator::Triangle, ..AudioChannel::default()},
                AudioChannel {generator: AudioGenerator::Noise,    ..AudioChannel::default()},
            ], 
            command_receiver, 
            current_frame: 0 
        }
    }

    fn set_sample_rate(&mut self, sample_rate: i32) {
        for ch in &mut self.channels {
            ch.sample_rate = sample_rate;
        }
    }

}

trait AudioCommandPoller {
    fn poll(&mut self) -> Option<AudioCommand>;
}

impl AudioCommandPoller for mpsc::Receiver<AudioCommand> {
    fn poll(&mut self) -> Option<AudioCommand> {
        self.try_recv().ok()
    }
}

enum Mode {
    Mode1_12,
    Mode2_25,
    Mode3_50,
    Mode4_75
}

impl Mode {
    fn from_tone_flags(flags: u32) -> Self {
        match flags & 0b01_00_11_00 {
            wasm4::TONE_MODE1 => Mode::Mode1_12,
            wasm4::TONE_MODE2 => Mode::Mode2_25,
            wasm4::TONE_MODE3 => Mode::Mode3_50,
            wasm4::TONE_MODE4 => Mode::Mode4_75,
            _ => Mode::Mode1_12
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Mode1_12
    }
}


enum Pan {
    Center,
    Left,
    Right,
}

impl Pan {
    fn from_tone_flags(flags: u32) -> Self {
        match flags & 0b00_11_00_00 {
            TONE_PAN_LEFT => Pan::Left,
            TONE_PAN_RIGHT => Pan::Right,
            _ => Pan::Center
        }
    }
}

impl Default for Pan {
    fn default() -> Self {
        Pan::Center
    }
}

#[derive(Default)]
struct AudioChannel {

    start_frame: FrameCount,
    samples_rendered: i32,
    current_config: ToneConfiguration,
    pending_config: Option<ToneConfiguration>,
    generator: AudioGenerator,

    sample_rate: i32,

    // samples / freq
    phase: i32,
    current_freq: i32,
    current_volume: i32,
}

impl AudioChannel {
    fn next(&mut self) -> (Sample,Sample) {

        let phase_ended;
        if self.current_freq == 0 {
            // when the current_freq
            self.phase = 0;
            phase_ended = true;
        } else if self.phase >= self.sample_rate {
            self.phase -= self.sample_rate;
            phase_ended = true
        } else {
            phase_ended = false
        }

        if phase_ended {
            self.commit_pending_config();
            
            if self.samples_rendered >= self.current_config.release_end {
                self.current_freq = 0;
                self.current_volume = 0;
            } else {
                // recalculate volume
                self.current_volume = self.current_config.volume_at(self.samples_rendered);

                // recalculate frequency
                self.current_freq = self.current_config.frequency_at(self.samples_rendered);
            }
        }

        if self.current_freq == 0 {
            return (0, 0)
        }


        // render sample
        let gen = &mut self.generator;
        let current_output = gen.clone().render_sample(self);

        self.samples_rendered += 1;
        self.phase += self.current_freq;

        match &self.current_config.pan {
            Pan::Center => (current_output, current_output),
            Pan::Left   => (current_output, 0             ),
            Pan::Right  => (             0, current_output)
        }
    }

    fn commit_pending_config(&mut self) {
        if let Some(new_config) = self.pending_config.take() {
            self.phase = 0;
            self.samples_rendered = 0;
            self.current_config = new_config;
        }
    }

}

#[derive(Default)]
struct ToneConfiguration {
    freq_start: i32,
    freq_end: i32,
    attack_end: i32,
    decay_end: i32,
    sustain_end: i32,
    release_end: i32,
    peak_volume: i32,
    sustain_volume: i32,
    pan: Pan,
    mode: Mode,
}

impl ToneConfiguration {
    fn from_tone_spec(value: &ToneSpec, channel: &AudioChannel) -> Self {
        let volume_bytes = bytemuck::bytes_of(&value.volume);
        let pan = Pan::from_tone_flags(value.flags);
        let mode = Mode::from_tone_flags(value.flags);

        // calculate at frame offsets for ADSR boundaries 
        let duration_bytes = bytemuck::bytes_of(&value.duration);
        let attack_end_frame =  0                 + duration_bytes[3] as FrameCount;
        let decay_end_frame =   attack_end_frame  + duration_bytes[2] as FrameCount; 
        let sustain_end_frame = decay_end_frame   + duration_bytes[0] as FrameCount;
        let release_end_frame = sustain_end_frame + duration_bytes[1] as FrameCount;

        let peak_volume    = volume_bytes[0] as i32 * channel.generator.max_volume() / MAX_VOLUME as i32;
        let sustain_volume = volume_bytes[1] as i32 * channel.generator.max_volume() / MAX_VOLUME as i32;

        Self {
            freq_start: (0x0000ffff & value.frequency) as i32,
            freq_end: ((0xffff0000 & value.frequency) >> 16) as i32,
            attack_end:  Self::to_samples(attack_end_frame, channel),
            decay_end:   Self::to_samples(decay_end_frame, channel),
            sustain_end: Self::to_samples(sustain_end_frame, channel),
            release_end: Self::to_samples(release_end_frame, channel),
            sustain_volume,
            peak_volume,
            pan,
            mode,
        }
    }

    fn to_samples(frames: FrameCount, channel: &AudioChannel) -> i32 {
        frames as i32 * channel.sample_rate / TARGET_FPS as i32
    }

    fn frequency_at(&self, sample_count: i32) -> i32 {
        if self.freq_end == 0 || self.freq_end == self.freq_start {
            self.freq_start
        } else {
            lerp(self.freq_start, self.freq_end, sample_count, self.release_end)
        }
    }

    fn volume_at(&self, sample_count: i32) -> i32 {
        if sample_count < self.attack_end {
            lerp(0, self.peak_volume, sample_count, self.attack_end)
        } else if sample_count < self.decay_end {
            let x = sample_count-self.attack_end;
            let x_max = self.decay_end-self.attack_end;
            lerp(self.peak_volume, self.sustain_volume, x, x_max)
        } else if sample_count < self.sustain_end {
            self.sustain_volume
        } else if sample_count < self.release_end {
            let x = sample_count-self.sustain_end;
            let x_max = self.release_end-self.sustain_end;
            lerp(self.peak_volume, self.sustain_volume, x, x_max)
        } else {
            0
        }
    }

}

#[derive(Clone)]
enum AudioGenerator {
    Pulse,
    Triangle,
    Noise
}

fn lerp<N: Num+Copy>(y0: N, y1: N, x: N, x_max: N) -> N {
    y0 + (y1-y0) * x / x_max
}

impl AudioGenerator {
    fn render_sample(&self, channel: &mut AudioChannel) -> Sample {
        match self {
            Self::Triangle 
            | _ => Self::render_triangle_sample(channel),
        }
    }

    /// Renders a sample
    fn render_triangle_sample(channel: &mut AudioChannel) -> Sample {

        // the triangle wave is split into four sections:
        // 0: rise 0..vol
        // 1: fall vol..0
        // 2: fall 0..-vol
        // 3: rise -vol..0
        //
        // the following cacluates in which of the four
        // segments we currently are, from the current phase
        // value. It takes into account channel.phase runs
        // from 0..channel.sample_rate.
        // 
        let mut segment = 0;
        let mut f = 4 * channel.phase;
        let s4 = channel.sample_rate;
        while f > s4 {
            f -= s4;
            segment += 1;
        }

        // depeding on the segment, caculate the current value via
        // lerp
        let vol = channel.current_volume;
        match segment {
            0 => lerp(0, channel.current_volume, f, s4),
            1 => lerp(channel.current_volume, 0, f, s4),
            2 => lerp(0, -channel.current_volume, f, s4),
            _ => lerp(-channel.current_volume, 0, f, s4)
        }
    }

    fn max_volume(&self) -> i32 {
        if let Self::Triangle = self {
            0x1333
        } else {
            0x2000
        }
    }
    
}


impl Default for AudioGenerator {
    fn default() -> Self {
        Self::Pulse
    }
}

