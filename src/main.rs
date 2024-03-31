#![no_std]
#![no_main]

mod arch;
mod io;

use core::panic::PanicInfo;

use arch::init_kernel;
use x86_64::instructions::hlt;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init_kernel();
    serial_println!("hello world!");

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
