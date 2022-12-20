use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use palette::Srgb;

use crate::wasm4::{FRAMEBUFFER_SIZE, SCREEN_SIZE};

/// draw wasm4 framebuffer to any embedded-graphics compatible DrawTargets.
pub fn draw<T: DrawTarget<Color = Rgb888>>(
    target: &mut T,
    framebuffer: &[u8; FRAMEBUFFER_SIZE],
    palette: &[u8; 16],
    adaptive_scaling: bool, // turn this on for resolutions other than 160x160.
) {
    let palette_srgb: [Srgb<u8>; 4] = palette
        .chunks_exact(4)
        .map(|x| Srgb::new(x[2], x[1], x[0]))
        .collect::<Vec<Srgb<u8>>>()
        .try_into()
        .unwrap();

    let target_width = target.bounding_box().size.width;
    let target_height = target.bounding_box().size.height;

    let x_scale = target_width as f32 / SCREEN_SIZE as f32;
    let y_scale = target_width as f32 / SCREEN_SIZE as f32;

    if let Err(_) = target.fill_contiguous(
        &target.bounding_box(),
        (0..(target_width * target_height)).map(|mut idx| {
            if adaptive_scaling {
                idx = (((idx / target_width) as f32 / y_scale) as u32 * SCREEN_SIZE)
                    + ((idx % target_width) as f32 / x_scale) as u32;
            }

            let color =
                palette_srgb[((framebuffer[(idx / 4) as usize] >> ((idx % 4) * 2)) & 0x3) as usize];

            Rgb888::new(color.red, color.green, color.blue)
        }),
    ) {
        eprintln!("error drawing framebuffer");
    };
}
