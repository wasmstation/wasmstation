use core::ops::Range;

use crate::core::{
    wasm4::{BLIT_2BPP, BLIT_FLIP_X, BLIT_FLIP_Y, BLIT_ROTATE, SCREEN_SIZE},
    Sink, Source,
};

use super::{remap_draw_color, set_pixel_unclipped};

#[derive(Clone, Copy)]
pub(crate) enum PixelFormat {
    Blit1BPP,
    Blit2BPP,
    #[cfg(test)]
    Framebuffer,
}

impl PixelFormat {
    fn from_blit_flags(flags: u32) -> Self {
        if flags & BLIT_2BPP != 0 {
            Self::Blit2BPP
        } else {
            Self::Blit1BPP
        }
    }
}

/// Copy a subregion within a larger sprite atlas to the framebuffer.
///
/// Same as `blit`, but with three additional parameters.
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

    for wy in w_range_y {
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
            let mut byte = sprite.item_at((pixel_index >> 3) as usize).unwrap();
            byte >>= 7 - (pixel_index & 0x07);
            byte & 0x01
        }
        PixelFormat::Blit2BPP => {
            let mut byte = sprite.item_at((pixel_index >> 2) as usize).unwrap();
            byte >>= 6 - ((pixel_index & 0x03) << 1);
            byte & 0x03
        }
        #[cfg(test)]
        PixelFormat::Framebuffer => panic!("invalid pixel format for reading sprite data"),
    }
}

/// Get the pixel width from blit flags parameter.
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
