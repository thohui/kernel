// use pic8259::ChainedPics;
// use spin::Once;

// use crate::{
//     pci::GeneralDevice,
//     sync::spinlock::{SpinLock, SpinLockGuard},
// };

// pub struct Pic {
//     pub pics: ChainedPics,
// }

// pub const PIC_1_OFFSET: u8 = 32;
// pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// static PIC: Once<SpinLock<Pic>> = Once::new();

// pub fn get_pic<'a>() -> SpinLockGuard<'a, Pic> {
//     PIC.get().unwrap().lock()
// }

// pub unsafe fn init_pics() {
//     PIC.call_once(|| {
//         let mut pics = unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) };

//         SpinLock::new(Pic { pics })
//     });
// }

// pub fn get_interrupt_vector(device: &GeneralDevice) -> Option<u8> {
//     let num = device.interrupt_pin;

//     if num < 8 {
//         return Some(PIC_1_OFFSET + num);
//     } else if num < 16 {
//         return Some(PIC_2_OFFSET + num - 8);
//     }
//     None
// }
