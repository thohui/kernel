use core::{cell::UnsafeCell, ops::Add};

use alloc::vec::Vec;
use spin::Once;
use x86_64::{PhysAddr, VirtAddr};

use crate::{
    acpi::{
        madt::{LocalApicEntry, MadtEntryKind},
        Acpi, AcpiTableKind,
    },
    paging::mapper::convert_to_virtual,
    sync::spinlock::{SpinLock, SpinLockGuard},
};

// MSR apic base Register
const IA32_APIC_BASE_MSR: u32 = 0x1B;

/// Local APIC ID Register
const LAPIC_ID: usize = 0x0020;

/// Local APIC Version Register
const LAPIC_VER: usize = 0x0030;

/// Local APIC Task Priority Register
const LAPIC_TPR: usize = 0x0080;

/// Local APIC Arbitration Priority Register
const LAPIC_APR: usize = 0x0090;

/// Local APIC Processor Priority Register
const LAPIC_PPR: usize = 0x00a0;

/// Local APIC EOI Register
const LAPIC_EOI: usize = 0x00b0;

/// Local APIC Remote Read Register
const LAPIC_RRD: usize = 0x00c0;

/// Local APIC Logical Destination Register
const LAPIC_LDR: usize = 0x00d0;

/// Local APIC Destination Format Register
const LAPIC_DFR: usize = 0x00e0;

/// Local APIC Spurious Interrupt Vector Register
const LAPIC_SVR: usize = 0x00f0;

/// Local APIC In-Service Registers (8 registers)
const LAPIC_ISR: usize = 0x0100;

/// Local APIC Trigger Mode Registers (8 registers)
const LAPIC_TMR: usize = 0x0180;

/// Local APIC Interrupt Request Registers (8 registers)
const LAPIC_IRR: usize = 0x0200;

/// Local APIC Error Status Register
const LAPIC_ESR: usize = 0x0280;

/// Local APIC Interrupt Command Register Low
const LAPIC_ICRLO: usize = 0x0300;

/// Local APIC Interrupt Command Register High
const LAPIC_ICRHI: usize = 0x0310;

/// Local APIC LVT Timer Register
const LAPIC_TIMER: usize = 0x0320;

/// Local APIC LVT Thermal Sensor Register
const LAPIC_THERMAL: usize = 0x0330;

/// Local APIC LVT Performance Counter Register
const LAPIC_PERF: usize = 0x0340;

/// Local APIC LVT LINT0 Register
const LAPIC_LINT0: usize = 0x0350;

/// Local APIC LVT LINT1 Register
const LAPIC_LINT1: usize = 0x0360;

/// Local APIC LVT Error Register
const LAPIC_ERROR: usize = 0x0370;

/// Local APIC Initial Count Register (for Timer)
const LAPIC_TICR: usize = 0x0380;

/// Local APIC Current Count Register (for Timer)
const LAPIC_TCCR: usize = 0x0390;

/// Local APIC Divide Configuration Register (for Timer)
const LAPIC_TDCR: usize = 0x03e0;

pub struct Apic {
    local_apic_address: VirtAddr,
}

// TODO: this needs doesn't need to be inside a spinlock as it is local to the core.
// In the future, when we support multiprocessing, we can perhaps impl a core local struct that contains the local apic too.
static APIC: Once<SpinLock<Apic>> = Once::new();

pub fn get_apic<'a>() -> SpinLockGuard<'a, Apic> {
    APIC.get().unwrap().lock()
}

pub unsafe fn init_apic(acpi: &Acpi) {
    let mut local_apics: Vec<&'static LocalApicEntry> = Vec::new();
    let mut local_apic_address: u32 = 0;

    for table in acpi.rsdt.iter() {
        if let AcpiTableKind::Madt(madt) = table {
            local_apic_address = madt.apic_addr;
            for madt_entry in madt.iter() {
                if let MadtEntryKind::LocalApic(local_apic) = madt_entry {
                    local_apics.push(local_apic)
                }
            }
        }
    }

    APIC.call_once(|| {
        let physical_addr = PhysAddr::new(local_apic_address.into());
        let virtual_apic_addr = convert_to_virtual(physical_addr);

        let mut inner = Apic {
            local_apic_address: virtual_apic_addr,
        };

        inner.enable_local_apic();

        SpinLock::new(inner)
    });
}

impl Apic {
    pub unsafe fn write_register(&mut self, offset: usize, value: u32) {
        self.local_apic_address
            .add(offset as u64)
            .as_mut_ptr::<u32>()
            .write_volatile(value);
    }
    pub unsafe fn read_register(&self, offset: usize) -> u32 {
        self.local_apic_address
            .add(offset as u64)
            .as_mut_ptr::<u32>()
            .read_volatile()
    }

    // Enable local apic
    pub unsafe fn enable_local_apic(&mut self) {
        // Clear Task priority register.
        self.write_register(LAPIC_TPR, 0);

        // Configure Spurious Interrupt Vector Register
        self.write_register(LAPIC_SVR, 0x100 | 0xff);
    }
}
