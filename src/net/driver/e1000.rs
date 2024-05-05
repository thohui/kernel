use x86_64::{PhysAddr, VirtAddr};

use crate::{
    paging::mapper::convert_to_virtual,
    pci::{DeviceAddr, GeneralDevice, Pci, PciDevice},
    serial_println,
};

/// Vendor ID for Intel
pub const INTEL_VENDOR: u16 = 0x8086;
/// Device ID for the e1000 Qemu, Bochs, and VirtualBox emmulated NICs
pub const E1000_DEVICE: u16 = 0x100E;
#[allow(dead_code)]
/// Device ID for Intel I217
pub const INTEL_I217: u16 = 0x153A;
#[allow(dead_code)]
/// Device ID for Intel 82577LM
pub const INTEL_82577LM: u16 = 0x10EA;

#[allow(dead_code)]
pub struct E1000Driver {
    register_base_addr: VirtAddr,
}

impl E1000Driver {
    pub fn init(pci: &mut Pci) -> Result<E1000Driver, ()> {
        let pci_result: Option<(DeviceAddr, GeneralDevice)> =
            pci.bus_iterator().find_map(|(addr, device)| {
                if let PciDevice::General(device) = device {
                    if device.header.vendor_id == INTEL_VENDOR
                        && device.header.device_id == E1000_DEVICE
                    {
                        return Some((addr, device));
                    }
                };
                None
            });

        if let Some((addr, device)) = pci_result {
            pci.enable_bus_mastering(addr.bus, addr.slot, addr.function);
            pci.enable_mmio(addr.bus, addr.slot, addr.function);
            let base_addr = PhysAddr::new((device.bar0 >> 4) as u64);
            let driver = E1000Driver {
                register_base_addr: convert_to_virtual(base_addr),
            };
            return Ok(driver);
        }
        Err(())
    }
}
