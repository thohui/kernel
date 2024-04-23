#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(exposed_provenance)]

mod arch;
mod display;
mod io;
mod memory;
mod net;
mod paging;
mod pci;
mod sync;

use core::panic::PanicInfo;

use arch::init_kernel;
use display::get_display;
use net::e1000;
use pci::get_pci;
use x86_64::instructions::hlt;

use crate::display::Color;

extern crate alloc;

#[no_mangle]

pub extern "C" fn _start() -> ! {
    init_kernel();

    let mut display = get_display();

    for y in 0..display.height {
        for x in 0..display.width {
            display.draw_pixel(x, y, Color::new(20, 223, 229));
        }
    }

    e1000::Driver::init(&mut get_pci()).unwrap();

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
