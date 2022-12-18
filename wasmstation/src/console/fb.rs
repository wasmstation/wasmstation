// WASM-4 Framebuffer functions

use std::{ops::Range, mem::size_of};

use num_traits::{PrimInt, Unsigned};

use winit::dpi::Pixel;

use crate::{
    wasm4::{BLIT_2BPP, SCREEN_SIZE, BLIT_FLIP_X, BLIT_ROTATE, BLIT_FLIP_Y},
    Sink, Source,
};

impl<T> Sink<T> for Vec<T>
where
    T: Copy,
{
    fn set_item_at(&mut self, offset: usize, item: T) {
        self[offset] = item
    }
}

impl<T> Source<T> for Vec<T>
where
    T: Copy,
{
    fn item_at(&self, offset: usize) -> T {
        self[offset]
    }
}

#[derive(Clone, Copy)]
enum PixelFormat {
    Blit1BPP,
    Blit2BPP,
}
impl PixelFormat {
    fn from_blit_flags(flags: u32) -> Self {
        if flags & (BLIT_2BPP as u32) != 0 {
            Self::Blit2BPP
        } else {
            Self::Blit1BPP
        }
    }
}
fn get_sprite_pixel_draw_color<T: Source<u8>>(sprite: &T, fmt: PixelFormat, x: i32, y: i32, width: i32) -> u8 {
    let pixel_index = width * y + x;
    match fmt {
        PixelFormat::Blit1BPP => {
            let mut byte = sprite.item_at((pixel_index >> 3) as usize);
            byte = byte >> (pixel_index & 0x07);
            byte & 0x01
        },
        PixelFormat::Blit2BPP => {
            let mut byte = sprite.item_at((pixel_index >> 3) as usize);
            byte = byte >> (pixel_index & 0x03);
            byte & 0x03
        }
    }
}

fn set_pixel<T: Source<u8> + Sink<u8>>(fb: &mut T, x: i32, y: i32, color: u8) {
    let idx: usize = (SCREEN_SIZE as usize * y as usize + x as usize) >> 2;
    let shift = (x & 0x3) << 1;
    let mask = 0x3 << shift;

    fb.set_item_at(idx, (color << shift) | (fb.item_at(idx) & !mask));
}

fn set_pixel_unclipped<T: Source<u8> + Sink<u8>>(fb: &mut T, x: i32, y: i32, color: u8) {
    if x >= 0 && x < SCREEN_SIZE as i32 && y >= 0 && y < SCREEN_SIZE as i32 {
        set_pixel(fb, x, y, color);
    }
}

// clamp range opened by sprite start and end in
// a sprite atlas. This function looks at a 
// single axis
// 
//                   0                   SCREEN_SIZE
// screen            |.........................|
// offset (x/y)      |---->:
//                   :     ^this is where the sprite should to be drawn
//                   :     :
//               0   :     :  sprite          width
// sprite_atlas  |...:.....:==========:......|
//                         :          :
//              sprite_r.start        :
//                                    :
//                          sprite_r.end
//
fn clamp_range(mut sprite_r: Range<i32>, mut offset: i32) -> (i32,Range<i32>) {

    let limit = SCREEN_SIZE as i32;
    // clamp start of range
    if offset < 0 {
        sprite_r.start += -offset;
        offset = 0
    }

    let screen_end_delta = (sprite_r.start + offset) - limit;
    if screen_end_delta > 0 {
        sprite_r.end -= limit;
    }
    (offset, sprite_r)
}

pub(crate) fn pixel_width_of_flags(flags: u32) -> u32 {
    if flags & BLIT_2BPP != 0 {
        2
    } else {
        1
    }
}

fn calculate_target_range(tgt_coord: i32, tgt_extent: i32, clip_range: Range<i32>) -> Range<i32>{
    Range {
        start: i32::max(clip_range.start, tgt_coord) - tgt_coord,
        end:   i32::min(tgt_extent, clip_range.end - tgt_coord),
    }
}


#[allow(clippy::too_many_arguments)]
pub(crate) fn blit_sub<S, T>(
    target: &mut T,
    sprite: &S,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    src_x: u32,
    src_y: u32,
    stride: u32,
    flags: u32,
    draw_colors: u16,
) where
    S: Source<u8>,
    T: Source<u8> + Sink<u8>,
{
    // we probably should change the signature to avoid signedness conversions
    let src_x = src_x as i32;
    let src_y = src_y as i32;
    let width = width as i32;
    let height = height as i32;
    let stride = stride as i32;

    let flip_x = flags & BLIT_FLIP_X != 0;
    let flip_y = flags & BLIT_FLIP_Y != 0;
    let rotate = flags & BLIT_ROTATE != 0;

    let mut flip_x = flip_x;

    // for now, this is clips to the screen edge. 
    // but it doesn't need to be that way; we may clip smaller too
    let clip_range_x = 0..(SCREEN_SIZE as i32);
    let clip_range_y = 0..(SCREEN_SIZE as i32);

    // ranges within the target window, local to target
    // start coordinates x and y:
    let w_range_x;
    let w_range_y;
    if rotate {
        flip_x = !flip_x;
        w_range_x = calculate_target_range(y, height, clip_range_y);
        w_range_y = calculate_target_range(x, width, clip_range_x);
    } else {
        w_range_x = calculate_target_range(x, width, clip_range_x);
        w_range_y = calculate_target_range(x, height, clip_range_y);
    }

    let fmt = PixelFormat::from_blit_flags(flags);

    for wy in w_range_y.clone() {
        for wx in w_range_x.clone() {

            // target coordinates where the sprite pixel will be written to,
            // relative to target start coordinates x,y
            // swap wx, wy if we rotate
            let tgt_location = if rotate {
                (wy, wx)
            } else {
                (wx, wy)
            };
            let tx = x + tgt_location.0;
            let ty = y + tgt_location.1;

            // source coordinates where the sprite pixel will be read from,
            // relative to sprite start coordinates src_x, src_y
            let src_location = (
                if flip_x { width  - wx - 1 } else { wx },
                if flip_y { height - wy - 1 } else { wy }
            );
            let sx = src_x + src_location.0;
            let sy = src_y + src_location.1;

            let draw_color_idx = get_sprite_pixel_draw_color(sprite, fmt, sx, sy, stride);
            let (color, opaque) = remap_draw_color(draw_color_idx, draw_colors);
            if opaque {
                set_pixel_unclipped(target, tx, ty, color)
            }
        }
    }


}


#[allow(clippy::too_many_arguments)]
pub(crate) fn blit_sub_multipixel<S, T>(
    target: &mut T,
    sprite: &S,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    src_x: u32,
    src_y: u32,
    stride: u32,
    flags: u32,
    draw_colors: u16,
) where
    S: Source<u8>,
    T: Source<u8> + Sink<u8>,
{
    let pixel_width = pixel_width_of_flags(flags);

    let num_bits_in_sprite = stride * height * pixel_width;
    let len = (num_bits_in_sprite + 7) / 8;
    
    let src_x = src_x as i32;
    let src_y = src_y as i32;
    let (tx,sx_r) = clamp_range(src_x..(src_x+width as i32), x);
    let (ty,sy_r) = clamp_range(src_y..(src_y+height as i32), y);

    // the number of bits to left-shift pixel index to get to the sprite byte that contains it
    let index_shift = if flags & BLIT_2BPP != 0 { 2 } else { 3 };

    // calculate number bytes in sprite for a single
    // line in the sprite. Takes into account that
    // tx and width may not be divisible by 8, implying
    // that we may need an extra byte
    let target_bytes_per_line = ((((tx & 0x03) + sx_r.end - sx_r.start) << 1) + 7) / 8;

    // initialize sprite pixel buffer. This buffer u32 holds
    // the sprite bits that we'll apply next to the coming target
    // byte. Excess bits (bit 8+) are masked out when applying, and
    // are shifted down by 8 bits right after. The pixels in this 
    // buffer are always aligned with the start of a frame buffer byte
    // so that no further shifting is required when they are applied.
    //
    // pixbuf: the sprite pixel buffer
    // maskbuf: the mask to apply to the target byte to clear
    //  the pixel bits that are opaque in the sprite
    // pixbuf_len: the number of bits currently stored in pixbuf.
    //
    // initally, pixbuf is initialized to 0. However, pixbuf_len
    // may be initialized to nonzero, to accomodate offset bits
    // if we're starting to write in the middle of a frame
    // buffer byte.
    let mut pixbuf = 0u32;
    let mut maskbuf = !((!0u32) << (tx & 0x3));
    let mut pixbuf_len = ((tx & 0x3)) << (pixel_width - 1);

    let mut tgt_start_idx = (tx + ty * SCREEN_SIZE as i32) >> 2;
    for sy in sy_r.start..sy_r.end {
        // index of the first sprite byte in this line
        let sprite_line_start_idx = ((sx_r.start + sy * stride as i32) >> index_shift) as usize;
        let mut sprite_idx = sprite_line_start_idx;
        let mut sprite_pixels_left = sx_r.end - sx_r.start;

        for n in tgt_start_idx..(tgt_start_idx + target_bytes_per_line) {
            // if there's room in pixbuf, get next sprite byte
            if pixbuf_len < 8 && sprite_pixels_left > 0 {
                // load next u8 from sprite line
                let sprite_byte = sprite.item_at(sprite_idx);
                sprite_idx += 1;

                let sprite_word;
                let sprite_byte_pixel_capacity: i32;
                if flags & BLIT_2BPP == 0 {
                    sprite_word = conv_1bpp_to_2bpp(sprite_byte);
                    sprite_byte_pixel_capacity = 8;
                } else {
                    sprite_word = sprite_byte as u32;
                    sprite_byte_pixel_capacity = 4;
                }

                // remap colors via draw_colors
                let sprite_pixels;
                let (mut sprite_word, mask_word) =
                    remap_draw_colors(sprite_word, sprite_byte_pixel_capacity, draw_colors);

                if sprite_pixels_left < sprite_byte_pixel_capacity {
                    // if we're reading the right fringe of a sprite line,
                    // mask the excess bits away
                    sprite_pixels = sprite_pixels_left;
                    sprite_word &= (!0u32) >> (32 - sprite_pixels_left * 2)
                } else {
                    sprite_pixels = sprite_byte_pixel_capacity;
                }
                sprite_pixels_left -= sprite_pixels;
                let bits = sprite_pixels << 1;

                let sprite_word_shift = pixbuf_len - ((sx_r.start & 0x3)<<1);
                pixbuf |= sprite_word << sprite_word_shift;
                maskbuf |= mask_word << sprite_word_shift;
                pixbuf_len += bits as i32;
            }

            // load source byte from pixbuf
            let mask_byte = maskbuf as u8;
            let src_byte = (pixbuf as u8) & !mask_byte;
            pixbuf >>= 8;
            maskbuf >>= 8;
            pixbuf_len -= 8;

            // apply src_byte to target byte in frame buffer
            let mut tgt_byte = target.item_at(n as usize);
            tgt_byte &= mask_byte;
            tgt_byte |= src_byte;
            target.set_item_at(n as usize, tgt_byte)
        }

        // move start index one screen line below
        tgt_start_idx += SCREEN_SIZE as i32 / 4;
    }
}

#[test]
fn test_blit_sub_impl_1byte() {
    let draw_colors = 0x4320;

    // regular
    let sprite =      as_b1_vec(0b__0__0__0__0__1__1__1__0_u8);
    let mut fb =      as_fb_vec(0b_00_00_00_00_00_00_00_00_u16);
    let expected_fb = as_fb_vec(0b_00_00_00_00_01_01_01_00_u16);

    blit_sub(&mut fb, &sprite, 0, 0, 8, 1, 0, 0, 8, 0, draw_colors);
    assert_eq!(as_fb_line(&expected_fb), as_fb_line(&fb));

    // because of the draw color config the 0 bits of this
    // 1BPP sprite are transparent. the formatting shows how
    // the individual pixels align with the framebuffer pixes
    // (which are 2 bits wide; the sprite pixes are 1 bit wide)
    let sprite =      as_b1_vec(0b__0__0__0__0__1__1__1__0_u8);
    let mut fb =      as_fb_vec(0b_00_10_10_00_11_11_11_11_u16);
    // the result is visible in that for each 1 bit in the sprite,
    // a 01 pixes is written into the fb
    let expected_fb = as_fb_vec(0b_00_10_10_00_01_01_01_11_u16);

    blit_sub(&mut fb, &sprite, 0, 0, 8, 1, 0, 0, 8, 0, draw_colors);

    assert_eq!(as_fb_line(&fb), as_fb_line(&expected_fb));
}

#[test]
fn test_blit_sub_impl_1byte_misaligned() {
    let draw_colors = 0x4320;

    // in this example, we write a 2BPP sprite into
    // the frame buffer at a position where the sprite
    // byte falls into two target fb bytes (x=2). You can
    // see this in the indentation we use. The example
    // is drawing on an empty framebuffer (filled with
    // 00 pixels).
    let sprite =            as_fb_vec(0b_10_11_11_10__u8);
    let mut fb =      as_fb_vec(0b_00_00_00_00_00_00_00_00__u16);

    // as a result, half the bits are written into the each side of the
    // fb
    let expected_fb = as_fb_vec(0b_00_00_10_11_11_10_00_00__u16);

    blit_sub(&mut fb, &sprite, 2, 0, 4, 1, 0, 0, 8, BLIT_2BPP, draw_colors);

    assert_eq!(fb, expected_fb)
}

// Convert arbitrary primitive integer types (aka u8..u128/i8..i128) 
// into a Vec<u8> for use in framebuffer related functions, allowing
// to define sprites and framebuffer patterns inline with rusts
// integer (bit) literals.
// This is needed for testing so that we can conveniently spell out bit
// patterns in various sizes, yielding them as Vec<u8>
fn as_fb_vec<T>(n: T) -> Vec<u8>
where T: PrimInt + Unsigned
{
    as_pix_vec(n, 2)
}

fn as_b1_vec<T>(n: T) -> Vec<u8>
where T: PrimInt + Unsigned
{
    as_pix_vec(n, 1)
}

fn as_pix_vec<T>(mut n: T, pix_size: usize) -> Vec<u8>
where T: PrimInt + Unsigned
{
    let mut v = Vec::with_capacity(size_of::<T>());
    let mask = T::from(0xff>>(8-pix_size)).unwrap();
    for i in 0..size_of::<T>() {
        
        let mut b = 0u8;
        for j in 0..(8/pix_size) {
            b = b << pix_size;
            b |= n.bitand(mask).to_u8().unwrap();
            n = n.shr(pix_size);
        }
        v.insert(0, b);
    }

    v
}

#[test]
fn test_as_fb_vec_u8() {
    // happy case, coming from u8
    assert_eq!(vec![0b10010110u8], as_fb_vec(0b10010110u8));

    // u16 into a vec of 2 x u8 - the two tests are equivalent, but
    // shows we can write it also as 0x as well as a 0b literal
    assert_eq!(vec![0b_11000111_u8, 0b_10010110_u8], as_fb_vec(0b_11010011_10010110_u16));
}

#[test]
fn test_as_b1_vec() {
    assert_eq!(vec![0b01110000u8], as_b1_vec(0b__0__0__0__0__1__1__1__0_u8))
}

#[test]
fn test_blit_sub_atlas() {

    let draw_colors = 0x4321;

    let src_x = 3;
    let src_end_x = 8;
    let src_y = 0;
    let width = src_end_x - src_x;
    let height = 1;
    let sprite = as_fb_vec(0b_00_00_00_01_10_11_01_10_00_00_00_00_00_00_00_00_u32);
    let stride = (sprite.len()*4) as u32;

    // fb is all zeros
    let mut fb =      as_fb_vec(0b_00_00_00_00_00_00_00_00_00_00_00_00_00_00_00_00_u32);

    // happy case: sprite and fb match one on one
    let expected_fb = as_fb_vec(0b_00_00_00_01_10_11_01_10_00_00_00_00_00_00_00_00_u32);
    blit_sub(&mut fb, &sprite, 3, 0, width, height, src_x, src_y, stride, BLIT_2BPP, draw_colors);
    assert_eq!(as_fb_line(&expected_fb), as_fb_line(&fb));



    // initial fb. we have it filled with pixels set to 10 so we can spot
    // the difference after blitting
    let mut fb =      as_fb_vec(0b_10_10_10_10_10_10_10_10_10_10_10_10_10_10_10_10__u32);

    let x = 3;
    let y = 0;
    let expected_fb = as_fb_vec(0b_10_10_10_10_11_00_11_11_11_10_11_10_10_10_10_10__u32);

    blit_sub(&mut fb, &sprite, x, y, width, height, src_x, src_y, stride, BLIT_2BPP, draw_colors);

    assert_eq!(as_fb_line(&expected_fb), as_fb_line(&fb));
}


fn conv_1bpp_to_2bpp(pixbuf: u8) -> u32 {
    // convert 1BPP to 2BPP format
    let pixbuf = pixbuf as u32;
    let mut mask = 0x01;
    let mut tgt = 0;
    for shift in 0..8 {
        tgt |= (pixbuf & mask) << shift;

        mask <<= 1;
    }
    tgt
}

#[test]
fn test_conv_1bpp_to_2bpp() {
    assert_eq!(0b01_01_01_01_01_01_01_01, conv_1bpp_to_2bpp(0b0000000011111111));
    assert_eq!(0b01_01_01_01_00_00_00_00, conv_1bpp_to_2bpp(0b0000000011110000));
}

fn remap_draw_color(draw_color_idx: u8, draw_colors: u16) -> (u8, bool) {
    let draw_color = (draw_colors as u32 >> (draw_color_idx * 4)) & 0b111;
    if draw_color == 0 {
        (0, false)
    } else {
        let palette_index = (draw_color - 1) as u8;
        (palette_index, true)
    }
}

/// convert a machine word of sprite pixels to actual FB indices and a mask
/// for applying transparent pixels
fn remap_draw_colors(sprite_word: u32, sprite_word_pixels: i32, draw_colors: u16) -> (u32, u32){
    let mut s = 0;
    let mut m = 0;
    for n in 0..sprite_word_pixels {
        let shift = 2 * n;
        let draw_color_idx = (sprite_word >> shift) & 0b11;

        let (palette_index, opaque) = remap_draw_color(draw_color_idx as u8, draw_colors);
        if !opaque {
            m |= 0b11 << shift;
        } else {
            s |= (palette_index as u32) << shift
        }
    }

    (s, m)
}

#[test]
fn test_remap_draw_colors() {
    assert_eq!((0b01_00_10_00, 0b00_11_00_00), remap_draw_colors(0b11100001, 4, 0x2013));
}

// see https://github.com/aduros/wasm4/blob/main/runtimes/native/src/framebuffer.c
// who in turn took it from https://github.com/nesbox/TIC-80/blob/master/src/core/draw.c
pub(crate) fn line<T: Source<u8> + Sink<u8>>(
    fb: &mut T,
    draw_colors: u16,
    mut x1: i32,
    mut y1: i32,
    mut x2: i32,
    mut y2: i32,
) {
    let dc0: u8 = (draw_colors & 0xf) as u8;
    if dc0 == 0 {
        return;
    }

    let stroke_color: u8 = (dc0 - 1) & 0x3;

    if y1 > y2 {
        let swap = x1;
        x1 = x2;
        x2 = swap;

        let swap = y1;
        y1 = y2;
        y2 = swap;
    }

    let dx = (x2 - x1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let dy = y2 - y1;

    let mut err = (if dx > dy { dx } else { -dy }) / 2;

    // we won't have to ever go through the entirety of FRAMEBUFFER_SIZE,
    // I just added this so the loop will stop incase something goes really wrong.
    for _ in 0..crate::wasm4::FRAMEBUFFER_SIZE {
        set_pixel_unclipped(fb, x1, y1, stroke_color);

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

fn as_fb_line(v: &Vec<u8>) -> String {
    let prefix = "0b";
    let mut s = String::with_capacity(prefix.len() + v.len() * 4 * 3);
    s += prefix;
    for e in v {
        let b = *e as u16;
        for n in 0..4 {
            s += "_";
            s += &((b >> (2*n+1)) & 0b1).to_string();
            s += &((b >> (2*n+0)) & 0b1).to_string();
        }
    }

    s
}

#[test]
fn test_as_fb_line() {
    let fb_vec = as_fb_vec(0b_10_10_10_10_11_00_11_11_11_10_11_10_10_10_10_10__u32);
    assert_eq!("0b_10_10_10_10_11_00_11_11_11_10_11_10_10_10_10_10", as_fb_line(&fb_vec));
}
