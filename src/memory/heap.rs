use core::{
    alloc::{GlobalAlloc, Layout},
    ptr,
};

use x86_64::{
    structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB},
    VirtAddr,
};

use crate::sync::spinlock::SpinLock;

#[global_allocator]
pub static GLOBAL_ALLOCATOR: SpinLock<BumpAllocator> = SpinLock::new(BumpAllocator::new_empty());

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 1000 * 1024;

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let heap_start = VirtAddr::new(HEAP_START as u64);
    let heap_end = heap_start + HEAP_SIZE.try_into().unwrap() - 1u64;
    let heap_start_page = Page::containing_address(heap_start);
    let heap_end_page = Page::containing_address(heap_end);

    let page_range = Page::range_inclusive(heap_start_page, heap_end_page);

    for page in page_range {
        let frame = frame_allocator.allocate_frame().unwrap();
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .unwrap()
                .flush()
        };
    }

    GLOBAL_ALLOCATOR
        .lock()
        .init(HEAP_START as *mut _, HEAP_SIZE)
}

pub struct BumpAllocator {
    start: *mut u8,
    next: *mut u8,
    size: usize,
}

impl BumpAllocator {
    pub const fn new_empty() -> Self {
        Self {
            start: ptr::null::<u8>() as *mut u8,
            next: ptr::null::<u8>() as *mut u8,
            size: 0,
        }
    }

    pub fn init(&mut self, start: *mut u8, size: usize) {
        self.start = start;
        self.next = start;
        self.size = size;
    }
}

unsafe impl GlobalAlloc for SpinLock<BumpAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut inner = self.lock();
        let allocation_size = layout.size().max(layout.align());
        let aligned_address = (inner.next as usize + allocation_size - 1) & !(allocation_size - 1);

        if aligned_address > inner.start as usize + inner.size {
            return ptr::null::<u8>() as *mut u8;
        }
        let next = aligned_address + allocation_size;
        inner.next = next as *mut u8;

        aligned_address as *mut u8
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {}

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: core::alloc::Layout,
        new_size: usize,
    ) -> *mut u8 {
        let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());

        let new_ptr = self.alloc(new_layout);

        if !new_ptr.is_null() && !ptr.is_null() {
            core::ptr::copy_nonoverlapping(
                ptr,
                self.alloc(layout),
                core::cmp::min(layout.size(), new_size),
            );
            self.dealloc(ptr, layout);
        }
        new_ptr
    }
}
