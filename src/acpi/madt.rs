use core::{marker::PhantomData, mem};

use super::AcpiHeader;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Madt {
    /// Acpi Header
    header: AcpiHeader,

    /// Local APIC Address
    pub apic_addr: u32,

    /// Flags (1 = Dual 8259 Legacy PICs Installed)
    pub flags: u32,

    /// The start of the madt table entries.
    entry_start: (),
}

impl Madt {
    pub unsafe fn from_addr<'a>(addr: *const ()) -> &'a Madt {
        &*(addr as *const Madt)
    }
    /// Creates an iterator over the Madt entries.
    pub unsafe fn iter<'a>(&self) -> MadtEntryIterator<'a> {
        // get the size of the entry data.
        let size = self.header.length as usize - mem::size_of::<Madt>();

        let entry_start = &self.entry_start as *const () as *const u8;

        MadtEntryIterator::new(entry_start, size)
    }
}

pub struct MadtEntryIterator<'a> {
    /// Start of the MADT entries.
    start: *const u8,
    size: usize,

    /// The iterator index
    index: usize,

    _phantom: PhantomData<&'a ()>,
}

impl<'a> MadtEntryIterator<'a> {
    pub fn new(start: *const u8, size: usize) -> MadtEntryIterator<'a> {
        MadtEntryIterator {
            start,
            size,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

impl<'a> Iterator for MadtEntryIterator<'a> {
    type Item = MadtEntryKind<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if we finished iterating.
        if self.index >= self.size {
            return None;
        }

        // Read entry type and size
        let entry_id = unsafe { *self.start.add(self.index) };
        let entry_size = unsafe { *self.start.add(self.index + 1) };

        self.index += entry_size as usize;

        // Check if the data can still fit.
        if self.index > self.size {
            return None;
        }

        let entry_start = self.index - entry_size as usize + 2;

        match entry_id {
            0 => Some(unsafe {
                MadtEntryKind::LocalApic(&*(self.start.add(entry_start) as *const LocalApicEntry))
            }),
            1 => Some(unsafe {
                MadtEntryKind::IoApic(&*(self.start.add(entry_start) as *const IoApicEntry))
            }),
            2 => Some(unsafe {
                MadtEntryKind::IoApicInterruptSourceOverride(
                    &*(self.start.add(entry_start) as *const IoApicInterruptSourceOverrideEntry),
                )
            }),
            3 => Some(unsafe {
                MadtEntryKind::IoApicNonMaskableInterruptSource(
                    &*(self.start.add(entry_start) as *const IoApicNonMaskableInterruptSourceEntry),
                )
            }),
            4 => Some(unsafe {
                MadtEntryKind::LocalApicNonMaskableInterrupts(
                    &*(self.start.add(entry_start) as *const LocalApicNonMaskableInterruptsEntry),
                )
            }),
            5 => Some(unsafe {
                MadtEntryKind::LocalApicAddressOverride(
                    &*(self.start.add(entry_start) as *const LocalApicAddressOverrideEntry),
                )
            }),
            9 => Some(unsafe {
                MadtEntryKind::ProcessorLocalx2Apic(
                    &*(self.start.add(entry_start) as *const ProcessorLocalx2ApicEntry),
                )
            }),
            _ => None,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
#[allow(dead_code)]
/// Mad Entry Kind.
pub enum MadtEntryKind<'a> {
    LocalApic(&'a LocalApicEntry),
    IoApic(&'a IoApicEntry),
    IoApicInterruptSourceOverride(&'a IoApicInterruptSourceOverrideEntry),
    IoApicNonMaskableInterruptSource(&'a IoApicNonMaskableInterruptSourceEntry),
    LocalApicAddressOverride(&'a LocalApicAddressOverrideEntry),
    LocalApicNonMaskableInterrupts(&'a LocalApicNonMaskableInterruptsEntry),
    ProcessorLocalx2Apic(&'a ProcessorLocalx2ApicEntry),
}

#[derive(Debug)]
#[repr(C, packed)]
/// Processor Local APIC
pub struct LocalApicEntry {
    /// ACPI Processor ID
    pub acpi_processor_id: u8,
    /// APIC id.
    pub apic_id: u8,
    /// If flags bit 0 is set the CPU is able to be enabled, if it is not set you need to check bit 1. If that one is set you can still enable it, if it is not the CPU can not be enabled and the OS should not try.
    pub flags: u32,
}

#[derive(Debug)]
#[repr(C, packed)]
/// I/O APIC
pub struct IoApicEntry {
    /// IO Apic ID
    pub apic_id: u8,

    /// Reserved
    reserved: u8,

    /// IO Apic address
    pub apic_addr: u32,

    /// Global System Interrupt Base
    pub global_system_interrupt_base: u32,
}

#[derive(Debug)]
#[repr(C, packed)]
/// I/O APIC Interrupt Source Override.
///
/// This entry type explains how IRQ sources are mapped to global system interrupts.
pub struct IoApicInterruptSourceOverrideEntry {
    /// Bus source
    pub bus_source: u8,

    /// IRQ source
    pub irq_source: u8,

    /// Global System Interrupt
    pub global_system_interrupt: u32,

    /// Flags
    pub flags: u16,
}

#[derive(Debug)]
#[repr(C, packed)]
///  I/O APIC Non-maskable interrupt source
///
/// Specifies which I/O APIC interrupt inputs should be enabled as non-maskable.
pub struct IoApicNonMaskableInterruptSourceEntry {
    /// NMI source
    pub nmi_source: u8,

    /// Reserved
    reserved: u8,

    /// Flags
    pub flags: u16,

    /// Global System Interrupt
    pub global_system_interrupt: u32,
}

#[derive(Debug)]
#[repr(C, packed)]
///  Local APIC Non-maskable interrupts
pub struct LocalApicNonMaskableInterruptsEntry {
    /// ACPI Processor ID (0xFF means all processors)
    pub acpi_processor_id: u8,

    /// Flags
    pub flags: u16,

    /// Lint
    pub lint: u8,
}

#[derive(Debug)]
#[repr(C, packed)]
/// Local APIC Address Override
///
/// Provides 64 bit systems with an override of the physical address of the Local APIC. There can only be one of these defined in the MADT.
///If this structure is defined, the 64-bit Local APIC address stored within it should be used instead of the 32-bit Local APIC address stored in the MADT header.
pub struct LocalApicAddressOverrideEntry {
    /// Reserved
    reserved: u16,
    /// 64-bit physical address of Local APIC
    pub local_apic_address: u64,
}

#[derive(Debug)]
#[repr(C, packed)]
///  Local APIC Non-maskable interrupts
///
/// Represents a physical processor and its Local x2APIC.
/// Identical to Local APIC; used only when that struct would not be able to hold the required values.
pub struct ProcessorLocalx2ApicEntry {
    /// Reserved
    reserved: u16,

    /// Processor's local x2APIC ID
    pub processor_id: u32,

    /// Flags (same as the Local APIC flags)
    pub flags: u32,

    // ACPI id
    pub acpi_id: u32,
}
