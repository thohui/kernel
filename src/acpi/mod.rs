use core::{marker::PhantomData, mem, panic, str::from_utf8_unchecked};

use alloc::{slice, string::String};
use spin::Once;

use crate::sync::spinlock::{SpinLock, SpinLockGuard};

use self::{fadt::Fadt, madt::Madt, rsdp::Rsdp, rsdt::Rsdt};

pub mod fadt;
pub mod lai;
pub mod madt;
pub mod rsdp;
pub mod rsdt;

static ACPI: Once<SpinLock<Acpi>> = Once::new();

pub fn get_acpi<'a>() -> SpinLockGuard<'a, Acpi<'static>> {
    ACPI.get().unwrap().lock()
}

/// ACPI
pub struct Acpi<'a> {
    pub rsdt: &'a Rsdt,
}

pub unsafe fn init_acpi(rsdp_address: *const u8) {
    // Get rsdp.
    let rsdp = Rsdp::from_addr(rsdp_address);
    // Get rsdt.
    let rsdt = Rsdt::from_addr(rsdp.rsdt_address);

    let acpi = Acpi { rsdt };
    ACPI.call_once(|| SpinLock::new(acpi));
}

#[repr(C, packed)]
#[derive(Debug)]
/// Acpi Header
pub struct AcpiHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: u64,
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

impl AcpiHeader {
    pub fn validate_checksum(&self) -> bool {
        let bytes = unsafe {
            core::slice::from_raw_parts(
                self as *const Self as *const u8,
                core::mem::size_of::<Self>(),
            )
        };

        let mut sum: u8 = 0;
        for byte in bytes.iter() {
            sum = sum.wrapping_add(*byte);
        }
        *bytes.last().unwrap_or(&1) == 0
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum AcpiTableKind<'a> {
    Fadt(&'a Fadt),
    Madt(&'a Madt),
    Unknown(&'a AcpiHeader),
}

impl<'a> AcpiTableKind<'a> {
    /// Attempts to parse an ACPI header.
    pub unsafe fn try_parse(header: &'a AcpiHeader) -> Result<AcpiTableKind<'a>, ()> {
        match &header.signature {
            b"FACP" => {
                let addr = header as *const AcpiHeader as *const ();
                let fadt = Fadt::from_addr(addr);
                Ok(AcpiTableKind::Fadt(fadt))
            }
            b"APIC" => {
                let addr = header as *const AcpiHeader as *const ();
                let madt = Madt::from_addr(addr);
                Ok(AcpiTableKind::Madt(madt))
            }
            _ => Ok(AcpiTableKind::Unknown(header)),
        }
    }
}
