use crate::pci::{DeviceAddr, GeneralDevice, Pci, PciDevice};

/// Vendor ID for Intel
pub const INTEL_VENDOR: u16 = 0x8086;
/// Device ID for the e1000 Qemu, Bochs, and VirtualBox emmulated NICs
pub const E1000_DEVICE: u16 = 0x100E;
#[allow(dead_code)]
/// Device ID for Intel I217
pub const E1000_I217: u16 = 0x153A;
#[allow(dead_code)]
/// Device ID for Intel 82577LM
pub const E1000_82577LM: u16 = 0x10EA;

#[allow(dead_code)]
pub struct Driver {
    device: GeneralDevice,
    addr: DeviceAddr,
}

impl Driver {
    pub fn init(pci: &mut Pci) -> Result<Driver, ()> {
        let mut e1000_device: Option<(DeviceAddr, GeneralDevice)> = None;
        pci.bus_iterator().for_each(|(addr, device)| {
            if let PciDevice::General(device) = device {
                if device.header.vendor_id == INTEL_VENDOR
                    && device.header.device_id == E1000_DEVICE
                {
                    e1000_device = Some((addr, device));
                }
            }
        });
        if let Some((addr, device)) = e1000_device {
            pci.enable_bus_mastering(addr.bus, addr.slot, addr.function);
            return Ok(Driver { addr, device });
        }
        Err(())
    }
}
