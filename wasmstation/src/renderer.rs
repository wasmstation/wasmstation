//! Utilities for Rendering Games

use std::{
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

use anyhow::anyhow;
use log::{debug, error, warn};
use palette::Srgb;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    mouse::MouseButton,
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    render::Canvas,
    video::Window,
    EventPump,
};

use crate::{
    disk, utils,
    wasm4::{
        BUTTON_1, BUTTON_2, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT, BUTTON_UP, FRAMEBUFFER_SIZE,
        MOUSE_LEFT, MOUSE_MIDDLE, MOUSE_RIGHT, SCREEN_SIZE,
    },
    Backend,
};

const TARGET_FPS: f32 = 60.0;
const TARGET_MS_PER_FRAME: Duration = Duration::from_millis((1000.0 / TARGET_FPS) as u64);

const SCREEN_LENGTH: usize = (SCREEN_SIZE * SCREEN_SIZE) as usize;
const TEXTURE_LENGTH: usize = SCREEN_LENGTH * 3;

pub struct LaunchConfig {
    pub disk_write: Box<dyn Fn([u8; 1024]) -> Result<(), String>>,
    pub disk_read: Box<dyn Fn() -> Result<[u8; 1024], String>>,
    pub display_scale: u32,
    pub title: String,
}

impl Default for LaunchConfig {
    fn default() -> Self {
        Self {
            disk_write: Box::new(|_| {
                warn!("no target set for save file");
                Ok(())
            }),
            disk_read: Box::new(|| {
                warn!("no target set for save file");
                Ok((0..1024)
                    .into_iter()
                    .map(|_| 0)
                    .collect::<Vec<u8>>()
                    .try_into()
                    .unwrap())
            }),
            display_scale: 3,
            title: "wasmstation".to_string(),
        }
    }
}

impl LaunchConfig {
    pub fn from_savefile(savefile: PathBuf, display_scale: u32, title: &str) -> Self {
        Self {
            disk_write: disk::write(savefile.clone()),
            disk_read: disk::read(savefile),
            display_scale,
            title: title.to_string(),
        }
    }
}

/// Launch a game in a SDL2 window.
pub fn launch(mut backend: impl Backend, config: LaunchConfig) -> anyhow::Result<()> {
    // read from save cache on game start
    match (config.disk_read)() {
        Ok(data) => backend.set_save_cache(data),
        Err(err) => error!("{err}"),
    }

    let sdl_context = sdl2::init().map_err(|s| anyhow!("{s}"))?;
    let mut window = sdl_context
        .video()
        .map_err(|x| anyhow!("{x}"))?
        .window(
            &config.title,
            SCREEN_SIZE * config.display_scale,
            SCREEN_SIZE * config.display_scale,
        )
        .position_centered()
        .resizable()
        .build()?;
    window.set_minimum_size(SCREEN_SIZE, SCREEN_SIZE)?;

    let mut event_pump: EventPump = sdl_context.event_pump().map_err(|x| anyhow!("{x}"))?;

    let mut canvas: Canvas<Window> = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGB24,
        SCREEN_SIZE,
        SCREEN_SIZE,
    )?;

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    backend.call_start();

    let mut mouse: (i16, i16) = (0, 0);
    let mut mouse_buttons: u8 = 0;
    let mut gamepads: u32 = 0;

    let mut framebuffer: [u8; FRAMEBUFFER_SIZE] = utils::default_framebuffer();
    let mut palette: [u8; 16] = utils::default_palette();

    'running: loop {
        let start = Instant::now();

        // update input
        for event in event_pump.poll_iter() {
            if handle_input(
                event,
                &mut gamepads,
                &mut mouse,
                &mut mouse_buttons,
                canvas.window().size(),
            ) {
                break 'running;
            }
        }
        backend.set_gamepad(gamepads);
        backend.set_mouse(mouse.0, mouse.1, mouse_buttons);

        // update state
        backend.call_update();

        // write to save file on request from backend.
        if let Some(data) = backend.write_save_cache() {
            if let Err(err) = (config.disk_write)(data) {
                error!("{err}");
            };
        }

        // update screen
        backend.read_screen(&mut framebuffer, &mut palette);

        canvas.clear();
        texture.update(
            None,
            &framebuffer_to_rgb24(&framebuffer, &palette),
            SCREEN_SIZE as usize * 3,
        )?;
        canvas
            .copy(&texture, None, bounding_rect(&canvas.viewport()))
            .map_err(|s| anyhow!("{s}"))?;
        canvas.present();

        thread::sleep(start + TARGET_MS_PER_FRAME - Instant::now());
        debug!(
            "game loop took {} ms",
            Instant::now().saturating_duration_since(start).as_millis()
        );
    }

    Ok(())
}

fn framebuffer_to_rgb24(
    framebuffer: &[u8; FRAMEBUFFER_SIZE],
    palette: &[u8; 16],
) -> [u8; TEXTURE_LENGTH] {
    let palette_srgb: [Srgb<u8>; 4] = [
        Srgb::new(palette[2], palette[1], palette[0]),
        Srgb::new(palette[6], palette[5], palette[4]),
        Srgb::new(palette[10], palette[9], palette[8]),
        Srgb::new(palette[14], palette[13], palette[12]),
    ];

    let mut result = vec![0; TEXTURE_LENGTH];

    for idx in 0..SCREEN_LENGTH {
        let color = palette_srgb[((framebuffer[idx / 4] >> ((idx % 4) * 2)) & 0x3) as usize];

        let tex_idx: usize = idx * 3;

        result[tex_idx] = color.red;
        result[tex_idx + 1] = color.green;
        result[tex_idx + 2] = color.blue;
    }

    result.try_into().unwrap()
}

fn bounding_rect(size: &Rect) -> Rect {
    let game_size = size.width().min(size.height());

    Rect::new(
        ((size.width() - game_size) / 2) as i32,
        ((size.height() - game_size) / 2) as i32,
        game_size,
        game_size,
    )
}

enum DesktopInputEvent {
    Key {
        down: bool,
        keycode: Option<Keycode>,
    },
    Mouse {
        buttons: Option<MouseButton>,
        down: Option<bool>,
        location: (i32, i32),
    },
}

fn handle_input(
    event: Event,
    gamepads: &mut u32,
    mouse: &mut (i16, i16),
    mouse_buttons: &mut u8,
    window_size: (u32, u32),
) -> bool {
    let event = match event {
        Event::Quit { .. } => return true,
        Event::KeyDown { keycode, .. } => DesktopInputEvent::Key {
            down: true,
            keycode,
        },
        Event::KeyUp { keycode, .. } => DesktopInputEvent::Key {
            down: false,
            keycode,
        },
        Event::MouseButtonDown {
            mouse_btn, x, y, ..
        } => DesktopInputEvent::Mouse {
            buttons: Some(mouse_btn),
            down: Some(true),
            location: (x, y),
        },
        Event::MouseButtonUp {
            mouse_btn, x, y, ..
        } => DesktopInputEvent::Mouse {
            buttons: Some(mouse_btn),
            down: Some(false),
            location: (x, y),
        },
        Event::MouseMotion { x, y, .. } => DesktopInputEvent::Mouse {
            buttons: None,
            down: None,
            location: (x, y),
        },
        _ => return false,
    };

    let [gamepad1, _, _, _] = bytemuck::cast_mut::<u32, [u8; 4]>(gamepads);

    match event {
        DesktopInputEvent::Key { down, keycode } => {
            let mask: u8 = match keycode {
                Some(Keycode::Left) => BUTTON_LEFT,
                Some(Keycode::Right) => BUTTON_RIGHT,
                Some(Keycode::Up) => BUTTON_UP,
                Some(Keycode::Down) => BUTTON_DOWN,
                Some(Keycode::X) => BUTTON_1,
                Some(Keycode::Z) => BUTTON_2,
                _ => 0x0,
            };

            match down {
                true => *gamepad1 |= mask,
                false => *gamepad1 ^= mask,
            }
        }
        DesktopInputEvent::Mouse {
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

            let location: (u32, u32) = (location.0 as u32, location.1 as u32);

            let game_size = window_size.0.min(window_size.1);
            let border_x = (window_size.0 - game_size) / 2;
            let border_y = (window_size.1 - game_size) / 2;

            if location.0 >= border_x
                && location.1 >= border_y
                && location.0 <= (window_size.0 - border_x)
                && location.1 <= (window_size.1 - border_y)
            {
                *mouse = (
                    (((location.0 - border_x) as f32 / game_size as f32) * SCREEN_SIZE as f32)
                        as i16,
                    (((location.1 - border_y) as f32 / game_size as f32) * SCREEN_SIZE as f32)
                        as i16,
                );
            }
        }
    }

    false
}
