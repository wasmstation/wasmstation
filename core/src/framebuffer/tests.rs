use core::mem;
use num_traits::{PrimInt, Unsigned};
use std::fmt::Write;

use crate::{
    blit_sub,
    framebuffer::{
        blit::PixelFormat,
        line::{hline_impl, line_impl, vline_impl},
        oval::oval_impl,
        Screen,
    },
    wasm4::BLIT_2BPP,
};

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

/// create a Vec<u8> of framebuffer data from pixels from an integer literal
pub fn as_fb_vec<T>(n: T) -> Vec<u8>
where
    T: PrimInt + Unsigned,
{
    as_pix_vec(n, PixelFormat::Framebuffer)
}

/// create a Vec<u8> of 1BPP sprite data from pixels from an integer literal
pub fn as_b1_vec<T>(n: T) -> Vec<u8>
where
    T: PrimInt + Unsigned,
{
    as_pix_vec(n, PixelFormat::Blit1BPP)
}

/// create a Vec<u8> of 2BPP sprite data from pixels from an integer literal
pub fn as_b2_vec<T>(n: T) -> Vec<u8>
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

    let mut v = Vec::with_capacity(mem::size_of::<T>());
    let mask = T::from(0xff >> (8 - pix_size)).unwrap();
    for i in 0..mem::size_of::<T>() {
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

#[test]
fn test_hline() {
    let mut screen = ArrayScreen::<4, 8>::new();
    let expected = ArrayScreen::new_with_fb_lines(&[
        as_fb_vec(0b_00_11_11_11_11_11_11_00__u16),
        as_fb_vec(0b_11_11_11_11_11_11_00_00__u16),
    ]);

    hline_impl(&mut screen, 3, 1, 0, 6);
    hline_impl(&mut screen, 3, -1, 1, 7);

    println!("{:?}", &screen);
    assert_eq!(screen, expected);
}

#[test]
fn test_vline() {
    let mut screen = ArrayScreen::<7, 4>::new();
    let expected = ArrayScreen::new_with_fb_lines(&[
        as_fb_vec(0b_00_00_00_00__u8),
        as_fb_vec(0b_11_00_11_00__u8),
        as_fb_vec(0b_11_00_11_00__u8),
        as_fb_vec(0b_11_00_11_00__u8),
        as_fb_vec(0b_00_00_11_00__u8),
        as_fb_vec(0b_11_00_11_00__u8),
        as_fb_vec(0b_00_00_11_00__u8),
    ]);

    vline_impl(&mut screen, 3, 2, 1, 6);
    vline_impl(&mut screen, 3, 0, 1, 3);
    vline_impl(&mut screen, 3, 0, 5, 1);

    println!("{:?}", &screen);
    assert_eq!(screen, expected);
}

#[test]
fn test_line() {
    let mut screen = ArrayScreen::<18, 8>::new();
    let expected = ArrayScreen::new_with_fb_lines(&[
        as_fb_vec(0b_11_00_00_00_00_00_00_00__u16),
        as_fb_vec(0b_00_11_00_00_00_00_11_00__u16),
        as_fb_vec(0b_00_00_11_00_00_00_11_00__u16),
        as_fb_vec(0b_00_00_00_11_00_00_11_00__u16),
        as_fb_vec(0b_00_00_00_00_00_00_00_11__u16),
        as_fb_vec(0b_00_11_00_00_00_00_00_11__u16),
        as_fb_vec(0b_00_11_00_00_00_00_00_11__u16),
        as_fb_vec(0b_11_00_00_00_11_11_11_00__u16),
        as_fb_vec(0b_11_00_00_00_00_00_00_00__u16),
    ]);

    line_impl(&mut screen, 3, -1, -1, 3, 3);
    line_impl(&mut screen, 3, 0, 8, 1, 5);
    line_impl(&mut screen, 3, 6, 1, 7, 6);
    line_impl(&mut screen, 3, 4, 7, 6, 7);

    println!("{:?}", &screen);
    assert_eq!(screen, expected);
}

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
