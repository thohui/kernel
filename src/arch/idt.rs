use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

lazy_static! {
    pub static ref IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();
}

pub fn init_idt() {
    IDT.load();
}
