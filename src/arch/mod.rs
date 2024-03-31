use limine::{
    framebuffer::Framebuffer,
    memory_map::Entry,
    request::{
        BootloaderInfoRequest, FramebufferRequest, HhdmRequest, MemoryMapRequest, StackSizeRequest,
    },
};

pub mod gdt;

use spin::Once;

use crate::arch::gdt::init_gdt;

pub static PHYSICAL_OFFSET: Once<usize> = Once::new();

pub fn init_kernel() {
    let limine_data = init_limine();

    PHYSICAL_OFFSET.call_once(|| limine_data.physical_offset);

    init_gdt();
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
