// WASM-4 Framebuffer functions

use crate::{wasm4::SCREEN_SIZE, Sink, Source};
use std::{fmt::Write, mem::size_of};

mod blit;
mod line;
mod oval;
mod rect;
mod text;

pub use blit::{blit_sub, pixel_width_of_flags, PixelFormat};
pub use line::{hline, line, vline};
pub use oval::oval;
pub use rect::rect;
pub use text::text;

use line::{hline_impl, vline_impl};
use num_traits::{PrimInt, Unsigned};

const DRAW_COLOR_1: u8 = 0;
const DRAW_COLOR_2: u8 = 1;
const DRAW_COLOR_3: u8 = 2;
const DRAW_COLOR_4: u8 = 3;

pub(crate) trait Screen {
    type Framebuffer: Source<u8> + Sink<u8>;
    const WIDTH: u32;
    const HEIGHT: u32;
    fn fb(&self) -> &Self::Framebuffer;
    fn fb_mut(&mut self) -> &mut Self::Framebuffer;
}

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

#[derive(PartialEq)]
struct ArrayScreen<const N: usize, const W: u32> {
    fb: [u8; N],
}

impl<const N: usize, const W: u32> std::fmt::Debug for ArrayScreen<N, W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "ArrayScreen(WIDTH:{},HEIGHT:{}). framebuffer:\n",
            Self::WIDTH,
            Self::HEIGHT
        ))?;
        for n in 0..Self::HEIGHT as usize {
            let start = n * Self::WIDTH as usize / 4;
            let end = (n + 1) * Self::WIDTH as usize / 4;
            let line = as_fb_line(&self.fb()[start..end]);
            f.write_str(&line)?;
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl<const N: usize, const W: u32> ArrayScreen<N, W> {
    fn new() -> ArrayScreen<N, W> {
        ArrayScreen::<N, W> { fb: [0u8; N] }
    }

    fn new_with_fb_lines(fb_lines: &[Vec<u8>]) -> ArrayScreen<N, W> {
        let mut s = Self::new();
        for n in 0..fb_lines.len() {
            let line = &fb_lines[n];
            let start = n * Self::WIDTH as usize / 4;
            let end = (n + 1) * Self::WIDTH as usize / 4;
            s.fb_mut()[start..end].copy_from_slice(line.as_slice());
        }
        s
    }
}

impl<const N: usize, const W: u32> Screen for ArrayScreen<N, W> {
    type Framebuffer = [u8; N];
    const WIDTH: u32 = W;
    const HEIGHT: u32 = N as u32 * 4 / W;

    fn fb(&self) -> &Self::Framebuffer {
        &self.fb
    }

    fn fb_mut(&mut self) -> &mut Self::Framebuffer {
        &mut self.fb
    }
}

/// Set a pixel on the screen to a color in the palette.
pub fn set_pixel<T: Source<u8> + Sink<u8>>(fb: &mut T, x: i32, y: i32, color: u8) {
    let mut screen = Wasm4Screen { fb };
    set_pixel_impl(&mut screen, x, y, color)
}

pub(crate) fn set_pixel_impl<S: Screen>(s: &mut S, x: i32, y: i32, color: u8) {
    let idx: usize = (S::WIDTH as usize * y as usize + x as usize) >> 2;
    let shift = (x & 0x3) << 1;
    let mask = 0x3 << shift;

    let fb_byte = s.fb().item_at(idx);
    s.fb_mut()
        .set_item_at(idx, (color << shift) | (fb_byte & !mask));
}

pub fn set_pixel_unclipped<T: Source<u8> + Sink<u8>>(fb: &mut T, x: i32, y: i32, color: u8) {
    let mut screen = Wasm4Screen { fb };
    set_pixel_unclipped_impl(&mut screen, x, y, color)
}

pub(crate) fn set_pixel_unclipped_impl<S: Screen>(s: &mut S, x: i32, y: i32, color: u8) {
    if x >= 0 && x < S::WIDTH as i32 && y >= 0 && y < S::HEIGHT as i32 {
        set_pixel_impl(s, x, y, color);
    }
}

/// Clears the entire screen.
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

/// convert a machine word of sprite pixels to actual FB indices and a mask
/// for applying transparent pixels
fn remap_draw_colors(sprite_word: u32, sprite_word_pixels: i32, draw_colors: u16) -> (u32, u32) {
    let mut s = 0;
    let mut m = 0;
    for n in 0..sprite_word_pixels {
        let shift = 2 * n;
        let draw_color_idx = (sprite_word >> shift) & 0b11;

        if let Some(palette_index) = remap_draw_color(draw_color_idx as u8, draw_colors) {
            s |= (palette_index as u32) << shift
        } else {
            m |= 0b11 << shift;
        }
    }

    (s, m)
}

/// create a Vec<u8> of framebuffer data from pixels from an integer literal
fn as_fb_vec<T>(n: T) -> Vec<u8>
where
    T: PrimInt + Unsigned,
{
    as_pix_vec(n, PixelFormat::Framebuffer)
}

/// create a Vec<u8> of 1BPP sprite data from pixels from an integer literal
fn as_b1_vec<T>(n: T) -> Vec<u8>
where
    T: PrimInt + Unsigned,
{
    as_pix_vec(n, PixelFormat::Blit1BPP)
}

/// create a Vec<u8> of 2BPP sprite data from pixels from an integer literal
fn as_b2_vec<T>(n: T) -> Vec<u8>
where
    T: PrimInt + Unsigned,
{
    as_pix_vec(n, PixelFormat::Blit2BPP)
}

/// Convert arbitrary primitive integer types (aka u8..u128/i8..i128)
/// into a Vec<u8> for use in framebuffer related functions, allowing
/// to define sprites and framebuffer patterns inline with Rust's
/// integer (bit) literals.
/// This is needed for testing so that we can conveniently spell out bit
/// patterns in various sizes, yielding them as Vec<u8>
fn as_pix_vec<T>(n: T, fmt: PixelFormat) -> Vec<u8>
where
    T: PrimInt + Unsigned,
{
    let (pix_size, reverse_pixel_order) = match fmt {
        PixelFormat::Blit1BPP => (1, false),
        PixelFormat::Blit2BPP => (2, false),
        PixelFormat::Framebuffer => (2, true),
    };

    let mut v = Vec::with_capacity(size_of::<T>());
    let mask = T::from(0xff >> (8 - pix_size)).unwrap();
    for i in 0..size_of::<T>() {
        let mut b = 0u8;
        for j in 0..(8 / pix_size) {
            b = b << pix_size;
            let shift = if reverse_pixel_order {
                j * pix_size
            } else {
                8 - (j + 1) * pix_size
            };
            let pix = n.shr(i * 8 + shift);
            b |= pix.bitand(mask).to_u8().unwrap();
        }
        v.insert(0, b);
    }

    v
}

fn as_fb_line(v: &[u8]) -> String {
    let prefix = "0b";
    let mut s = String::with_capacity(prefix.len() + v.len() * 4 * 3);
    s += prefix;
    for e in v {
        let b = *e as u16;
        for n in 0..4 {
            s += "_";
            s += &((b >> (2 * n + 1)) & 0b1).to_string();
            s += &((b >> (2 * n + 0)) & 0b1).to_string();
        }
    }

    s
}

#[cfg(test)]
mod tests {
    use crate::console::framebuffer::{as_fb_line, as_fb_vec, remap_draw_colors};

    use super::as_b1_vec;

    #[test]
    fn test_remap_draw_colors() {
        assert_eq!(
            (0b01_00_10_00, 0b00_11_00_00),
            remap_draw_colors(0b11100001, 4, 0x2013)
        );
    }

    #[test]
    fn test_as_fb_vec() {
        // happy case, coming from u8
        assert_eq!(vec![0b_10_01_01_10_u8], as_fb_vec(0b_10_01_01_10_u8));

        // u16 into a vec of 2 x u8 - the two tests are equivalent, but
        // shows we can write it also as 0x as well as a 0b literal
        assert_eq!(
            vec![0b_11_00_01_11_u8, 0b_10_01_01_10_u8],
            as_fb_vec(0b_11_01_00_11__10_01_01_10_u16)
        );
    }

    #[test]
    fn test_as_b1_vec() {
        assert_eq!(
            vec![0b__0__0__0__0__1__1__1__0_u8],
            as_b1_vec(0b__0__0__0__0__1__1__1__0_u8)
        )
    }

    #[test]
    fn test_as_b2_vec() {
        assert_eq!(
            vec![0b_11_01_00_11_u8, 0b_10_01_01_10_u8],
            as_b1_vec(0b_11_01_00_11__10_01_01_10_u16)
        )
    }

    #[test]
    fn test_as_fb_line() {
        let fb_vec = as_fb_vec(0b_10_10_10_10_11_00_11_11_11_10_11_10_10_10_10_10__u32);
        assert_eq!(
            "0b_10_10_10_10_11_00_11_11_11_10_11_10_10_10_10_10",
            as_fb_line(&fb_vec)
        );
    }
}
