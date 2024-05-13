#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(exposed_provenance)]
#![feature(strict_provenance)]

mod acpi;
mod apic;
mod arch;
mod display;
mod io;
mod memory;
mod net;
mod paging;
mod pci;
mod pic;
mod sync;

use core::panic::PanicInfo;

use acpi::{get_acpi, AcpiTableKind};
use alloc::string::{String, ToString};
use arch::init_kernel;
use display::get_display;
use net::driver::e1000::{E1000Driver, INTEL_VENDOR};
use pci::get_pci;
use x86_64::instructions::hlt;

use crate::display::Color;

extern crate alloc;

#[no_mangle]

pub extern "C" fn _start() -> ! {
    init_kernel();

    E1000Driver::init(&mut get_pci()).expect("Could not initialize e1000 driver");

    let mut display = get_display();

    for y in 0..display.height {
        for x in 0..display.width {
            display.draw_pixel(x, y, Color::new(20, 223, 229));
        }
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
