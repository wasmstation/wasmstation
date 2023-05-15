#![no_std]

use core::{arch::wasm32, panic::PanicInfo};

#[no_mangle]
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[panic_handler]
fn phandler(_: &PanicInfo<'_>) -> ! {
    wasm32::unreachable()
}
