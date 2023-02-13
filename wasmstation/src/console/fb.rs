// WASM-4 Framebuffer functions

use std::{ops::Range, mem::size_of, fmt::Write};

use num_traits::{PrimInt, Unsigned};


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

    fn fill(&mut self, item: T) {
        <[T]>::fill(self, item)
    }
}

impl<const N: usize, T> Sink<T> for [T; N]
where
    T: Copy,
{
    fn set_item_at(&mut self, offset: usize, item: T) {
        self[offset] = item
    }

    fn fill(&mut self, item: T) {
        <[T]>::fill(self, item)
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

impl<const N: usize, T> Source<T> for [T; N]
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

const DRAW_COLOR_1: u8 = 0;
const DRAW_COLOR_2: u8 = 1;
const DRAW_COLOR_3: u8 = 2;
const DRAW_COLOR_4: u8 = 3;

// charset
const CHARSET_WIDTH: u32 = 128;
const CHARSET_HEIGHT: u32 = 112;
const CHARSET_FLAGS: u32 = 0; // BLIT_1BPP
const CHARSET: [u8; 1792] = [ 0xff,0xc7,0x93,0x93,0xef,0x9d,0x8f,0xcf,0xf3,0x9f,0xff,0xff,0xff,0xff,0xff,0xfd,0xff,0xc7,0x93,0x01,0x83,0x5b,0x27,0xcf,0xe7,0xcf,0x93,0xe7,0xff,0xff,0xff,0xfb,0xff,0xc7,0x93,0x93,0x2f,0x37,0x27,0xcf,0xcf,0xe7,0xc7,0xe7,0xff,0xff,0xff,0xf7,0xff,0xcf,0xff,0x93,0x83,0xef,0x8f,0xff,0xcf,0xe7,0x01,0x81,0xff,0x81,0xff,0xef,0xff,0xcf,0xff,0x93,0xe9,0xd9,0x25,0xff,0xcf,0xe7,0xc7,0xe7,0xff,0xff,0xff,0xdf,0xff,0xff,0xff,0x01,0x03,0xb5,0x33,0xff,0xe7,0xcf,0x93,0xe7,0xcf,0xff,0xcf,0xbf,0xff,0xcf,0xff,0x93,0xef,0x73,0x81,0xff,0xf3,0x9f,0xff,0xff,0xcf,0xff,0xcf,0x7f,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x9f,0xff,0xff,0xff,0xc7,0xe7,0x83,0x81,0xe3,0x03,0xc3,0x01,0x87,0x83,0xff,0xff,0xf3,0xff,0x9f,0x83,0xb3,0xc7,0x39,0xf3,0xc3,0x3f,0x9f,0x39,0x3b,0x39,0xcf,0xcf,0xe7,0xff,0xcf,0x01,0x39,0xe7,0xf1,0xe7,0x93,0x03,0x3f,0xf3,0x1b,0x39,0xcf,0xcf,0xcf,0x01,0xe7,0x39,0x39,0xe7,0xc3,0xc3,0x33,0xf9,0x03,0xe7,0x87,0x81,0xff,0xff,0x9f,0xff,0xf3,0xf3,0x39,0xe7,0x87,0xf9,0x01,0xf9,0x39,0xcf,0x61,0xf9,0xcf,0xcf,0xcf,0x01,0xe7,0xc7,0x9b,0xe7,0x1f,0x39,0xf3,0x39,0x39,0xcf,0x79,0xf3,0xcf,0xcf,0xe7,0xff,0xcf,0xff,0xc7,0x81,0x01,0x83,0xf3,0x83,0x83,0xcf,0x83,0x87,0xff,0x9f,0xf3,0xff,0x9f,0xc7,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x83,0xc7,0x03,0xc3,0x07,0x01,0x01,0xc1,0x39,0x81,0xf9,0x39,0x9f,0x39,0x39,0x83,0x7d,0x93,0x39,0x99,0x33,0x3f,0x3f,0x9f,0x39,0xe7,0xf9,0x33,0x9f,0x11,0x19,0x39,0x45,0x39,0x39,0x3f,0x39,0x3f,0x3f,0x3f,0x39,0xe7,0xf9,0x27,0x9f,0x01,0x09,0x39,0x55,0x39,0x03,0x3f,0x39,0x03,0x03,0x31,0x01,0xe7,0xf9,0x0f,0x9f,0x01,0x01,0x39,0x41,0x01,0x39,0x3f,0x39,0x3f,0x3f,0x39,0x39,0xe7,0xf9,0x07,0x9f,0x29,0x21,0x39,0x7f,0x39,0x39,0x99,0x33,0x3f,0x3f,0x99,0x39,0xe7,0x39,0x23,0x9f,0x39,0x31,0x39,0x83,0x39,0x03,0xc3,0x07,0x01,0x3f,0xc1,0x39,0x81,0x83,0x31,0x81,0x39,0x39,0x83,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x03,0x83,0x03,0x87,0x81,0x39,0x39,0x39,0x39,0x99,0x01,0xc3,0x7f,0x87,0xc7,0xff,0x39,0x39,0x39,0x33,0xe7,0x39,0x39,0x39,0x11,0x99,0xf1,0xcf,0xbf,0xe7,0x93,0xff,0x39,0x39,0x39,0x3f,0xe7,0x39,0x39,0x29,0x83,0x99,0xe3,0xcf,0xdf,0xe7,0xff,0xff,0x39,0x39,0x31,0x83,0xe7,0x39,0x11,0x01,0xc7,0xc3,0xc7,0xcf,0xef,0xe7,0xff,0xff,0x03,0x21,0x07,0xf9,0xe7,0x39,0x83,0x01,0x83,0xe7,0x8f,0xcf,0xf7,0xe7,0xff,0xff,0x3f,0x33,0x23,0x39,0xe7,0x39,0xc7,0x11,0x11,0xe7,0x1f,0xcf,0xfb,0xe7,0xff,0xff,0x3f,0x85,0x31,0x83,0xe7,0x83,0xef,0x39,0x39,0xe7,0x01,0xc3,0xfd,0x87,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x01,0xef,0xff,0x3f,0xff,0xf9,0xff,0xf1,0xff,0x3f,0xe7,0xf3,0x3f,0xc7,0xff,0xff,0xff,0xf7,0xff,0x3f,0xff,0xf9,0xff,0xe7,0xff,0x3f,0xff,0xff,0x3f,0xe7,0xff,0xff,0xff,0xff,0x83,0x03,0x81,0x81,0x83,0x81,0x81,0x03,0xc7,0xe3,0x31,0xe7,0x03,0x03,0x83,0xff,0xf9,0x39,0x3f,0x39,0x39,0xe7,0x39,0x39,0xe7,0xf3,0x03,0xe7,0x49,0x39,0x39,0xff,0x81,0x39,0x3f,0x39,0x01,0xe7,0x39,0x39,0xe7,0xf3,0x07,0xe7,0x49,0x39,0x39,0xff,0x39,0x39,0x3f,0x39,0x3f,0xe7,0x81,0x39,0xe7,0xf3,0x23,0xe7,0x49,0x39,0x39,0xff,0x81,0x83,0x81,0x81,0x83,0xe7,0xf9,0x39,0x81,0xf3,0x31,0x81,0x49,0x39,0x83,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x83,0xff,0xff,0x87,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xe7,0xff,0xff,0xff,0xff,0xff,0xff,0xf3,0xe7,0x9f,0xff,0xff,0xff,0xff,0xff,0xff,0xe7,0xff,0xff,0xff,0xff,0xff,0xff,0xe7,0xe7,0xcf,0xff,0xff,0x03,0x81,0x91,0x83,0x81,0x39,0x99,0x49,0x39,0x39,0x01,0xe7,0xe7,0xcf,0x8f,0xff,0x39,0x39,0x8f,0x3f,0xe7,0x39,0x99,0x49,0x01,0x39,0xe3,0xcf,0xe7,0xe7,0x45,0xff,0x39,0x39,0x9f,0x83,0xe7,0x39,0x99,0x49,0xc7,0x39,0xc7,0xe7,0xe7,0xcf,0xe3,0xff,0x03,0x81,0x9f,0xf9,0xe7,0x39,0xc3,0x49,0x01,0x81,0x8f,0xe7,0xe7,0xcf,0xff,0x93,0x3f,0xf9,0x9f,0x03,0xe7,0x81,0xe7,0x81,0x39,0xf9,0x01,0xf3,0xe7,0x9f,0xff,0x93,0x3f,0xf9,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x83,0xff,0xff,0xff,0xff,0xff,0xff,0x83,0x83,0xff,0xff,0x83,0x83,0x83,0x83,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x29,0x39,0xff,0xff,0x11,0x11,0x11,0x11,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x29,0x09,0xff,0xff,0x21,0x09,0x39,0x11,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x11,0x11,0xff,0xff,0x7d,0x7d,0x55,0x55,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x29,0x21,0xff,0xff,0x21,0x09,0x11,0x39,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x29,0x39,0xff,0xff,0x11,0x11,0x11,0x11,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x83,0x83,0xff,0xff,0x83,0x83,0x83,0x83,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xe7,0xef,0xc3,0xff,0x99,0xe7,0xc3,0x93,0xc3,0x87,0xff,0xff,0xff,0xc3,0x83,0xff,0xff,0x83,0x99,0xa5,0x99,0xe7,0x99,0xff,0xbd,0xc3,0xc9,0xff,0xff,0xbd,0xff,0xff,0xe7,0x29,0x9f,0xdb,0xc3,0xe7,0x87,0xff,0x66,0x93,0x93,0x81,0xff,0x46,0xff,0xff,0xe7,0x2f,0x03,0xdb,0x81,0xff,0xdb,0xff,0x5e,0xc3,0x27,0xf9,0xff,0x5a,0xff,0xff,0xc7,0x29,0x9f,0xdb,0xe7,0xe7,0xe1,0xff,0x5e,0xff,0x93,0xf9,0xff,0x46,0xff,0xff,0xc7,0x83,0x9f,0xa5,0x81,0xe7,0x99,0xff,0x66,0xff,0xc9,0xff,0xff,0x5a,0xff,0xff,0xc7,0xef,0x01,0xff,0xe7,0xe7,0xc3,0xff,0xbd,0xff,0xff,0xff,0xff,0xbd,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xc3,0xff,0xff,0xff,0xff,0xc3,0xff,0xef,0xe7,0xc7,0xc3,0xf7,0xff,0xc1,0xff,0xff,0xe7,0xc7,0xff,0xbd,0xbd,0x1d,0xc7,0xd7,0xe7,0xf3,0xe7,0xef,0xff,0x95,0xff,0xff,0xc7,0x93,0x27,0x3b,0x3b,0xbb,0xff,0xef,0x81,0xe7,0xf3,0xff,0x33,0xb5,0xff,0xff,0xe7,0x93,0x93,0xb7,0xb7,0xd7,0xc7,0xff,0xe7,0xc3,0xc7,0xff,0x33,0x95,0xcf,0xff,0xc3,0xc7,0xc9,0xad,0xa9,0x2d,0x9f,0xff,0xe7,0xff,0xff,0xff,0x33,0xc1,0xcf,0xff,0xff,0xff,0x93,0xd9,0xdd,0xd9,0x39,0xff,0xff,0xff,0xff,0xff,0x33,0xf5,0xff,0xff,0xff,0xff,0x27,0xb1,0xbb,0xb1,0x01,0xff,0x81,0xff,0xff,0xff,0x09,0xf5,0xff,0xf7,0xff,0xff,0xff,0x7d,0x71,0x7d,0x83,0xff,0xff,0xff,0xff,0xff,0x3f,0xff,0xff,0xcf,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xdf,0xf7,0xc7,0xcb,0x93,0xef,0xc1,0xc3,0xdf,0xf7,0xc7,0x93,0xef,0xf7,0xe7,0x99,0xef,0xef,0x93,0xa7,0xff,0xd7,0x87,0x99,0xef,0xef,0x93,0xff,0xf7,0xef,0xc3,0xff,0xc7,0xc7,0xc7,0xc7,0xc7,0xc7,0x27,0x3f,0x01,0x01,0x01,0x01,0x81,0x81,0x81,0x81,0x93,0x93,0x93,0x93,0x93,0x93,0x21,0x3f,0x3f,0x3f,0x3f,0x3f,0xe7,0xe7,0xe7,0xe7,0x39,0x39,0x39,0x39,0x39,0x39,0x07,0x99,0x03,0x03,0x03,0x03,0xe7,0xe7,0xe7,0xe7,0x01,0x01,0x01,0x01,0x01,0x01,0x27,0xc3,0x3f,0x3f,0x3f,0x3f,0xe7,0xe7,0xe7,0xe7,0x39,0x39,0x39,0x39,0x39,0x39,0x21,0xf7,0x01,0x01,0x01,0x01,0x81,0x81,0x81,0x81,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xcf,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x87,0xcb,0xdf,0xf7,0xc7,0xcb,0x93,0xff,0x83,0xdf,0xf7,0xc7,0x93,0xf7,0x3f,0xc3,0x93,0xa7,0xef,0xef,0x93,0xa7,0xff,0xbb,0x39,0xef,0xef,0x93,0xff,0xef,0x03,0x99,0x99,0x19,0x83,0x83,0x83,0x83,0x83,0xd7,0x31,0x39,0x39,0xff,0x39,0x99,0x39,0x99,0x09,0x09,0x39,0x39,0x39,0x39,0x39,0xef,0x29,0x39,0x39,0x39,0x39,0x99,0x39,0x93,0x99,0x01,0x39,0x39,0x39,0x39,0x39,0xd7,0x19,0x39,0x39,0x39,0x39,0xc3,0x39,0x99,0x93,0x21,0x39,0x39,0x39,0x39,0x39,0xbb,0x39,0x39,0x39,0x39,0x39,0xe7,0x03,0x89,0x87,0x31,0x83,0x83,0x83,0x83,0x83,0xff,0x83,0x83,0x83,0x83,0x83,0xe7,0x3f,0x93,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xdf,0xf7,0xc7,0xcb,0x93,0xef,0xff,0xff,0xdf,0xf7,0xc7,0x93,0xdf,0xf7,0xc7,0x93,0xef,0xef,0x93,0xa7,0xff,0xd7,0xff,0xff,0xef,0xef,0x93,0xff,0xef,0xef,0x93,0xff,0x83,0x83,0x83,0x83,0x83,0x83,0x83,0x81,0x83,0x83,0x83,0x83,0xff,0xff,0xff,0xc7,0xf9,0xf9,0xf9,0xf9,0xf9,0xf9,0xe9,0x3f,0x39,0x39,0x39,0x39,0xc7,0xc7,0xc7,0xe7,0x81,0x81,0x81,0x81,0x81,0x81,0x81,0x3f,0x01,0x01,0x01,0x01,0xe7,0xe7,0xe7,0xe7,0x39,0x39,0x39,0x39,0x39,0x39,0x2f,0x81,0x3f,0x3f,0x3f,0x3f,0xe7,0xe7,0xe7,0xe7,0x81,0x81,0x81,0x81,0x81,0x81,0x83,0xf7,0x83,0x83,0x83,0x83,0x81,0x81,0x81,0x81,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xcf,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x9b,0xcb,0xdf,0xf7,0xc7,0xcb,0x93,0xff,0xff,0xdf,0xf7,0xc7,0x93,0xf7,0x3f,0x93,0x87,0xa7,0xef,0xef,0x93,0xa7,0xff,0xe7,0xff,0xef,0xef,0x93,0xff,0xef,0x3f,0xff,0x67,0x03,0x83,0x83,0x83,0x83,0x83,0xff,0x83,0x39,0x39,0xff,0x39,0x39,0x03,0x39,0x83,0x39,0x39,0x39,0x39,0x39,0x39,0x81,0x31,0x39,0x39,0x39,0x39,0x39,0x39,0x39,0x39,0x39,0x39,0x39,0x39,0x39,0x39,0xff,0x29,0x39,0x39,0x39,0x39,0x39,0x39,0x39,0x39,0x39,0x39,0x39,0x39,0x39,0x39,0xe7,0x19,0x39,0x39,0x39,0x39,0x81,0x03,0x81,0x83,0x39,0x83,0x83,0x83,0x83,0x83,0xff,0x83,0x81,0x81,0x81,0x81,0xf9,0x3f,0xf9,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x83,0x3f,0x83 ];


fn get_sprite_pixel_draw_color<T: Source<u8>>(sprite: &T, fmt: PixelFormat, x: i32, y: i32, width: i32) -> u8 {
    let pixel_index = width * y + x;
    match fmt {
        PixelFormat::Blit1BPP => {
            let mut byte = sprite.item_at((pixel_index >> 3) as usize);
            byte = byte >> (7-(pixel_index & 0x07));
            byte & 0x01
        },
        PixelFormat::Blit2BPP => {
            let mut byte = sprite.item_at((pixel_index >> 2) as usize);
            byte = byte >> (6-((pixel_index & 0x03) << 1));
            byte & 0x03
        },
        _ => panic!("invalid pixel format for reading sprite data")
    }
}

trait Screen {
    type Framebuffer: Source<u8> + Sink<u8>;
    const WIDTH: u32;
    const HEIGHT: u32;
    fn fb(&self) -> &Self::Framebuffer;
    fn fb_mut(&mut self) -> &mut Self::Framebuffer;
}

struct Wasm4Screen<'a, B: Sink<u8> + Source<u8>> {
    fb: &'a mut B
}

impl <'a, B: Sink<u8> + Source<u8>> Screen for Wasm4Screen<'a, B> {
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
    fb: [u8; N]
}

impl <const N: usize, const W: u32> std::fmt::Debug for ArrayScreen<N,W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("ArrayScreen(WIDTH:{},HEIGHT:{}). framebuffer:\n", Self::WIDTH, Self::HEIGHT))?;
        for n in 0..Self::HEIGHT as usize{
            let start = n   *Self::WIDTH as usize/4;
            let end =  (n+1)*Self::WIDTH as usize/4;
            let line = as_fb_line(&self.fb()[start..end]);
            f.write_str(&line)?;
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl <const N: usize, const W: u32> ArrayScreen<N,W> {
    fn new() -> ArrayScreen<N,W> {
        ArrayScreen::<N,W> {
            fb: [0u8; N]
        }
    }

    fn new_with_fb_lines(fb_lines: &[Vec<u8>]) -> ArrayScreen<N,W>{
        let mut s = Self::new();
        for n in 0..fb_lines.len() {
            let line = &fb_lines[n];
            let start = n*Self::WIDTH as usize/4;
            let end = (n+1)*Self::WIDTH as usize/4;
            s.fb_mut()[start..end].copy_from_slice(line.as_slice());
        };
        s
    }
}

impl <const N: usize, const W: u32> Screen for ArrayScreen<N, W> {
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



fn set_pixel<T: Source<u8> + Sink<u8>>(fb: &mut T, x: i32, y: i32, color: u8) {
    let mut screen = Wasm4Screen { fb };
    set_pixel_impl(&mut screen, x, y, color)
}

fn set_pixel_impl<S: Screen>(s: &mut S, x: i32, y: i32, color: u8) {
    let idx: usize = (S::WIDTH as usize * y as usize + x as usize) >> 2;
    let shift = (x & 0x3) << 1;
    let mask = 0x3 << shift;

    let fb_byte = s.fb().item_at(idx);
    s.fb_mut().set_item_at(idx, (color << shift) | (fb_byte & !mask));
}

fn set_pixel_unclipped<T: Source<u8> + Sink<u8>>(fb: &mut T, x: i32, y: i32, color: u8) {
    let mut screen = Wasm4Screen { fb };
    set_pixel_unclipped_impl(&mut screen, x, y, color)
}

fn set_pixel_unclipped_impl<S: Screen>(s: &mut S, x: i32, y: i32, color: u8) {
    if x >= 0 && x < S::WIDTH as i32 && y >= 0 && y < S::HEIGHT as i32 {
        set_pixel_impl(s, x, y, color);
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

pub(crate) fn clear<T:Sink<u8>>(fb: &mut T) {
    fb.fill(0u8);
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
            if let Some(color) = remap_draw_color(draw_color_idx, draw_colors) {
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
    let sprite =            as_b2_vec(0b_10_11_11_10__u8);
    let mut fb =      as_fb_vec(0b_00_00_00_00_00_00_00_00__u16);

    // as a result, half the bits are written into the each side of the
    // fb
    let expected_fb = as_fb_vec(0b_00_00_10_11_11_10_00_00__u16);

    blit_sub(&mut fb, &sprite, 2, 0, 4, 1, 0, 0, 8, BLIT_2BPP, draw_colors);

    assert_eq!(as_fb_line(&fb), as_fb_line(&expected_fb))
}

/// create a Vec<u8> of framebuffer data from pixels from an integer literal
fn as_fb_vec<T>(n: T) -> Vec<u8>
where T: PrimInt + Unsigned
{
    as_pix_vec(n, PixelFormat::Framebuffer)
}

/// create a Vec<u8> of 1BPP sprite data from pixels from an integer literal
fn as_b1_vec<T>(n: T) -> Vec<u8>
where T: PrimInt + Unsigned
{
    as_pix_vec(n, PixelFormat::Blit1BPP)
}

/// create a Vec<u8> of 2BPP sprite data from pixels from an integer literal
fn as_b2_vec<T>(n: T) -> Vec<u8>
where T: PrimInt + Unsigned
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
where T: PrimInt + Unsigned
{
    let (pix_size, reverse_pixel_order) = match fmt {
        PixelFormat::Blit1BPP => (1, false),
        PixelFormat::Blit2BPP => (2, false),
        PixelFormat::Framebuffer => (2, true),
    };

    let mut v = Vec::with_capacity(size_of::<T>());
    let mask = T::from(0xff>>(8-pix_size)).unwrap();
    for i in 0..size_of::<T>() {
        
        let mut b = 0u8;
        for j in 0..(8/pix_size) {
            b = b << pix_size;
            let shift = if reverse_pixel_order {
                j*pix_size
            } else {
                8-(j+1)*pix_size
            };
            let pix = n.shr(i*8 + shift);
            b |= pix.bitand(mask).to_u8().unwrap();
        }
        v.insert(0, b);
    }

    v
}

#[test]
fn test_as_fb_vec() {
    // happy case, coming from u8
    assert_eq!(vec![0b_10_01_01_10_u8], as_fb_vec(0b_10_01_01_10_u8));

    // u16 into a vec of 2 x u8 - the two tests are equivalent, but
    // shows we can write it also as 0x as well as a 0b literal
    assert_eq!(vec![0b_11_00_01_11_u8, 0b_10_01_01_10_u8], as_fb_vec(0b_11_01_00_11__10_01_01_10_u16));
}

#[test]
fn test_as_b1_vec() {
    assert_eq!(vec![0b__0__0__0__0__1__1__1__0_u8], as_b1_vec(0b__0__0__0__0__1__1__1__0_u8))
}

#[test]
fn test_as_b2_vec() {
    assert_eq!(vec![0b_11_01_00_11_u8, 0b_10_01_01_10_u8], as_b1_vec(0b_11_01_00_11__10_01_01_10_u16))
}

#[test]
fn test_blit_sub_atlas() {

    let draw_colors = 0x4321;

    let src_x = 3;
    let src_end_x = 8;
    let src_y = 0;
    let width = src_end_x - src_x;
    let height = 1;
    let sprite =      as_b2_vec(0b_00_00_00_01_10_11_01_10_00_00_00_00_00_00_00_00_u32);
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

    let expected_fb = as_fb_vec(0b_10_10_10_01_10_11_01_10_10_10_10_10_10_10_10_10__u32);

    blit_sub(&mut fb, &sprite, 3, 0, width, height, src_x, src_y, stride, BLIT_2BPP, draw_colors);

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
fn remap_draw_colors(sprite_word: u32, sprite_word_pixels: i32, draw_colors: u16) -> (u32, u32){
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

#[test]
fn test_remap_draw_colors() {
    assert_eq!((0b01_00_10_00, 0b00_11_00_00), remap_draw_colors(0b11100001, 4, 0x2013));
}

// see https://github.com/aduros/wasm4/blob/main/runtimes/native/src/framebuffer.c
// who in turn took it from https://github.com/nesbox/TIC-80/blob/master/src/core/draw.c
pub(crate) fn line<T: Source<u8> + Sink<u8>>(
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
    line_stroke(&mut Wasm4Screen { fb }, stroke_color, x1, y1, x2, y2);
}


fn line_stroke<T: Screen>(
    screen: &mut T,
    stroke_color: u8,
    mut x1: i32,
    mut y1: i32,
    mut x2: i32,
    mut y2: i32,
) {
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

pub(crate) fn hline<T: Source<u8> + Sink<u8>>(
    fb: &mut T,
    draw_colors: u16,
    x: i32,
    y: i32,
    len: u32
) {
    if let Some(stroke) = remap_draw_color(DRAW_COLOR_1, draw_colors) {
        hline_stroke(&mut Wasm4Screen { fb }, stroke, x, y, len);
    }
}

fn hline_stroke<T: Screen>(
    screen: &mut T,
    stroke: u8,
    x: i32,
    y: i32,
    len: u32
) {
    if y < 0 || y > T::HEIGHT as i32 {
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

pub(crate) fn vline<T: Source<u8> + Sink<u8>>(
    fb: &mut T,
    draw_colors: u16,
    x: i32,
    y: i32,
    len: u32
) {
    if let Some(stroke) = remap_draw_color(DRAW_COLOR_1, draw_colors) {
        vline_stroke(&mut Wasm4Screen { fb }, stroke, x, y, len);
    }
}

fn vline_stroke<T: Screen>(
    screen: &mut T,
    stroke: u8,
    x: i32,
    y: i32,
    len: u32
) {
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

pub(crate) fn rect<T: Source<u8> + Sink<u8>>(
    fb: &mut T,
    draw_colors: u16,
    x: i32,
    y: i32,
    width: u32,
    height: u32
){
    let mut screen = Wasm4Screen { fb };
    if let Some(fill_stroke) = remap_draw_color(DRAW_COLOR_1, draw_colors) {
        let fx = x;
        let flen = width;
        for fy in y..y+(height as i32){
            hline_stroke(&mut screen, fill_stroke, fx, fy, flen)
        }
    }
    if let Some(line_stroke) = remap_draw_color(DRAW_COLOR_2, draw_colors) {
        hline_stroke(&mut screen, line_stroke, x, y, width);
        hline_stroke(&mut screen, line_stroke, x, y+(height as i32)-1, width);
        vline_stroke(&mut screen, line_stroke, x, y, height);
        vline_stroke(&mut screen, line_stroke, x+(width as i32)-1, y, height);
    }
}

pub(crate) fn oval<T: Sink<u8> + Source<u8>>(
    fb: &mut T,
    draw_colors: u16,
    x: i32,
    y: i32,
    width: u32,
    height: u32
){
    let mut screen = Wasm4Screen { fb };
    oval_impl(&mut screen, draw_colors, x, y, width, height)
}

/// Draw axis parallel ellipse centered around `x` and `y` with given `width` and
/// `height`. The algorithm aligns with what is implemented in W4's framebuffer.c
fn oval_impl<T: Screen>(
        screen: &mut T,
        draw_colors: u16,
        x: i32,
        y: i32,
        width: u32,
        height: u32
    ){
    
    let width = width as i32;
    let height = height as i32;

    // name variables like in the Wikipedia article. 
    let a = width  - 1;
    let b = height - 1;

    let b0 = b % 2;

    let a2 = a*a;
    let b2 = b*b;
    let mut dx = 4 * (1-a ) * b2;
    let mut dy = 4 * (1+b0) * a2;
    let mut err = dx + dy + b0 * a2;

    let fill_opaque: Option<u8> = remap_draw_color(DRAW_COLOR_1, draw_colors);
    let line_opaque: Option<u8> = remap_draw_color(DRAW_COLOR_2, draw_colors);

    // x1, x2 start on the left and right maxima of the horizontal axis
    let mut x1 = x;
    let mut x2 = x + width-1;
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

        let e2 = 2*err;

        if e2 <= dy {
            // filled ellipse with fill color
            if let Some(fill_stroke) = fill_opaque {
                let len = (x2-x1-1) as u32;
                hline_stroke(screen, fill_stroke, x1+1, y2, len);
                hline_stroke(screen, fill_stroke, x1+1, y1, len);
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
        set_pixel_unclipped_impl(screen, x1-1, y1, line_opaque.unwrap_or(0));
        set_pixel_unclipped_impl(screen, x2+1, y1, line_opaque.unwrap_or(0));
        set_pixel_unclipped_impl(screen, x1-1, y2, line_opaque.unwrap_or(0));
        set_pixel_unclipped_impl(screen, x2+1, y2, line_opaque.unwrap_or(0));
        y2 += 1;
        y1 -= 1;
    }

}

#[test]
fn test_oval_small_circular() {
    // 8x5 pixels, with 4 pix/byte, that's 2 bytes/row, 10 bytes in total
    let mut screen = ArrayScreen::<10,8>::new();
    
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
    let mut screen = ArrayScreen::<6,8>::new();
    
    let expected = ArrayScreen::new_with_fb_lines(&[
        as_fb_vec(0b_00_00_00_11_11_00_00_00__u16),
        as_fb_vec(0b_11_11_11_11_11_11_11_11__u16),
        as_fb_vec(0b_00_00_00_11_11_00_00_00__u16),
    ]);
    
    oval_impl(&mut screen, 0x0040, 0, 0, 8, 3);
    println!("{:?}", &screen);
    assert_eq!(screen, expected);
}

pub fn text<T: Source<u8> + Sink<u8>>(fb: &mut T, text: &[u8], x: i32, y: i32, draw_colors: u16) {
    let (mut tx, mut ty) = (x, y);

    for c in text {
        match c {
            10 => { // line feed
                ty += 8;
                tx = x;
            },
            32..=255 => {
                // weird. this is what w4 is doing...
                let src_x =  (((c-32) & 0x0f) * 8) as u32;
                let src_y =    (((c-32) >> 4) * 8) as u32;
                
                blit_sub(fb, &Vec::from(CHARSET), tx, ty, 8, 8, src_x, src_y, CHARSET_WIDTH, CHARSET_FLAGS, draw_colors);
                tx += 8;
            },
            _ => {
                tx += 8;
            }
        }
    }
}

fn as_fb_line(v: &[u8]) -> String {
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
