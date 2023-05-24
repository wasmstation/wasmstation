use crate::core::{Sink, Source};

use super::{hline_impl, remap_draw_color, vline_impl, Wasm4Screen, DRAW_COLOR_1, DRAW_COLOR_2};

/// Draw a rectangle.
pub fn rect<T: Source<u8> + Sink<u8>>(
    fb: &mut T,
    draw_colors: u16,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) {
    let mut screen = Wasm4Screen { fb };
    if let Some(fill_stroke) = remap_draw_color(DRAW_COLOR_1, draw_colors) {
        let fx = x;
        let flen = width;
        for fy in y..y + (height as i32) {
            hline_impl(&mut screen, fill_stroke, fx, fy, flen)
        }
    }
    if let Some(line_stroke) = remap_draw_color(DRAW_COLOR_2, draw_colors) {
        hline_impl(&mut screen, line_stroke, x, y, width);
        hline_impl(&mut screen, line_stroke, x, y + (height as i32) - 1, width);
        vline_impl(&mut screen, line_stroke, x, y, height);
        vline_impl(&mut screen, line_stroke, x + (width as i32) - 1, y, height);
    }
}
