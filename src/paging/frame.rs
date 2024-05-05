use limine::memory_map::EntryType;
use spin::Once;
use x86_64::{
    structures::paging::{
        frame::PhysFrameRangeInclusive, FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB,
    },
    PhysAddr,
};

use crate::sync::spinlock::{SpinLock, SpinLockGuard};

pub const FRAME_SIZE: usize = 4096;
const MAX_BITMAP_ENTRIES: usize = 50000;
const MAX_PHYSICAL_ADDRESS: usize = FRAME_SIZE * MAX_BITMAP_ENTRIES;

static mut BITMAP: [u8; MAX_BITMAP_ENTRIES] = [0; MAX_BITMAP_ENTRIES];

static FRAME_ALLOCATOR: Once<SpinLock<BitMapFrameAllocator<'static>>> = Once::new();

pub fn get_frame_allocator<'a>() -> SpinLockGuard<'a, BitMapFrameAllocator<'static>> {
    FRAME_ALLOCATOR.get().unwrap().lock()
}

pub fn init_allocator(memory_map: &'static [&'static limine::memory_map::Entry]) {
    FRAME_ALLOCATOR.call_once(|| SpinLock::new(BitMapFrameAllocator::new(memory_map)));
}

/// Frame allocator based on: https://shell-storm.org/blog/Physical-page-frame-allocation-with-bitmap-algorithms/
pub struct BitMapFrameAllocator<'a> {
    memory_map: &'a [&'a limine::memory_map::Entry],
}

impl BitMapFrameAllocator<'_> {
    pub fn new<'a>(memory_map: &'a [&limine::memory_map::Entry]) -> BitMapFrameAllocator<'a> {
        BitMapFrameAllocator { memory_map }
    }

    fn frame_number(&self, frame: &PhysFrame) -> usize {
        frame.start_address().as_u64() as usize / FRAME_SIZE
    }

    pub fn is_used(&self, frame: &PhysFrame) -> bool {
        let num = self.frame_number(frame);
        unsafe { (BITMAP[num / 8] >> (7 - num % 8) & 1) == 1 }
    }

    pub fn mark_used(&self, frame: &PhysFrame) {
        assert!((frame.start_address().as_u64() as usize) < MAX_PHYSICAL_ADDRESS);

        let num = self.frame_number(frame);

        unsafe { BITMAP[num / 8] |= 1 << (7 - num % 8) }
    }

    pub fn mark_unused(&self, frame: &PhysFrame) {
        assert!((frame.start_address().as_u64() as usize) < MAX_PHYSICAL_ADDRESS);
        let num = self.frame_number(frame);
        unsafe { BITMAP[num / 8] &= !(1 << (7 - num % 8)) }
    }

    pub fn mark_range_used(&self, range: PhysFrameRangeInclusive<Size4KiB>) {
        // TODO: This can be optimized.
        range.into_iter().for_each(|f| {
            self.mark_used(&f);
        })
    }

    pub fn mark_range_unused(&self, range: PhysFrameRangeInclusive<Size4KiB>) {
        // TODO: This can be optimized.
        range.into_iter().for_each(|f| {
            self.mark_unused(&f);
        });
    }
}

unsafe impl<'a> FrameAllocator<Size4KiB> for BitMapFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frames = self
            .memory_map
            .iter()
            .filter(|e| e.entry_type == EntryType::USABLE)
            .map(|e| e.base..e.base + e.length)
            .flat_map(|r| r.step_by(FRAME_SIZE))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)));

        for frame in frames {
            if !self.is_used(&frame) {
                self.mark_used(&frame);
                return Some(frame);
            }
        }

        None
    }
}

impl FrameDeallocator<Size4KiB> for BitMapFrameAllocator<'_> {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        self.mark_unused(&frame);
    }
}

pub fn stringify_entry_type(entry_type: EntryType) -> &'static str {
    let type_id = unsafe { core::mem::transmute::<EntryType, u64>(entry_type) };
    match type_id {
        0 => "usable",
        1 => "reserved",
        2 => "acpi reclaimable",
        3 => "acpi nvs",
        4 => "bad memory",
        5 => "bootloader reclaimable",
        6 => "kernel and modules",
        7 => "framebuffer",
        _ => "unknown",
    }
}
