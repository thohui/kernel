use core::mem;

use alloc::vec::Vec;
use x86_64::VirtAddr;

use crate::{
    io::port::Port,
    paging::mapper::convert_to_physical,
    pci::{bar::Bar, Pci, PciDevice},
    serial_println,
};

pub const VENDOR_ID: u16 = 0x10ec;
pub const DEVICE_ID: u16 = 0x8139;

#[repr(C)]
struct RxDescriptor {
    flags: u32,
    vlan: u32,
    low_buf: u32,
    high_buf: u32,
}

const RX_BUFFER_SIZE: usize = 8192 / mem::size_of::<RxDescriptor>();

pub struct Rtl8139Driver {
    io: Port<u32>,
    rx_descriptors: Vec<Option<RxDescriptor>>,
}

impl Rtl8139Driver {
    pub fn init(pci: &mut Pci) -> Result<Self, ()> {
        let (device_addr, device) = pci
            .bus_iterator()
            .find_map(|(device_addr, device)| {
                if let PciDevice::General(device) = device {
                    if device.header.vendor_id == VENDOR_ID && device.header.device_id == DEVICE_ID
                    {
                        return Some((device_addr, device));
                    }
                };
                None
            })
            .unwrap();

        let msi = device.header.status >> 3 & 1;
        serial_println!("msi support: {:?}, status: {}", msi, device.header.status);
        // Check bar0 type.
        if let Ok(Bar::Io { address }) = Bar::parse(device.bar0) {
            // Enable bus mastering.
            pci.enable_bus_mastering(device_addr.bus, device_addr.slot, device_addr.function);

            let mut inner = Self {
                io: Port::new(address),
                rx_descriptors: (0..RX_BUFFER_SIZE).map(|_| None).collect(),
            };

            // Turn on the device.
            inner.io.write_offset(0x52, 0x0);

            // Reset the device.
            inner.io.write_offset(0x37, 0x10);

            // Block until the chip has finished resetting.
            while inner.io.read_offset(0x37) & 0x10 != 0 {}
            serial_println!("finished!");

            // Init receive buffer.
            let ptr = inner.rx_descriptors.as_ptr_range().start;
            let virt_addr = VirtAddr::from_ptr(ptr);

            serial_println!("{:?}", virt_addr);
            let physical_addr = convert_to_physical(virt_addr).unwrap();

            inner.io.write_offset(0x30, physical_addr.as_u64() as u32);

            return Ok(inner);
        }

        Err(())
    }

    fn software_reset(&mut self) {
        self.io.write_offset(0x52, 0x0)
    }
}
