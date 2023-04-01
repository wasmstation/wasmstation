//! Various utility functions for WASM-4.

use byteorder::{LittleEndian, WriteBytesExt};

use crate::wasm4::FRAMEBUFFER_SIZE;

/// Returns the default WASM-4 palette.
pub fn default_palette() -> [u8; 16] {
    let mut buf = vec![];
    buf.write_u32::<LittleEndian>(0xe0f8cf).unwrap();
    buf.write_u32::<LittleEndian>(0x86c06c).unwrap();
    buf.write_u32::<LittleEndian>(0x306850).unwrap();
    buf.write_u32::<LittleEndian>(0x071821).unwrap();

    buf.try_into().expect("wrong palette size")
}

/// Returns the default WASM-4 draw colors.
pub fn default_draw_colors() -> [u8; 2] {
    bytemuck::cast_slice(&[0x1203_u16])
        .to_vec()
        .try_into()
        .expect("wrong draw colors size")
}

/// Returns an empty WASM-4 framebuffer.
pub fn default_framebuffer() -> [u8; FRAMEBUFFER_SIZE] {
    (0..6400)
        .map(|_| 0)
        .collect::<Vec<u8>>()
        .try_into()
        .expect("wrong framebuffer size")
}
