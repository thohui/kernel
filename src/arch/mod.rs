use core::ops::{Add, DerefMut};

use ::alloc::vec;
use limine::{
    framebuffer::Framebuffer,
    memory_map::Entry,
    request::{
        BootloaderInfoRequest, FramebufferRequest, HhdmRequest, MemoryMapRequest, StackSizeRequest,
    },
};

use crate::{
    display::init_display,
    memory::heap::init_heap,
    paging::{
        frame::{init_allocator, stringify_entry_type, FRAME_ALLOCATOR},
        mapper::{init_mapper, PAGE_MAPPER},
    },
};

pub mod gdt;
pub mod idt;

use crate::{arch::gdt::init_gdt, serial_println};

use self::idt::init_idt;

pub fn init_kernel() {
    let limine_data = init_limine();

    init_gdt();
    init_idt();
    init_allocator(limine_data.memory_map);
    init_mapper(limine_data.physical_offset as u64);

    let mut mapper = PAGE_MAPPER.get().unwrap().lock();
    let mut allocator = FRAME_ALLOCATOR.get().unwrap().lock();

    init_heap(mapper.deref_mut(), allocator.deref_mut());

    init_display(limine_data.framebuffer);

    for entry in limine_data.memory_map {
        let entry_type = stringify_entry_type(entry.entry_type);
        serial_println!(
            "{} from 0x{:X} to 0x{:X}",
            entry_type,
            entry.base,
            entry.base.add(entry.length)
        );
    }
}

static STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new().with_size(4096);
static BOOTLOADER_INFO: BootloaderInfoRequest = BootloaderInfoRequest::new();
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

struct LimineData<'a> {
    physical_offset: usize,
    framebuffer: Framebuffer<'a>,
    memory_map: &'a [&'a Entry],
}

fn init_limine() -> LimineData<'static> {
    let _ = BOOTLOADER_INFO.get_response().unwrap();
    let _ = STACK_SIZE_REQUEST.get_response().unwrap();

    let hhdm_response = HHDM_REQUEST.get_response().unwrap();
    let physical_offset = hhdm_response.offset() as usize;

    let memory_map_response = MEMORY_MAP_REQUEST.get_response().unwrap();
    let memory_map = memory_map_response.entries();

    let framebuffer_response = FRAMEBUFFER_REQUEST.get_response().unwrap();
    let framebuffer = framebuffer_response.framebuffers().next().unwrap();

    LimineData {
        physical_offset,
        memory_map,
        framebuffer,
    }
}
