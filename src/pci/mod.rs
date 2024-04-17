use spin::Once;
use x86_64::instructions::port::Port;

use crate::sync::spinlock::{SpinLock, SpinLockGuard};

// Reference: https://wiki.osdev.org/PCI

static PCI: Once<SpinLock<Pci>> = Once::new();

pub fn get_pci<'a>() -> SpinLockGuard<'a, Pci> {
    PCI.get().unwrap().lock()
}

/// Initializes the PCI instance. This can only be called once.
pub fn init_pci() {
    let pci = Pci::new();
    PCI.call_once(|| SpinLock::new(pci));
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// Pci header.
pub struct PciHeader {
    vendor_id: u16,
    device_id: u16,
    command: u16,
    status: u16,
    revision: u8,
    prog_if: u8,
    subclass: u8,
    class_code: u8,
    cache_line_size: u8,
    latency_timer: u8,
    header_type: u8,
    bist: u8,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// PCI device
pub enum PciDevice {
    General(GeneralDevice),
    PciPciBridge(PciHeader),
    PciCardbusBridge(PciHeader),
    Unknown(PciHeader),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// General PCI device.
pub struct GeneralDevice {
    pub header: PciHeader,
    pub bar0: u32,
    pub bar1: u32,
    pub bar2: u32,
    pub bar3: u32,
    pub bar4: u32,
    pub bar5: u32,
    pub cardbus_cis_pointer: u32,
    pub subsystem_vendor_id: u16,
    pub subsystem_device_id: u16,
    pub expansion_rom_address: u32,
    pub capabilities_pointer: u8,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub min_grant: u8,
    pub max_latency: u8,
}

/// Address used for selecting a pci device.
const CONFIG_ADDRESS: u16 = 0xCF8;
/// Address used for reading a pci device config.
const CONFIG_DATA: u16 = 0xCFC;

/// Provides functionality for interacting with PCI devices.
pub struct Pci {
    command_port: Port<u32>,
    data_port: Port<u32>,
}

#[derive(Debug)]
pub enum PciError {
    NonExistentDevice,
}

impl Pci {
    /// Creates a new of the Pci struct.
    fn new() -> Pci {
        Pci {
            command_port: Port::new(CONFIG_ADDRESS),
            data_port: Port::new(CONFIG_DATA),
        }
    }

    /// Returns a bus iterator for the provided bus id.
    pub fn bus_iterator(&mut self) -> PciBusIterator {
        PciBusIterator::new(self)
    }

    /// Reads a config for the given params.
    pub fn config_read(&mut self, bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
        let address: u32 = ((bus as u32) << 16)
            | ((slot as u32) << 11)
            | ((func as u32) << 8)
            | (offset as u32 & 0xFC)
            | (1 << 31);

        unsafe { self.command_port.write(address) }
        unsafe { self.data_port.read() }
    }

    /// Retrieves a PCI device.
    fn get_pci_device(&mut self, bus: u8, device: u8, function: u8) -> Result<PciDevice, PciError> {
        let reg0 = self.config_read(bus, device, function, 0x0);

        // Check if the device exists.
        if reg0 & 0xFFFF == 0xFFFF {
            return Err(PciError::NonExistentDevice);
        }

        let reg1 = self.config_read(bus, device, function, 0x4);
        let reg2 = self.config_read(bus, device, function, 0x8);
        let reg3 = self.config_read(bus, device, function, 0xC);

        let header = PciHeader {
            vendor_id: (reg0 & 0xFFFF) as u16,
            device_id: (reg0 >> 16 & 0xFFFF) as u16,
            command: (reg1 & 0xFFFF) as u16,
            status: (reg1 >> 16 & 0xFFFF) as u16,
            revision: (reg2 & 0xFF) as u8,
            prog_if: (reg2 >> 8 & 0xFF) as u8,
            subclass: (reg2 >> 16 & 0xFF) as u8,
            class_code: (reg2 >> 24 & 0xFF) as u8,
            cache_line_size: (reg3 & 0xFF) as u8,
            latency_timer: (reg3 >> 8 & 0xFF) as u8,
            header_type: (reg3 >> 16 & 0xFF) as u8,
            bist: (reg3 >> 24 & 0xFF) as u8,
        };

        match &header.header_type {
            // General Device.
            0x0 => {
                let reg4 = self.config_read(bus, device, function, 0x10);
                let reg5 = self.config_read(bus, device, function, 0x14);
                let reg6 = self.config_read(bus, device, function, 0x18);
                let reg7 = self.config_read(bus, device, function, 0x1C);
                let reg8 = self.config_read(bus, device, function, 0x20);
                let reg9 = self.config_read(bus, device, function, 0x24);
                let reg10 = self.config_read(bus, device, function, 0x28);
                let reg11 = self.config_read(bus, device, function, 0x2C);
                let reg12 = self.config_read(bus, device, function, 0x30);
                let reg13 = self.config_read(bus, device, function, 0x34);
                let reg15 = self.config_read(bus, device, function, 0x3C);

                Ok(PciDevice::General(GeneralDevice {
                    header,
                    bar0: reg4,
                    bar1: reg5,
                    bar2: reg6,
                    bar3: reg7,
                    bar4: reg8,
                    bar5: reg9,
                    cardbus_cis_pointer: reg10,
                    subsystem_vendor_id: (reg11 & 0xFFFF) as u16,
                    subsystem_device_id: (reg11 >> 16 & 0xFFFF) as u16,
                    expansion_rom_address: reg12,
                    capabilities_pointer: (reg13 & 0xFF) as u8,
                    interrupt_line: (reg15 & 0xFF) as u8,
                    interrupt_pin: (reg15 >> 8 & 0xFF) as u8,
                    min_grant: (reg15 >> 16 & 0xFF) as u8,
                    max_latency: (reg15 >> 24 & 0xFF) as u8,
                }))
            }
            // Pci to Pci bridge device.
            0x1 => Ok(PciDevice::PciPciBridge(header)),
            // Pci to cardbus bridge device.
            0x2 => Ok(PciDevice::PciCardbusBridge(header)),
            _ => Ok(PciDevice::Unknown(header)),
        }
    }
}
/// PCI iterator
pub struct PciBusIterator<'a> {
    pci: &'a mut Pci,
    bus: u8,
    device: u8,
    function: u8,
}

impl PciBusIterator<'_> {
    pub fn new(pci: &mut Pci) -> PciBusIterator<'_> {
        PciBusIterator {
            bus: 0,
            device: 0,
            function: 0,
            pci,
        }
    }
}

impl<'a> Iterator for PciBusIterator<'a> {
    type Item = PciDevice;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.function < 8 {
                match self
                    .pci
                    .get_pci_device(self.bus, self.device, self.function)
                {
                    Ok(device) => {
                        self.function += 1;
                        return Some(device);
                    }
                    Err(PciError::NonExistentDevice) => {
                        self.function += 1;
                    }
                }
            } else if self.device < 32 {
                self.device += 1;
                self.function = 0;
            } else if self.bus < 255 {
                self.bus += 1;
                self.device = 0;
                self.function = 0;
            } else {
                return None;
            }
        }
    }
}
