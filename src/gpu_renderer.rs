//! A GPU renderer using [`pixels`] and [`winit`].

use crate::core::{utils, wasm4, Backend};
use pixels::{Pixels, SurfaceTexture};
use pollster::FutureExt;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

/// Launch a game window in a desktop window.
#[cfg(not(target_arch = "wasm32"))]
pub fn launch_desktop(
    backend: impl Backend + 'static,
    title: &str,
    display_scale: u32,
) -> anyhow::Result<()> {
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(
            wasm4::SCREEN_SIZE as f64 * display_scale as f64,
            wasm4::SCREEN_SIZE as f64 * display_scale as f64,
        );
        WindowBuilder::new()
            .with_title(title)
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)?
    };

    launch(backend, display_scale, window, event_loop)
}

/// Launch a game window on a web canvas.
#[cfg(any(doc, target_arch = "wasm32"))]
pub fn launch_web(
    backend: impl Backend + 'static,
    canvas_id: &str,
    display_scale: u32,
) -> anyhow::Result<()> {
    #[cfg(target_arch = "wasm32")]
    use {wasm_bindgen::JsCast, winit::platform::web::WindowBuilderExtWebSys};

    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(
            wasm4::SCREEN_SIZE as f64 * display_scale as f64,
            wasm4::SCREEN_SIZE as f64 * display_scale as f64,
        );
        WindowBuilder::new()
            .with_title(canvas_id)
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_canvas(
                web_sys::window()
                    .and_then(|window| window.document())
                    .and_then(|document| document.get_element_by_id(canvas_id))
                    .and_then(|elem| elem.dyn_into().map_or(None, Some)),
            )
            .build(&event_loop)?
    };

    launch(backend, display_scale, window, event_loop)
}

/// Launch a [`winit`]/[`pixels`] window with a custom [`Window`](winit::window::Window) and [`EventLoop`](winit::event_loop::EventLoop).
pub fn launch<T>(
    mut backend: impl Backend + 'static,
    display_scale: u32,
    window: Window,
    event_loop: EventLoop<T>,
) -> anyhow::Result<()> {
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new_async(wasm4::SCREEN_SIZE, wasm4::SCREEN_SIZE, surface_texture).block_on()?
    };

    let mut mouse: (i16, i16) = (0, 0);
    let mut mouse_buttons: u8 = 0;
    let mut gamepads: u32 = 0;

    let mut framebuffer: [u8; wasm4::FRAMEBUFFER_SIZE] = utils::default_framebuffer();
    let mut palette: [u8; 16] = utils::default_palette();

    backend.call_start();

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::RedrawRequested(_) => {
                backend.set_gamepad(gamepads);
                backend.set_mouse(mouse.0, mouse.1, mouse_buttons);

                backend.call_update();

                backend.read_screen(&mut framebuffer, &mut palette);
                read_framebuffer(pixels.frame_mut(), &mut framebuffer, &palette);

                if let Err(err) = pixels.render() {
                    log::error!("pixels render error: {err}");
                    control_flow.set_exit();
                    return;
                }
            }
            Event::WindowEvent {
                event: window_event,
                ..
            } => match window_event {
                WindowEvent::CloseRequested => {
                    control_flow.set_exit();
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    let [gamepad1, gamepad2, gamepad3, _] =
                        bytemuck::cast_mut::<u32, [u8; 4]>(&mut gamepads);

                    let (mask, gamepad): (u8, &mut u8) = match input.virtual_keycode {
                        // Player 1
                        Some(VirtualKeyCode::X) => (wasm4::BUTTON_1, gamepad1),
                        Some(VirtualKeyCode::Y) => (wasm4::BUTTON_2, gamepad1),
                        Some(VirtualKeyCode::Up) => (wasm4::BUTTON_UP, gamepad1),
                        Some(VirtualKeyCode::Down) => (wasm4::BUTTON_DOWN, gamepad1),
                        Some(VirtualKeyCode::Left) => (wasm4::BUTTON_LEFT, gamepad1),
                        Some(VirtualKeyCode::Right) => (wasm4::BUTTON_RIGHT, gamepad1),
                        // Player 2
                        Some(VirtualKeyCode::A) | Some(VirtualKeyCode::Q) => {
                            (wasm4::BUTTON_1, gamepad2)
                        }
                        Some(VirtualKeyCode::Tab) => (wasm4::BUTTON_2, gamepad2),
                        Some(VirtualKeyCode::E) => (wasm4::BUTTON_UP, gamepad2),
                        Some(VirtualKeyCode::D) => (wasm4::BUTTON_DOWN, gamepad2),
                        Some(VirtualKeyCode::S) => (wasm4::BUTTON_LEFT, gamepad2),
                        Some(VirtualKeyCode::F) => (wasm4::BUTTON_RIGHT, gamepad2),
                        // Player 3
                        Some(VirtualKeyCode::NumpadMultiply)
                        | Some(VirtualKeyCode::NumpadDecimal) => (wasm4::BUTTON_1, gamepad3),
                        Some(VirtualKeyCode::NumpadSubtract)
                        | Some(VirtualKeyCode::NumpadEnter) => (wasm4::BUTTON_2, gamepad3),
                        Some(VirtualKeyCode::Numpad8) => (wasm4::BUTTON_UP, gamepad3),
                        Some(VirtualKeyCode::Numpad5) => (wasm4::BUTTON_DOWN, gamepad3),
                        Some(VirtualKeyCode::Numpad4) => (wasm4::BUTTON_LEFT, gamepad3),
                        Some(VirtualKeyCode::Numpad6) => (wasm4::BUTTON_RIGHT, gamepad3),

                        _ => return,
                    };

                    match input.state {
                        ElementState::Pressed => *gamepad |= mask,
                        ElementState::Released => *gamepad ^= mask,
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    mouse = (
                        (position.x / display_scale as f64) as i16,
                        (position.y / display_scale as f64) as i16,
                    );
                }
                WindowEvent::Resized(size) => {
                    if let Err(err) = pixels.resize_surface(size.width, size.height) {
                        log::error!("pixels resize: {err}");
                        control_flow.set_exit();
                        return;
                    }
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    let mask = match button {
                        MouseButton::Left => wasm4::MOUSE_LEFT,
                        MouseButton::Middle => wasm4::MOUSE_MIDDLE,
                        MouseButton::Right => wasm4::MOUSE_RIGHT,
                        _ => return,
                    };

                    match state {
                        ElementState::Pressed => mouse_buttons |= mask,
                        ElementState::Released => mouse_buttons ^= mask,
                    }
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}

#[inline(always)]
fn read_framebuffer(
    pixels: &mut [u8],
    framebuffer: &mut [u8; wasm4::FRAMEBUFFER_SIZE],
    palette: &[u8; 16],
) {
    let palette: [[u8; 3]; 4] = [
        [palette[2], palette[1], palette[0]],
        [palette[6], palette[5], palette[4]],
        [palette[10], palette[9], palette[8]],
        [palette[14], palette[13], palette[12]],
    ];

    for (idx, pixel) in pixels.chunks_exact_mut(4).enumerate() {
        let color = palette[((framebuffer[idx / 4] >> ((idx % 4) * 2)) & 0x3) as usize];

        pixel.copy_from_slice(&[color[0], color[1], color[2], 0xff]);
    }
}
