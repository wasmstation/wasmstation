use crate::core::{wasm4::FRAMEBUFFER_SIZE, Sink, Source};
use core::mem;

use super::{
    remap_draw_color, set_pixel_impl, set_pixel_unclipped_impl, Screen, Wasm4Screen, DRAW_COLOR_1,
};

/// Draw a line between two points.
///
/// See <https://github.com/aduros/wasm4/blob/main/runtimes/native/src/framebuffer.c>,
/// who in turn took it from <https://github.com/nesbox/TIC-80/blob/master/src/core/draw.c>
pub fn line<T: Source<u8> + Sink<u8>>(
    fb: &mut T,
    draw_colors: u16,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
) {
    let dc0: u8 = (draw_colors & 0xf) as u8;
    if dc0 == 0 {
        return;
    }

    let stroke_color: u8 = (dc0 - 1) & 0x3;
    line_impl(&mut Wasm4Screen { fb }, stroke_color, x1, y1, x2, y2);
}

pub(crate) fn line_impl<T: Screen>(
    screen: &mut T,
    stroke_color: u8,
    mut x1: i32,
    mut y1: i32,
    mut x2: i32,
    mut y2: i32,
) {
    if y1 > y2 {
        mem::swap(&mut x1, &mut x2);
        mem::swap(&mut y1, &mut y2);
    }

    let dx = (x2 - x1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let dy = y2 - y1;

    let mut err = (if dx > dy { dx } else { -dy }) / 2;

    // we won't have to ever go through the entirety of FRAMEBUFFER_SIZE,
    // I just added this so the loop will stop incase something goes really wrong.
    for _ in 0..FRAMEBUFFER_SIZE {
        set_pixel_unclipped_impl(screen, x1, y1, stroke_color);

        if x1 == x2 && y1 == y2 {
            break;
        }

        let err2 = err;

        if err2 > -dx {
            err -= dy;
            x1 += sx;
        }

        if err2 < dy {
            err += dx;
            y1 += 1;
        }
    }
}

/// Draw a horizontal line.
///
/// Specifically, between `(x, y)` and `(x + len - 1, y)`.
pub fn hline<T: Source<u8> + Sink<u8>>(fb: &mut T, draw_colors: u16, x: i32, y: i32, len: u32) {
    if let Some(stroke) = remap_draw_color(DRAW_COLOR_1, draw_colors) {
        hline_impl(&mut Wasm4Screen { fb }, stroke, x, y, len);
    }
}

pub(crate) fn hline_impl<T: Screen>(screen: &mut T, stroke: u8, x: i32, y: i32, len: u32) {
    if y < 0 || y >= T::HEIGHT as i32 {
        return;
    }

    let mut start_x = x.max(0);
    let end_x = (len as i32 + x).min(T::WIDTH as i32);

    if start_x > end_x {
        return;
    }

    let fill_end = end_x - (end_x & 3);
    let fill_start = fill_end.min((start_x + 3) & !3);

    if fill_end - fill_start > 3 {
        for x in start_x..fill_start {
            set_pixel_impl(screen, x, y, stroke);
        }

        let from = ((T::WIDTH as i32 * y + fill_start) >> 2) as usize;
        let to = ((T::WIDTH as i32 * y + fill_end) >> 2) as usize;
        let byte_stroke = stroke * 0x55;

        for idx in from..to {
            screen.fb_mut().set_item_at(idx, byte_stroke);
        }
        start_x = fill_end;
    }

    for x in start_x..end_x {
        set_pixel_impl(screen, x, y, stroke);
    }
}

/// Draw a vertical line.
pub fn vline<T: Source<u8> + Sink<u8>>(fb: &mut T, draw_colors: u16, x: i32, y: i32, len: u32) {
    if let Some(stroke) = remap_draw_color(DRAW_COLOR_1, draw_colors) {
        vline_impl(&mut Wasm4Screen { fb }, stroke, x, y, len);
    }
}

pub(crate) fn vline_impl<T: Screen>(screen: &mut T, stroke: u8, x: i32, y: i32, len: u32) {
    if y + len as i32 <= 0 || x < 0 || x >= T::WIDTH as i32 {
        return;
    }

    let start_y: i32 = y.max(0);
    let end_y: i32 = (len as i32 + y).min(T::HEIGHT as i32);

    if start_y > end_y {
        return;
    }

    for y in start_y..end_y {
        set_pixel_impl(screen, x, y, stroke);
    }
}