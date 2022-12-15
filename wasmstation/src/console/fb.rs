// WASM-4 Framebuffer functions

use std::ops::Range;

use crate::{wasm4::{SCREEN_SIZE, BLIT_2BPP}, Sink, Source};

impl <T, > Sink<T> for Vec<T>
where T: Copy
{
    fn set_item_at(&mut self, offset: usize, item: T) {
        self[offset] = item
    }
}

impl <T> Source<T> for Vec<T>
where T: Copy
{
    fn item_at(&self, offset: usize) -> T {
        self[offset]
    }
}


// : 
//  overlay_r+offset |.........|
//  base_r             |...........|
//  result+offset      |.......|

fn clamp_range(overlay_r: Range<i32>, offset: i32, base_r: Range<i32>) -> Range<i32>{
    let abs_start = overlay_r.start + offset;
    let start_diff = if abs_start < base_r.start {
        base_r.start - abs_start
    } else {
        0
    };

    let abs_end = overlay_r.start + offset;
    let end_diff = if base_r.end < abs_end {
        abs_end - base_r.end
    } else {
        0
    };
    Range{start: overlay_r.start+start_diff, end: overlay_r.end+end_diff}
}

pub(crate) fn pixel_width_of_flags(flags: u32) -> u32 {
    if flags & BLIT_2BPP != 0 {
        2
    } else {
        1
    }
}

pub(crate) fn blit_sub<S,T>(
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
    draw_colors: u16
    ) 
    where 
        S: Source<u8>,
        T: Source<u8> + Sink<u8>
    {

    let pixel_width = pixel_width_of_flags(flags);

    let num_bits_in_sprite = stride * height * pixel_width;
    let len = (num_bits_in_sprite + 7) / 8;

    let sx_r = clamp_range(0..(width  as i32), x, 0..(SCREEN_SIZE as i32));
    let sy_r = clamp_range(0..(height as i32), y, 0..(SCREEN_SIZE as i32));
    let mut tx = (x + sx_r.start) as u32;
    let mut ty = (y + sy_r.start) as u32;
    let sx_r = Range{ start: sx_r.start as u32, end: sx_r.end as u32};
    let sy_r = Range{ start: sy_r.start as u32, end: sy_r.end as u32};
    
    // the number of bits to left-shift pixel index to get to the sprite byte that contains it
    let index_shift;
    if flags & BLIT_2BPP != 0 {
        index_shift = 2;
    } else {
        index_shift = 3;
    }
    
    // calculate number bytes in sprite for a single
    // line in the sprite. Takes into account that
    // tx and width may not be divisible by 8, implying
    // that we may need an extra byte
    let target_bytes_per_line = ((((tx & 0x03) + (sx_r.end-sx_r.start)) << 1) + 7) / 8;



    // initialize sprite pixel buffer. This buffer u32 holds 
    // the sprite bits that we'll apply next to the coming target
    // byte. Excess beits (bit 8+) are masked out when applying, and
    // are shifted down by 8 bits right after. 
    //
    // pixbuf: the sprite pixel buffer
    // pixbuf_len: the number of bits currently stored in pixbuf.
    //
    // initally, pixbuf is initialized to 0. Hoever, pixbuf_len
    // may be initialized to nonzero, to accomodate offset bits
    // if we're starting to write in the middle of a frame
    // buffer byte.
    let mut pixbuf = 0u32;
    let mut maskbuf = 0u32;
    let mut pixbuf_len = ((tx as i32 & 0x3) - (sx_r.start as i32 & 0x3)) << (pixel_width-1);

    let mut tgt_start_idx = (tx + ty*SCREEN_SIZE)>>2;
    for sy in sy_r.start..sy_r.end {
        
        // index of the first sprite byte in this line
        let sprite_line_start_idx = ((sx_r.start + sy * stride) >> index_shift) as usize;
        let mut sprite_idx = sprite_line_start_idx;
        let mut sprite_pixels_left = sx_r.end - sx_r.start;

        for n in tgt_start_idx..(tgt_start_idx + target_bytes_per_line) {
            
            // if there's room in pixbuf, get next sprite byte
            if pixbuf_len < 8 && sprite_pixels_left > 0 {

                // load next u8 from sprite line
                let sprite_byte = sprite.item_at(sprite_idx);
                sprite_idx += 1;

                let sprite_word;
                let sprite_byte_pixel_capacity: u32;
                if flags & BLIT_2BPP == 0 {
                    sprite_word = conv_1bpp_to_2bpp(sprite_byte);
                    sprite_byte_pixel_capacity = 8;
                } else {
                    sprite_word = sprite_byte as u32;
                    sprite_byte_pixel_capacity = 4;
                }

                // remap colors via draw_colors
                let sprite_pixels;
                let (mut sprite_word, mask_word) = remap_draw_colors(sprite_word, sprite_byte_pixel_capacity, draw_colors);

                if sprite_pixels_left < sprite_byte_pixel_capacity {
                    // if we're reading the right fringe of a sprite line,
                    // mask the excess bits away
                    sprite_pixels = sprite_pixels_left;
                    sprite_word &= (!0u32) >> (32-sprite_pixels_left*2)
                } else {
                    sprite_pixels = sprite_byte_pixel_capacity;
                }
                sprite_pixels_left -= sprite_pixels;
                let bits = sprite_pixels << 1;

                pixbuf = pixbuf | (sprite_word << pixbuf_len);
                maskbuf = maskbuf | (mask_word << pixbuf_len);
                pixbuf_len += bits as i32;
            }

            // load source byte from pixbuf
            let src_byte = pixbuf as u8;
            let mask_byte = maskbuf as u8;
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
        tgt_start_idx += SCREEN_SIZE / 4;
    }

}


#[test]
fn test_blit_sub_impl_1byte() {

    let draw_colors = 0x4320;

    // regular
    let sprite = vec![0b00001111u8];
    let mut fb = vec![0u8;2];

    blit_sub(&mut fb, &sprite, 0, 0, 8, 1, 0, 0, 8, 0, draw_colors);

    assert_eq!(fb, vec![0b01010101, 0b00000000]);

    // with background
    let sprite = vec![0b00001110u8];
    let mut fb = vec![0b11111111, 0b00101000];

    blit_sub(&mut fb, &sprite, 0, 0, 8, 1, 0, 0, 8, 0, draw_colors);

    assert_eq!(fb, vec![0b01010111, 0b00101000])
}

#[test]
fn test_blit_sub_impl_1byte_misaligned() {

    let draw_colors = 0x4320;
    let sprite = vec![0b1111_1111u8];
    let mut fb = vec![0u8;2];

    blit_sub(&mut fb, &sprite, 2, 0, 4, 1, 0, 0, 8, BLIT_2BPP, draw_colors);

    assert_eq!(fb, vec![0b11110000, 0b00001111])
}

fn conv_1bpp_to_2bpp(pixbuf: u8) -> u32 {
    // convert 1BPP to 2BPP format
    let pixbuf = pixbuf as u32;
    let mut mask = 0x01;
    let mut tgt = 0;
    for shift in 0..8 {
        tgt |= (pixbuf & mask) << shift;

        mask = mask << 1;
    }
    tgt
}

#[test]
fn test_conv_1bpp_to_2bpp() {
    assert_eq!(0b0101010101010101, conv_1bpp_to_2bpp(0b0000000011111111));
    assert_eq!(0b0101010100000000, conv_1bpp_to_2bpp(0b0000000011110000));
}

fn remap_draw_colors(sprite_word: u32, sprite_word_pixels: u32, draw_colors: u16) -> (u32, u32){
    let mut s = 0;
    let mut m = 0;
    for n in 0..sprite_word_pixels {
        let shift = 2*n;
        let draw_color_idx = (sprite_word >> shift) & 0b11;
        let draw_color = (draw_colors as u32 >> (draw_color_idx*4)) & 0b111;
        if draw_color == 0 {
            m |= 0b11 << shift;
        } else {
            let palette_index = draw_color - 1;
            s |= palette_index << shift;
        }
    }

    (s,m)
}

#[test]
fn test_remap_draw_colors() {
    assert_eq!((0b01001000, 0b00110000), remap_draw_colors(0b11100001, 4, 0x2013));
}