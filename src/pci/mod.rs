use core::{
    mem,
    ptr::{self, addr_of},
};

use alloc::vec::Vec;
use spin::Once;
use x86_64::PhysAddr;

pub mod bar;

use crate::{
    io::port::Port,
    sync::spinlock::{SpinLock, SpinLockGuard},
};

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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// Pci header.
pub struct PciHeader {
    /// Vendor ID.
    pub vendor_id: u16,
    /// Device ID.
    pub device_id: u16,
    ///Provides control over a device's ability to generate and respond to PCI cycles. Where the only functionality guaranteed to be supported by all devices is, when a 0 is written to this register, the device is disconnected from the PCI bus for all accesses except Configuration Space access
    pub command: u16,
    /// A register used to record status information for PCI bus related events
    pub status: u16,
    ///Specifies a revision identifier for a particular device. Where valid IDs are allocated by the vendor
    pub revision_id: u8,
    /// Prog IF(Programming Interface Byte): A read-only register that specifies a register-level programming interface the device has, if it has any at all.
    pub prog_if: u8,
    /// A read-only register that specifies the specific function the device performs
    pub subclass: u8,
    /// A read-only register that specifies the type of function the device performs
    pub class_code: u8,
    /// Specifies the system cache line size in 32-bit units. A device can limit the number of cacheline sizes it can support, if a unsupported value is written to this field, the device will behave as if a value of 0 was written.
    pub cache_line_size: u8,
    /// Specifies the latency timer in units of PCI bus clocks
    pub latency_timer: u8,
    /// Header Type.
    pub header_type: u8,
    /// Built in self test.
    pub bist: u8,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// PCI device
pub enum PciDevice {
    /// General device.
    General(GeneralDevice),
    /// PCI To PCI bridge device.
    PciPciBridge(PciHeader),
    /// PCI to card bus bridge device.
    PciCardbusBridge(PciHeader),
    /// Unknown device.
    Unknown(PciHeader),
}

impl PciDevice {
    pub fn to_string<'a>(&self) -> &'a str {
        match self {
            PciDevice::General(_) => "General",
            PciDevice::PciPciBridge(_) => "PCI to PCI bridge",
            PciDevice::PciCardbusBridge(_) => "Pci cardbus bridge",
            PciDevice::Unknown(_) => "Unknown",
        }
    }
}

// TODO: create bar struct for handling BARs.

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// General PCI device.
pub struct GeneralDevice {
    pub header: PciHeader,
    /// Base address 0
    pub bar0: u32,
    /// Base address 1
    pub bar1: u32,
    /// Base address 2
    pub bar2: u32,
    /// Base address 3
    pub bar3: u32,
    /// Base address 4
    pub bar4: u32,
    /// Base address 5
    pub bar5: u32,
    /// Points to the Card Information Structure and is used by devices that share silicon between CardBus and PCI
    pub cardbus_cis_pointer: u32,
    ///Sub system vendor id.
    pub subsystem_vendor_id: u16,
    /// Sub system device id.
    pub subsystem_device_id: u16,
    /// Expansion rom address.
    pub expansion_rom_address: u32,
    /// Points (offset) to a linked list of new capabilities implemented by the device. Used if bit 4 of the status register is set.
    pub capabilities_pointer: u8,
    /// Specifies which input of the system interrupt controllers the device's interrupt pin is connected to and is implemented by any device that makes use of an interrupt pin
    pub interrupt_line: u8,
    /// Specifies which interrupt pin the device uses. Where a value of 0x1 is INTA#, 0x2 is INTB#, 0x3 is INTC#, 0x4 is INTD#, and 0x0 means the device does not use an interrupt pin
    pub interrupt_pin: u8,
    /// A read-only register that specifies the burst period length, in 1/4 microsecond units, that the device needs (assuming a 33 MHz clock rate)
    pub min_grant: u8,
    /// A read-only register that specifies how often the device needs access to the PCI bus (in 1/4 microsecond units)
    pub max_latency: u8,
}

#[derive(Debug)]
pub struct MsiCapabilities {
    /// Capability ID
    pub capability_id: u8,

    /// Next Pointer
    pub next_pointer: u8,

    /// Message Control
    pub message_control: u8,

    // Message address (Low)
    pub message_addr_low: u32,

    // Message address (High)
    pub message_addr_high: u32,

    // Mesage Data
    pub message_data: u8,

    // Mask
    pub mask: u32,

    // Pending
    pub pending: u32,
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

    /// Selects a config register.
    fn select_config(&mut self, bus: u8, slot: u8, function: u8, offset: u8) {
        // Get address.
        let address: u32 = ((bus as u32) << 16)
            | ((slot as u32) << 11)
            | ((function as u32) << 8)
            | (offset as u32 & 0xFC)
            | (1 << 31);
        unsafe { self.command_port.write(address) };
    }

    /// Reads a config register.
    pub fn config_read(&mut self, bus: u8, slot: u8, function: u8, offset: u8) -> u32 {
        self.select_config(bus, slot, function, offset);
        unsafe { self.data_port.read() }
    }

    /// Writes to a config register.
    pub fn config_write(&mut self, bus: u8, slot: u8, function: u8, offset: u8, value: u32) {
        self.select_config(bus, slot, function, offset);
        unsafe { self.data_port.write(value) };
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
            revision_id: (reg2 & 0xFF) as u8,
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

    pub fn enable_bus_mastering(&mut self, bus: u8, slot: u8, function: u8) {
        let value = self.config_read(bus, slot, function, 0x4);
        self.config_write(bus, slot, function, 0x4, value | (1 << 2))
    }

    pub fn enable_mmio(&mut self, bus: u8, slot: u8, function: u8) {
        let value = self.config_read(bus, slot, function, 0x4);
        self.config_write(bus, slot, function, 0x4, value | (1 << 1))
    }

    /// Returns an iterator over the pci capabilities.
    /// It is up to the function caller to make sure that the has capabilities in the first place (by checking the status register).
    pub unsafe fn device_capabilities(
        &mut self,
        bus: u8,
        slot: u8,
        function: u8,
    ) -> PciCapabilityIterator<'_> {
        let cap_ptr = (self.config_read(bus, slot, function, 0x34) & 0xFF) as u8;
        PciCapabilityIterator::new(self, bus, slot, function, cap_ptr, 16)
    }
}

/// Iterator that iterates over every bus, every device and every function.
pub struct PciBusIterator<'a> {
    pci: &'a mut Pci,
    bus: u8,
    slot: u8,
    function: u8,
}

impl PciBusIterator<'_> {
    pub fn new(pci: &mut Pci) -> PciBusIterator<'_> {
        PciBusIterator {
            bus: 0,
            slot: 0,
            function: 0,
            pci,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
// pub struct DeviceAddr(u8, u8, u8);
pub struct DeviceAddr {
    pub bus: u8,
    pub slot: u8,
    pub function: u8,
}

impl DeviceAddr {
    pub fn new(bus: u8, slot: u8, function: u8) -> Self {
        Self {
            bus,
            slot,
            function,
        }
    }
}

impl<'a> Iterator for PciBusIterator<'a> {
    type Item = (DeviceAddr, PciDevice);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.function < 8 {
                match self.pci.get_pci_device(self.bus, self.slot, self.function) {
                    Ok(device) => {
                        let addr = DeviceAddr::new(self.bus, self.slot, self.function);
                        self.function += 1;
                        return Some((addr, device));
                    }
                    Err(PciError::NonExistentDevice) => {
                        self.function += 1;
                    }
                }
            } else if self.slot < 32 {
                self.slot += 1;
                self.function = 0;
            } else if self.bus < 255 {
                self.bus += 1;
                self.slot = 0;
                self.function = 0;
            } else {
                return None;
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum PciCapability {
    Msi,
    Unknown,
}

/// Iterator that iterates over the PCI caps.
pub struct PciCapabilityIterator<'a> {
    // Pci
    pci: &'a mut Pci,

    /// The bus id of the PCI device.
    bus: u8,

    /// The slot of the PCI device.
    slot: u8,

    /// The function of the PCI device.
    function: u8,

    /// The pointer to the next capability.
    ptr: u8,

    // The number of attempts (iterations) left.
    attempts_left: u8,
}

impl PciCapabilityIterator<'_> {
    pub fn new<'a>(
        pci: &'a mut Pci,
        bus: u8,
        slot: u8,
        function: u8,
        capabilities_ptr: u8,
        max_attempts: u8,
    ) -> PciCapabilityIterator<'a> {
        PciCapabilityIterator {
            pci,
            bus,
            slot,
            function,
            attempts_left: max_attempts,
            ptr: capabilities_ptr,
        }
    }
}

impl<'a> Iterator for PciCapabilityIterator<'a> {
    type Item = (PciCapability, u8);

    fn next(&mut self) -> Option<Self::Item> {
        // Check if we have reached the max amount of attempts.
        if self.attempts_left == 0 {
            return None;
        }

        // Clear reserved bits.
        let ptr = self.ptr & !0x3;

        // Read register.
        let reg = self
            .pci
            .config_read(self.bus, self.slot, self.function, ptr);

        // Extract capability ID.
        let capability_id = (reg & 0xFF) as u8;

        let capability: PciCapability = match capability_id {
            0x5 => PciCapability::Msi,
            _ => PciCapability::Unknown,
        };

        // Setup next pointer.
        self.ptr = (reg >> 8 & 0xFF) as u8;

        self.attempts_left -= 1;

        Some((capability, ptr))
    }
}
