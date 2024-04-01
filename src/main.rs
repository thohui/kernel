#![no_std]
#![no_main]

mod arch;
mod display;
mod io;
mod sync;

use core::panic::PanicInfo;

use arch::init_kernel;
use display::font::FONT;
use x86_64::instructions::hlt;

use crate::{arch::DISPLAY, display::Color};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init_kernel();

    let mut display = DISPLAY.get().unwrap().lock();

    for y in 0..display.height {
        for x in 0..display.width {
            display.draw_pixel(x, y, Color::new(20, 223, 229));
        }
    }

    let text = "hello world";
    for (idx, char) in text.chars().enumerate() {
        let x = FONT.glyph_size().0.checked_mul(idx as u64).unwrap_or(0);
        display.draw_character(x, 0, char);
    }

    loop {
        hlt();
    }
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    serial_println!("{:?}", info);
    loop {
        hlt();
    }
}
