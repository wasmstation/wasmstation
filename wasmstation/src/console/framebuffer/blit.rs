use core::ops::Range;

use crate::{
    wasm4::{BLIT_2BPP, BLIT_FLIP_X, BLIT_FLIP_Y, BLIT_ROTATE, SCREEN_SIZE},
    Sink, Source,
};

use super::{remap_draw_color, remap_draw_colors, set_pixel_unclipped};

#[derive(Clone, Copy)]
pub enum PixelFormat {
    Blit1BPP,
    Blit2BPP,
    Framebuffer,
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

/// Copy a subregion within a larger sprite atlas to the framebuffer. Same as `blit`, but with three additional parameters.
#[allow(clippy::too_many_arguments)]
pub fn blit_sub<S, T>(
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
            let tgt_location = if rotate { (wy, wx) } else { (wx, wy) };
            let tx = x + tgt_location.0;
            let ty = y + tgt_location.1;

            // source coordinates where the sprite pixel will be read from,
            // relative to sprite start coordinates src_x, src_y
            let src_location = (
                if flip_x { width - wx - 1 } else { wx },
                if flip_y { height - wy - 1 } else { wy },
            );
            let sx = src_x + src_location.0;
            let sy = src_y + src_location.1;

            let draw_color_idx = get_sprite_pixel_draw_color(sprite, fmt, sx, sy, stride);
            if let Some(color) = remap_draw_color(draw_color_idx, draw_colors) {
                set_pixel_unclipped(target, tx, ty, color)
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn blit_sub_multipixel<S, T>(
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
    let (tx, sx_r) = clamp_range(src_x..(src_x + width as i32), x);
    let (ty, sy_r) = clamp_range(src_y..(src_y + height as i32), y);

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
    let mut pixbuf_len = (tx & 0x3) << (pixel_width - 1);

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

                let sprite_word_shift = pixbuf_len - ((sx_r.start & 0x3) << 1);
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

fn get_sprite_pixel_draw_color<T: Source<u8>>(
    sprite: &T,
    fmt: PixelFormat,
    x: i32,
    y: i32,
    width: i32,
) -> u8 {
    let pixel_index = width * y + x;
    match fmt {
        PixelFormat::Blit1BPP => {
            let mut byte = sprite.item_at((pixel_index >> 3) as usize);
            byte = byte >> (7 - (pixel_index & 0x07));
            byte & 0x01
        }
        PixelFormat::Blit2BPP => {
            let mut byte = sprite.item_at((pixel_index >> 2) as usize);
            byte = byte >> (6 - ((pixel_index & 0x03) << 1));
            byte & 0x03
        }
        _ => panic!("invalid pixel format for reading sprite data"),
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
fn clamp_range(mut sprite_r: Range<i32>, mut offset: i32) -> (i32, Range<i32>) {
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

/// Get pixel width from blit flags parameter.
pub fn pixel_width_of_flags(flags: u32) -> u32 {
    if flags & BLIT_2BPP != 0 {
        2
    } else {
        1
    }
}

fn calculate_target_range(tgt_coord: i32, tgt_extent: i32, clip_range: Range<i32>) -> Range<i32> {
    Range {
        start: i32::max(clip_range.start, tgt_coord) - tgt_coord,
        end: i32::min(tgt_extent, clip_range.end - tgt_coord),
    }
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

#[cfg(test)]
mod tests {
    use crate::{
        console::framebuffer::{
            as_b1_vec, as_b2_vec, as_fb_line, as_fb_vec, blit::conv_1bpp_to_2bpp, blit_sub,
        },
        wasm4::BLIT_2BPP,
    };

    #[test]
    fn test_blit_sub_impl_1byte() {
        let draw_colors = 0x4320;

        // regular
        let sprite = as_b1_vec(0b__0__0__0__0__1__1__1__0_u8);
        let mut fb = as_fb_vec(0b_00_00_00_00_00_00_00_00_u16);
        let expected_fb = as_fb_vec(0b_00_00_00_00_01_01_01_00_u16);

        blit_sub(&mut fb, &sprite, 0, 0, 8, 1, 0, 0, 8, 0, draw_colors);
        assert_eq!(as_fb_line(&expected_fb), as_fb_line(&fb));

        // because of the draw color config the 0 bits of this
        // 1BPP sprite are transparent. the formatting shows how
        // the individual pixels align with the framebuffer pixes
        // (which are 2 bits wide; the sprite pixes are 1 bit wide)
        let sprite = as_b1_vec(0b__0__0__0__0__1__1__1__0_u8);
        let mut fb = as_fb_vec(0b_00_10_10_00_11_11_11_11_u16);
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
        let sprite = as_b2_vec(0b_10_11_11_10__u8);
        let mut fb = as_fb_vec(0b_00_00_00_00_00_00_00_00__u16);

        // as a result, half the bits are written into the each side of the
        // fb
        let expected_fb = as_fb_vec(0b_00_00_10_11_11_10_00_00__u16);

        blit_sub(
            &mut fb,
            &sprite,
            2,
            0,
            4,
            1,
            0,
            0,
            8,
            BLIT_2BPP,
            draw_colors,
        );

        assert_eq!(as_fb_line(&fb), as_fb_line(&expected_fb))
    }

    #[test]
    fn test_blit_sub_atlas() {
        let draw_colors = 0x4321;

        let src_x = 3;
        let src_end_x = 8;
        let src_y = 0;
        let width = src_end_x - src_x;
        let height = 1;
        let sprite = as_b2_vec(0b_00_00_00_01_10_11_01_10_00_00_00_00_00_00_00_00_u32);
        let stride = (sprite.len() * 4) as u32;

        // fb is all zeros
        let mut fb = as_fb_vec(0b_00_00_00_00_00_00_00_00_00_00_00_00_00_00_00_00_u32);

        // happy case: sprite and fb match one on one
        let expected_fb = as_fb_vec(0b_00_00_00_01_10_11_01_10_00_00_00_00_00_00_00_00_u32);
        blit_sub(
            &mut fb,
            &sprite,
            3,
            0,
            width,
            height,
            src_x,
            src_y,
            stride,
            BLIT_2BPP,
            draw_colors,
        );
        assert_eq!(as_fb_line(&expected_fb), as_fb_line(&fb));

        // initial fb. we have it filled with pixels set to 10 so we can spot
        // the difference after blitting
        let mut fb = as_fb_vec(0b_10_10_10_10_10_10_10_10_10_10_10_10_10_10_10_10__u32);

        let expected_fb = as_fb_vec(0b_10_10_10_01_10_11_01_10_10_10_10_10_10_10_10_10__u32);

        blit_sub(
            &mut fb,
            &sprite,
            3,
            0,
            width,
            height,
            src_x,
            src_y,
            stride,
            BLIT_2BPP,
            draw_colors,
        );

        assert_eq!(as_fb_line(&expected_fb), as_fb_line(&fb));
    }

    #[test]
    fn test_conv_1bpp_to_2bpp() {
        assert_eq!(
            0b01_01_01_01_01_01_01_01,
            conv_1bpp_to_2bpp(0b0000000011111111)
        );
        assert_eq!(
            0b01_01_01_01_00_00_00_00,
            conv_1bpp_to_2bpp(0b0000000011110000)
        );
    }
}
