use byteorder::{LittleEndian, WriteBytesExt};

use crate::wasm4::FRAMEBUFFER_SIZE;

pub fn default_palette() -> [u8; 16] {
    let mut buf = vec![];
    buf.write_u32::<LittleEndian>(0xe0f8cf).unwrap();
    buf.write_u32::<LittleEndian>(0x86c06c).unwrap();
    buf.write_u32::<LittleEndian>(0x306850).unwrap();
    buf.write_u32::<LittleEndian>(0x071821).unwrap();

    buf.try_into().expect("wrong palette size")
}

pub fn default_draw_colors() -> [u8; 2] {
    bytemuck::cast_slice(&[0x1203_u16])
        .to_vec()
        .try_into()
        .expect("wrong draw colors size")
}

pub fn empty_framebuffer() -> [u8; FRAMEBUFFER_SIZE] {
    (0..6400)
        .into_iter()
        .map(|_| 0x0000)
        .collect::<Vec<u8>>()
        .try_into()
        .expect("wrong framebuffer size")
}
