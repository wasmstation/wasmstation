use crate::{Sink, Source};

use super::{hline_impl, set_pixel_unclipped_impl, Screen, Wasm4Screen};

/// Draw an oval (circle).
pub fn oval<T: Sink<u8> + Source<u8>>(
    fb: &mut T,
    draw_colors: u16,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) {
    let mut screen = Wasm4Screen { fb };
    oval_impl(&mut screen, draw_colors, x, y, width, height)
}

/// Draw axis parallel ellipse centered around `x` and `y` with given `width` and
/// `height`. The algorithm aligns with what is implemented in W4's framebuffer.c
pub(crate) fn oval_impl<T: Screen>(
    screen: &mut T,
    draw_colors: u16,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) {
    let width = width as i32;
    let height = height as i32;

    let dc0 = draw_colors & 0xf;
    let dc1 = (draw_colors >> 4) & 0xf;

    if dc1 == 0xf {
        return;
    }

    let stroke = ((dc1.overflowing_sub(1).0) & 0x3) as u8;
    let fill = (dc0.overflowing_sub(1).0 & 0x3) as u8;

    let mut a = width - 1;
    let b = height - 1;
    let mut b1 = b % 2;

    let mut north = y + height / 2;
    let mut west = x;
    let mut east = x + width - 1;
    let mut south = north - b1;

    let b2 = b * b;
    let a2 = a * a;

    let mut dx = 4 * (1 - a) * b2;
    let mut dy = 4 * (b1 + 1) * a2;
    let mut err = dx + dy + b1 * a2;

    a = 8 * a2;
    b1 = 8 * b2;

    while west <= east {
        set_pixel_unclipped_impl(screen, east, north, stroke);
        set_pixel_unclipped_impl(screen, west, north, stroke);
        set_pixel_unclipped_impl(screen, west, south, stroke);
        set_pixel_unclipped_impl(screen, east, south, stroke);

        let start = west + 1;

        if dc0 != 0 && (east - start) > 0 {
            let len = (east - west - 1) as u32;

            hline_impl(screen, fill, start, north, len);
            hline_impl(screen, fill, start, south, len);
        }

        let err2 = 2 * err;

        if err2 <= dy {
            north += 1;
            south -= 1;
            dy += a;
            err += dy;
        }

        if err2 >= dx || err2 > dy {
            west += 1;
            east -= 1;
            dx += b1;
            err += dx;
        }
    }

    while north - south < height {
        set_pixel_unclipped_impl(screen, west - 1, north, stroke);
        set_pixel_unclipped_impl(screen, east + 1, north, stroke);
        north += 1;

        set_pixel_unclipped_impl(screen, west - 1, south, stroke);
        set_pixel_unclipped_impl(screen, east + 1, south, stroke);
        south -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::oval_impl;
    use crate::console::framebuffer::{as_fb_vec, ArrayScreen};

    #[test]
    fn test_oval_small_circular() {
        // 8x5 pixels, with 4 pix/byte, that's 2 bytes/row, 10 bytes in total
        let mut screen = ArrayScreen::<10, 8>::new();
        oval_impl(&mut screen, 0x40, 0, 0, 5, 5);

        let expected = ArrayScreen::new_with_fb_lines(&[
            as_fb_vec(0b_00_11_11_11_00_00_00_00__u16),
            as_fb_vec(0b_11_00_00_00_11_00_00_00__u16),
            as_fb_vec(0b_11_00_00_00_11_00_00_00__u16),
            as_fb_vec(0b_11_00_00_00_11_00_00_00__u16),
            as_fb_vec(0b_00_11_11_11_00_00_00_00__u16),
        ]);

        assert_eq!(screen, expected);
    }

    #[test]
    fn test_oval_slim_horizontal() {
        // 8x3 pixels, with 4 pix/byte, that's 2 bytes/row, 6 bytes in total
        let mut screen = ArrayScreen::<6, 8>::new();
        oval_impl(&mut screen, 0x40, 0, 0, 8, 3);

        let expected = ArrayScreen::new_with_fb_lines(&[
            as_fb_vec(0b_00_00_11_11_11_11_00_00__u16),
            as_fb_vec(0b_11_11_00_00_00_00_11_11__u16),
            as_fb_vec(0b_00_00_11_11_11_11_00_00__u16),
        ]);

        assert_eq!(screen, expected);
    }
}
