use std::time::Duration;

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, Palette, PixelFormatEnum},
    rect::Rect,
    surface::Surface,
};

use crate::{
    wasm4::{FRAMEBUFFER_SIZE, SCREEN_SIZE},
    Renderer,
};

pub struct Sdl2Renderer {}

fn expand_fb_to_index8(fbtexdata: &mut [u8]) {
    assert!(fbtexdata.len() % 4 == 0);

    for n in (0..fbtexdata.len() / 4).rev() {
        let buf = fbtexdata[n];
        let m = 4 * n + 3;
        fbtexdata[m] = buf >> 6;
        let m = m - 1;
        fbtexdata[m] = (buf >> 4) & 0b00000011;
        let m = m - 1;
        fbtexdata[m] = (buf >> 2) & 0b00000011;
        let m = m - 1;
        fbtexdata[m] = buf & 0b00000011;
    }
}

#[test]
fn test_expand_fb_to_index8() {
    let mut testfb = [0b11100100, 0b01100011, 0, 0, 0, 0, 0, 0];
    expand_fb_to_index8(&mut testfb);

    assert!(testfb == [0b00, 0b01, 0b10, 0b11, 0b11, 0b00, 0b10, 0b01]);
}

impl Renderer for Sdl2Renderer {
    fn present(self, mut backend: impl crate::Backend + 'static) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("WASM Station", 3 * SCREEN_SIZE, 3 * SCREEN_SIZE)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        let mut surface = Surface::new(SCREEN_SIZE, SCREEN_SIZE, PixelFormatEnum::Index8).unwrap();
        let tc = canvas.texture_creator();

        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();
        canvas.present();
        let mut event_pump = sdl_context.event_pump().unwrap();

        let mut raw_colors = [0xffu8; 4 * 4];
        let mut colors = Vec::with_capacity(256);
        colors.resize(256, Color::RGB(0, 0, 0));

        'running: loop {
            backend.call_update();

            // read palette for this frame
            for c in 0..4 {
                colors[c] = Color::RGB(
                    raw_colors[3 * c],
                    raw_colors[3 * c + 1],
                    raw_colors[3 * c + 2],
                )
            }

            let fbdata = surface.without_lock_mut().unwrap();
            let fbdata: &mut [u8; FRAMEBUFFER_SIZE] =
                (&mut fbdata[0..FRAMEBUFFER_SIZE]).try_into().unwrap();
            backend.read_screen(fbdata, &mut raw_colors);
            expand_fb_to_index8(fbdata);

            let palette = Palette::with_colors(&colors).unwrap();
            surface.set_palette(&palette).unwrap();

            let fb_tex = tc.create_texture_from_surface(&surface).unwrap();
            canvas
                .copy(
                    &fb_tex,
                    None,
                    Some(Rect::new(0, 0, 3 * SCREEN_SIZE, 3 * SCREEN_SIZE)),
                )
                .unwrap();

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => (),
                }
            }
            // The rest of the game loop goes here...

            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}
