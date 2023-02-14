use crate::{Sink, Source};

use super::{
    hline_impl, remap_draw_color, set_pixel_unclipped_impl, Screen, Wasm4Screen, DRAW_COLOR_1,
    DRAW_COLOR_2,
};

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

    // name variables like in the Wikipedia article.
    let a = width - 1;
    let b = height - 1;

    let b0 = b % 2;

    let a2 = a * a;
    let b2 = b * b;
    let mut dx = 4 * (1 - a) * b2;
    let mut dy = 4 * (1 + b0) * a2;
    let mut err = dx + dy + b0 * a2;

    let fill_opaque: Option<u8> = remap_draw_color(DRAW_COLOR_1, draw_colors);
    let line_opaque: Option<u8> = remap_draw_color(DRAW_COLOR_2, draw_colors);

    // x1, x2 start on the left and right maxima of the horizontal axis
    let mut x1 = x;
    let mut x2 = x + width - 1;
    // y1, y2 start in the very center, 1px off in case of odd heights
    let mut y2 = y + height / 2;
    let mut y1 = y2 - b0;

    let dx_inc = 8 * a2;
    let dy_inc = 8 * b2;

    while x1 <= x2 {
        // ellipse outline with line color
        if let Some(line_stroke) = line_opaque {
            set_pixel_unclipped_impl(screen, x1, y2, line_stroke);
            set_pixel_unclipped_impl(screen, x2, y2, line_stroke);
            set_pixel_unclipped_impl(screen, x1, y1, line_stroke);
            set_pixel_unclipped_impl(screen, x2, y1, line_stroke);
        }

        let e2 = 2 * err;

        if e2 <= dy {
            // filled ellipse with fill color
            if let Some(fill_stroke) = fill_opaque {
                let len = (x2 - x1 - 1) as u32;
                hline_impl(screen, fill_stroke, x1 + 1, y2, len);
                hline_impl(screen, fill_stroke, x1 + 1, y1, len);
            }
            y2 += 1;
            y1 -= 1;
            dy += dy_inc;
            err += dy;
        }
        if e2 >= dx || e2 > dy {
            x1 += 1;
            x2 -= 1;
            dx += dx_inc;
            err += dx;
        }
    }

    while y2 - y1 < height {
        set_pixel_unclipped_impl(screen, x1 - 1, y1, line_opaque.unwrap_or(0));
        set_pixel_unclipped_impl(screen, x2 + 1, y1, line_opaque.unwrap_or(0));
        set_pixel_unclipped_impl(screen, x1 - 1, y2, line_opaque.unwrap_or(0));
        set_pixel_unclipped_impl(screen, x2 + 1, y2, line_opaque.unwrap_or(0));
        y2 += 1;
        y1 -= 1;
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

        let expected = ArrayScreen::new_with_fb_lines(&[
            as_fb_vec(0b_00_11_11_11_00_00_00_00__u16),
            as_fb_vec(0b_11_00_00_00_11_00_00_00__u16),
            as_fb_vec(0b_11_00_00_00_11_00_00_00__u16),
            as_fb_vec(0b_11_00_00_00_11_00_00_00__u16),
            as_fb_vec(0b_00_11_11_11_00_00_00_00__u16),
        ]);

        oval_impl(&mut screen, 0x0040, 0, 0, 5, 5);
        println!("{:?}", &screen);
        assert_eq!(screen, expected);
    }

    #[test]
    fn test_oval_slim_horizontal() {
        // 8x3 pixels, with 4 pix/byte, that's 2 bytes/row, 6 bytes in total
        let mut screen = ArrayScreen::<6, 8>::new();

        let expected = ArrayScreen::new_with_fb_lines(&[
            as_fb_vec(0b_00_00_00_11_11_00_00_00__u16),
            as_fb_vec(0b_11_11_11_11_11_11_11_11__u16),
            as_fb_vec(0b_00_00_00_11_11_00_00_00__u16),
        ]);

        oval_impl(&mut screen, 0x0040, 0, 0, 8, 3);
        println!("{:?}", &screen);
        assert_eq!(screen, expected);
    }
}
