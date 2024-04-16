use spin::Once;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{OffsetPageTable, PageTable},
    PhysAddr, VirtAddr,
};

use crate::sync::spinlock::{SpinLock, SpinLockGuard};

static PAGE_MAPPER: Once<SpinLock<OffsetPageTable>> = Once::new();

pub fn get_page_mapper<'a>() -> SpinLockGuard<'a, OffsetPageTable<'static>> {
    PAGE_MAPPER.get().unwrap().lock()
}

static PHYSICAL_OFFSET: Once<u64> = Once::new();

pub fn init_mapper(offset: u64) {
    PHYSICAL_OFFSET.call_once(|| offset);
    PAGE_MAPPER.call_once(|| unsafe {
        SpinLock::new(OffsetPageTable::new(
            active_page_table(),
            VirtAddr::new(offset),
        ))
    });
}

pub fn convert_to_virtual(addr: PhysAddr) -> VirtAddr {
    VirtAddr::new(addr.as_u64() + PHYSICAL_OFFSET.get().unwrap())
}

pub unsafe fn active_page_table() -> &'static mut PageTable {
    let (frame, _) = Cr3::read();
    let phys_addr = frame.start_address();
    let virt_addr = convert_to_virtual(phys_addr);
    let page_table_ptr: *mut PageTable = virt_addr.as_mut_ptr();
    &mut *page_table_ptr
}
