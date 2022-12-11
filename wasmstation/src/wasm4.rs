pub const SCREEN_SIZE: u8 = 160;
pub const FRAMEBUFFER_SIZE: usize = (SCREEN_SIZE as usize * SCREEN_SIZE as usize) / 4;

pub const PALETTE_ADDR: u64 = 0x04;
pub const DRAW_COLORS_ADDR: u64 = 0x14;
pub const FRAMEBUFFER_ADDR: u64 = 0xa0;
