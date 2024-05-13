use alloc::vec::Vec;
use x86_64::{PhysAddr, VirtAddr};

use crate::{
    paging::mapper::convert_to_virtual,
    pci::{DeviceAddr, GeneralDevice, Pci, PciCapability, PciDevice},
    serial_println,
};

use super::NetworkDriver;

/// Intel Vendor id
pub const INTEL_VENDOR: u16 = 0x8086;

/// Device ID for Intel 82577L (e1000e)
pub const INTEL_82577L: u16 = 0x100e;

#[allow(dead_code)]
pub struct E1000Driver {
    register_base_addr: VirtAddr,
}

impl E1000Driver {
    pub fn init(pci: &mut Pci) -> Result<E1000Driver, ()> {
        let e1000_device: Option<(DeviceAddr, GeneralDevice)> =
            pci.bus_iterator().find_map(|(addr, device)| {
                if let PciDevice::General(device) = device {
                    if device.header.vendor_id == INTEL_VENDOR
                        && device.header.device_id == INTEL_82577L
                    {
                        serial_println!(
                            "vendor {:x} device id {:x} status: {}",
                            device.header.vendor_id,
                            device.header.device_id,
                            device.header.status
                        );
                        return Some((addr, device));
                    }
                };
                None
            });

        // Check if we found a compatible NIC.
        if let Some((addr, device)) = e1000_device {
            // Check if this device has capabilities enabled.
            // if device.header.status >> 4 & 1 != 1 {
            //     return Err(());
            // }

            // Check if the device is MSI capable, return an error otherwise.
            // let cap = unsafe {
            //     let msi_cap = pci
            //         .device_capabilities(addr.bus, addr.slot, addr.function)
            //         .find(|(cap, _)| *cap == PciCapability::Msi);

            //     if let Some(cap) = msi_cap {
            //         cap
            //     } else {
            //         return Err(());
            //     }
            // };

            // Enable bus mastering.
            pci.enable_bus_mastering(addr.bus, addr.slot, addr.function);

            let result =
                lai::pci_route_pin(0, addr.bus, addr.slot, addr.function, device.interrupt_pin)
                    .unwrap();

            serial_println!("{:?}", result);

            // Enable memory mapped i/o
            pci.enable_mmio(addr.bus, addr.slot, addr.function);

            // Get base addr.
            let base_addr = PhysAddr::new((device.bar0 >> 4) as u64);

            let driver = E1000Driver {
                register_base_addr: convert_to_virtual(base_addr),
            };

            return Ok(driver);
        }
        Err(())
    }
}

impl NetworkDriver for E1000Driver {}
