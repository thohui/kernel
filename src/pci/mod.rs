use bit_field::BitField;
use spin::Once;
use x86_64::instructions::port::Port;

use crate::{serial_println, sync::spinlock::SpinLock};

static PCI: Once<SpinLock<Pci>> = Once::new();

pub fn init_pci() {
    let pci = Pci::new();
    PCI.call_once(|| SpinLock::new(pci));
    PCI.get().unwrap().lock().scan_bus(0);
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
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

#[allow(dead_code)]
impl PciHeader {
    pub fn has_multiple_fns(&self) -> bool {
        self.header_type.get_bit(7)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ConversionError {
    UnknownHeaderType,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ConvertedDevice {
    // TODO: expand on this.
    General,
    PciPciBridge,
    PciCardbusBridge,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
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
    pub capabilities: u8,
    pub reserved: [u8; 7],
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub min_grant: u8,
    pub max_latency: u8,
}

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

pub struct Pci {
    command_port: Port<u32>,
    data_port: Port<u32>,
}

#[derive(Debug)]
pub enum PciError {
    NonExistentDevice,
}

impl Pci {
    pub fn new() -> Pci {
        Pci {
            command_port: Port::new(CONFIG_ADDRESS),
            data_port: Port::new(CONFIG_DATA),
        }
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

    /// Attempts to retrieve a PCI header for the given params.
    fn get_pci_device(&mut self, bus: u8, device: u8, function: u8) -> Result<PciHeader, PciError> {
        let reg0 = self.config_read(bus, device, function, 0x0);

        // Check if the device exists.
        if reg0 & 0xFFFF == 0xFFFF {
            return Err(PciError::NonExistentDevice);
        }

        let reg1 = self.config_read(bus, device, function, 0x4);
        let reg2 = self.config_read(bus, device, function, 0x8);
        let reg3 = self.config_read(bus, device, function, 0xC);

        Ok(PciHeader {
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
        })
    }

    pub fn convert_device(&self, header: &PciHeader) -> Result<ConvertedDevice, ConversionError> {
        // TODO: how do we handle devices that have multiple funcs?
        match header.header_type {
            0x0 => Ok(ConvertedDevice::General),
            0x1 => Ok(ConvertedDevice::PciPciBridge),
            0x2 => Ok(ConvertedDevice::PciCardbusBridge),
            _ => Err(ConversionError::UnknownHeaderType),
        }
    }

    pub fn scan_bus(&mut self, bus: u8) {
        for device in 0..=31 {
            for func in 0..=7 {
                if let Ok(device) = self.get_pci_device(bus, device, func) {
                    serial_println!("{} {:?}", device.has_multiple_fns(), device);
                }
            }
        }
    }
}
