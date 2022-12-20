use std::{
    thread,
    time::{Duration, Instant},
};

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use embedded_graphics_simulator::{
    sdl2::{Keycode, MouseButton},
    OutputSettings, SimulatorDisplay, SimulatorEvent, Window,
};

use crate::{
    utils,
    wasm4::{
        BUTTON_1, BUTTON_2, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT, BUTTON_UP, FRAMEBUFFER_SIZE,
        MOUSE_LEFT, MOUSE_MIDDLE, MOUSE_RIGHT, SCREEN_SIZE,
    },
    Renderer,
};

use super::embedded;

const TARGET_MS_PER_FRAME: Duration = Duration::from_millis((1000.0 / 60.0) as u64);

pub struct EmbeddedRendererSimulator {
    pub display_scale: u32,
    pub title: String,
}

impl Default for EmbeddedRendererSimulator {
    fn default() -> Self {
        Self {
            display_scale: 3,
            title: "wasmstation - embedded".to_string(),
        }
    }
}

impl Renderer for EmbeddedRendererSimulator {
    fn present(self, mut backend: impl crate::Backend + 'static) {
        let mut display: SimulatorDisplay<Rgb888> =
            SimulatorDisplay::new(Size::new(SCREEN_SIZE, SCREEN_SIZE));
        let mut window = Window::new(
            &self.title,
            &OutputSettings {
                scale: self.display_scale,
                ..Default::default()
            },
        );

        backend.call_start();

        let mut mouse: (i16, i16) = (0, 0);
        let mut mouse_buttons: u8 = 0;
        let mut gamepad1: u8 = 0;

        let mut framebuffer: [u8; FRAMEBUFFER_SIZE] = utils::empty_framebuffer();
        let mut palette: [u8; 16] = utils::default_palette();

        'running: loop {
            let start = Instant::now();

            window.update(&display);

            for event in window.events() {
                if handle_input(event, &mut gamepad1, &mut mouse, &mut mouse_buttons) {
                    break 'running;
                }
            }

            backend.set_gamepad(bytemuck::cast([gamepad1, 0, 0, 0]));
            backend.set_mouse(mouse.0, mouse.1, mouse_buttons);

            backend.call_update();
            backend.read_screen(&mut framebuffer, &mut palette);

            embedded::draw(&mut display, &framebuffer, &palette, false);
            thread::sleep(start + TARGET_MS_PER_FRAME - Instant::now());
        }
    }
}

enum EmbeddedEvent {
    Key {
        down: bool,
        keycode: Keycode,
    },
    Mouse {
        buttons: Option<MouseButton>,
        down: Option<bool>,
        location: Point,
    },
}

impl EmbeddedEvent {
    pub fn from_simulator_event(event: SimulatorEvent) -> (Option<Self>, bool) {
        match event {
            SimulatorEvent::Quit => return (None, true),
            SimulatorEvent::KeyDown { keycode, .. } => (
                Some(EmbeddedEvent::Key {
                    down: true,
                    keycode,
                }),
                false,
            ),
            SimulatorEvent::KeyUp { keycode, .. } => (
                Some(EmbeddedEvent::Key {
                    down: false,
                    keycode,
                }),
                false,
            ),
            SimulatorEvent::MouseButtonDown { mouse_btn, point } => (
                Some(EmbeddedEvent::Mouse {
                    buttons: Some(mouse_btn),
                    down: Some(true),
                    location: point,
                }),
                false,
            ),
            SimulatorEvent::MouseButtonUp { mouse_btn, point } => (
                Some(EmbeddedEvent::Mouse {
                    buttons: Some(mouse_btn),
                    down: Some(false),
                    location: point,
                }),
                false,
            ),
            SimulatorEvent::MouseMove { point } => (
                Some(EmbeddedEvent::Mouse {
                    buttons: None,
                    down: None,
                    location: point,
                }),
                false,
            ),
            _ => (None, false),
        }
    }
}

fn handle_input(
    event: SimulatorEvent,
    gamepad1: &mut u8,
    mouse: &mut (i16, i16),
    mouse_buttons: &mut u8,
) -> bool {
    let event = match EmbeddedEvent::from_simulator_event(event) {
        (Some(event), false) => event,
        (None, true) => return true,
        _ => return false,
    };

    match event {
        EmbeddedEvent::Key { down, keycode } => {
            let mask: u8 = match keycode {
                Keycode::Left => BUTTON_LEFT,
                Keycode::Right => BUTTON_RIGHT,
                Keycode::Up => BUTTON_UP,
                Keycode::Down => BUTTON_DOWN,
                Keycode::X => BUTTON_1,
                Keycode::Z => BUTTON_2,
                _ => 0x0,
            };

            match down {
                true => *gamepad1 |= mask,
                false => *gamepad1 ^= mask,
            }
        }
        EmbeddedEvent::Mouse {
            buttons,
            down,
            location,
        } => {
            if let Some(buttons) = buttons {
                let mask: u8 = match buttons {
                    MouseButton::Left => MOUSE_LEFT,
                    MouseButton::Middle => MOUSE_MIDDLE,
                    MouseButton::Right => MOUSE_RIGHT,
                    _ => 0x0,
                };

                match down.unwrap_or(false) {
                    true => *mouse_buttons |= mask,
                    false => *mouse_buttons ^= mask,
                };
            }

            *mouse = (location.x as i16, location.y as i16);
        }
    }

    false
}
