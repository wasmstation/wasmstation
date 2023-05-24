//! Framebuffer utilities.

use crate::core::{wasm4::SCREEN_SIZE, Sink, Source};

#[cfg(test)]
mod tests;

mod blit;
mod line;
mod oval;
mod rect;
mod text;

pub use blit::{blit_sub, pixel_width_of_flags};
pub use line::{hline, line, vline};
pub use oval::oval;
pub use rect::rect;
pub use text::text;

use line::{hline_impl, vline_impl};

const DRAW_COLOR_1: u8 = 0;
const DRAW_COLOR_2: u8 = 1;

/// A common trait for index-based framebuffers.
pub(crate) trait Screen {
    type Framebuffer: Source<u8> + Sink<u8>;
    const WIDTH: u32;
    const HEIGHT: u32;
    fn fb(&self) -> &Self::Framebuffer;
    fn fb_mut(&mut self) -> &mut Self::Framebuffer;
}

/// The WASM-4 specific implementation of [`Screen`].
struct Wasm4Screen<'a, B: Sink<u8> + Source<u8>> {
    fb: &'a mut B,
}

impl<'a, B: Sink<u8> + Source<u8>> Screen for Wasm4Screen<'a, B> {
    type Framebuffer = B;
    const WIDTH: u32 = SCREEN_SIZE;
    const HEIGHT: u32 = SCREEN_SIZE;

    fn fb(&self) -> &Self::Framebuffer {
        self.fb
    }

    fn fb_mut(&mut self) -> &mut Self::Framebuffer {
        self.fb
    }
}

/// Set a pixel on the screen as a color in the palette.
pub fn set_pixel<T: Source<u8> + Sink<u8>>(fb: &mut T, x: i32, y: i32, color: u8) {
    let mut screen = Wasm4Screen { fb };
    set_pixel_impl(&mut screen, x, y, color)
}

pub(crate) fn set_pixel_impl<S: Screen>(s: &mut S, x: i32, y: i32, color: u8) {
    let idx: usize = (S::WIDTH as usize * y as usize + x as usize) >> 2;
    let shift = (x & 0x3) << 1;
    let mask = 0x3 << shift;

    let fb_byte = s.fb().item_at(idx).unwrap();
    s.fb_mut()
        .set_item_at(idx, (color << shift) | (fb_byte & !mask));
}

/// Set a pixel on the screen as a color in the palette. (clipping if out of bounds)
pub fn set_pixel_unclipped<T: Source<u8> + Sink<u8>>(fb: &mut T, x: i32, y: i32, color: u8) {
    let mut screen = Wasm4Screen { fb };
    set_pixel_unclipped_impl(&mut screen, x, y, color)
}

pub(crate) fn set_pixel_unclipped_impl<S: Screen>(s: &mut S, x: i32, y: i32, color: u8) {
    if x >= 0 && x < S::WIDTH as i32 && y >= 0 && y < S::HEIGHT as i32 {
        set_pixel_impl(s, x, y, color);
    }
}

/// Clears an entire framebuffer.
pub fn clear<T: Sink<u8>>(fb: &mut T) {
    fb.fill(0u8);
}

/// Returns a Some<u8> palette address if the draw color at the index is opaque,
/// and None if transparent.
fn remap_draw_color(draw_color_idx: u8, draw_colors: u16) -> Option<u8> {
    let draw_color = (draw_colors as u32 >> (draw_color_idx * 4)) & 0b111;
    if draw_color == 0 {
        None
    } else {
        Some((draw_color - 1) as u8)
    }
}
