use lazy_static::lazy_static;
use x86_64::{
    registers::segmentation::{Segment, CS, DS},
    structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
};

pub struct Segments {
    kernel_code_selector: SegmentSelector,
    kernel_data_selector: SegmentSelector,
}

lazy_static! {
    pub static ref GDT: (GlobalDescriptorTable, Segments) = {
        let mut gdt = GlobalDescriptorTable::new();

        let kernel_code_selector = gdt.append(Descriptor::kernel_code_segment());
        let kernel_data_selector = gdt.append(Descriptor::kernel_data_segment());

        (
            gdt,
            Segments {
                kernel_code_selector,
                kernel_data_selector,
            },
        )
    };
}

pub fn init_gdt() {
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.kernel_code_selector);
        DS::set_reg(GDT.1.kernel_data_selector);
    }
}
