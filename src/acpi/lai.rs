use core::ptr;

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    sync::Arc,
};

use crate::{
    io::port::PortReadWrite, paging::mapper::convert_to_virtual_raw, pci::get_pci, serial_println,
};

use super::{get_acpi, AcpiTableKind};
struct LaiHost;

impl lai::Host for LaiHost {
    fn scan(&self, _signature: &str, _index: usize) -> *const u8 {
        serial_println!("scan: {}", _signature);
        if _signature == "DSDT" {
            let fadt = get_acpi()
                .rsdt
                .iter()
                .find_map(|kind| {
                    if let AcpiTableKind::Fadt(fadt) = kind {
                        return Some(fadt);
                    }
                    None
                })
                .unwrap();
            convert_to_virtual_raw(fadt.dsdt.into()).as_ptr::<u8>()
        } else {
            let result = get_acpi()
                .rsdt
                .raw_iter()
                .find(|e| e.0.signature == _signature.as_bytes())
                .map(|e| e.0 as *const _ as *const u8)
                .unwrap_or(ptr::null_mut::<u8>());
            result
        }
    }

    fn sleep(&self, _ms: u64) {
        // unimplemented!()
    }

    fn outb(&self, _port: u16, _value: u8) {
        unsafe { u8::write_port(_port, _value) };
    }

    fn outw(&self, _port: u16, _value: u16) {
        unsafe { u16::write_port(_port, _value) };
    }

    fn outd(&self, _port: u16, _value: u32) {
        unsafe { u32::write_port(_port, _value) };
    }

    fn inb(&self, _port: u16) -> u8 {
        unsafe { u8::read_port(_port) }
    }

    fn inw(&self, _port: u16) -> u16 {
        unsafe { u16::read_port(_port) }
    }

    fn ind(&self, _port: u16) -> u32 {
        unsafe { u32::read_port(_port) }
    }

    fn pci_readb(&self, _seg: u16, _bus: u8, _slot: u8, _fun: u8, _offset: u16) -> u8 {
        serial_println!("readb {:x}", _offset);
        get_pci().config_read(_bus, _slot, _fun, _offset as u8) as u8
    }

    fn pci_readw(&self, _seg: u16, _bus: u8, _slot: u8, _fun: u8, _offset: u16) -> u16 {
        serial_println!("readw");
        let data = get_pci().config_read(_bus, _slot, _fun, _offset as u8) as u16;
        data
    }

    fn pci_readd(&self, _seg: u16, _bus: u8, _slot: u8, _fun: u8, _offset: u16) -> u32 {
        serial_println!("readd");
        get_pci().config_read(_bus, _slot, _fun, _offset as u8) as u32
    }

    fn map(&self, _address: usize, _count: usize) -> *mut u8 {
        serial_println!("map");
        convert_to_virtual_raw(_address as u64).as_mut_ptr()
    }
}

pub fn init_lai() {
    let lai_host = Arc::new(LaiHost);
    lai::init(lai_host);
    lai::set_acpi_revision(get_acpi().rsdt.header.revision as _);
    lai::create_namespace();
    lai::enable_acpi(1);
}
