use core::{marker::PhantomData, mem, panic};

use alloc::slice;
use spin::Once;

use crate::sync::spinlock::{SpinLock, SpinLockGuard};

use self::{fadt::Fadt, madt::Madt};

pub mod fadt;
pub mod madt;

static ACPI: Once<SpinLock<Acpi>> = Once::new();

pub fn get_acpi() -> SpinLockGuard<'static, Acpi<'static>> {
    ACPI.get().unwrap().lock()
}

/// ACPI
pub struct Acpi<'a> {
    rsdt: &'a Rsdt,
}

impl<'a> Acpi<'a> {
    pub fn iter(&self) -> AcpiIterator<'a> {
        // Calculate the amount of rsdt entries.
        let entries = (self.rsdt.header.length as usize - core::mem::size_of::<AcpiHeader>()) / 4;

        let ptr_start = &self.rsdt.pointers as *const () as *const u32;

        AcpiIterator {
            entries,
            ptr_start,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

pub unsafe fn init_acpi(rsdp_address: *const u8) {
    // Get rsdp.
    let rsdp = Rsdp::from_addr(rsdp_address);
    // Get rsdt.
    let rsdt = Rsdt::from_addr(rsdp.rsdt_address);
    ACPI.call_once(|| SpinLock::new(Acpi { rsdt }));
}

/// https://wiki.osdev.org/RSDP
#[repr(C, packed)]
#[derive(Debug)]
pub struct Rsdp {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: *const u8,
}

impl Rsdp {
    /// Attempts to parse a Rsdp
    pub unsafe fn from_addr<'a>(rsdp_addr: *const u8) -> &'a Rsdp {
        let rsdp = &*(rsdp_addr as *const Rsdp);

        // Validate rsdp signature
        if &rsdp.signature != b"RSD PTR " {
            panic!("invalid rsdp signature");
        }

        // Before the RSDP is relied upon you should check that the checksum is valid.
        // For ACPI 1.0 (the first structure) you add up every byte in the structure and make sure the lowest byte of the result is equal to zero.
        let bytes = unsafe { slice::from_raw_parts(rsdp_addr, mem::size_of::<Rsdp>()) };
        let checksum: u8 = *bytes.iter().last().unwrap();

        if checksum != 0 {
            panic!("checksum check failed");
        }

        rsdp
    }
}

#[repr(C, packed)]
#[derive(Debug)]
/// Acpi Header
pub struct AcpiHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: u64,
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

#[repr(C, packed)]
///https://wiki.osdev.org/RSDT
pub struct Rsdt {
    header: AcpiHeader,
    pointers: (),
}

impl<'a> Rsdt {
    pub unsafe fn from_addr(addr: *const u8) -> &'static Rsdt {
        &*(addr as *const Rsdt)
    }

    pub fn iter(&self) -> AcpiIterator<'a> {
        AcpiIterator {
            index: 0,
            entries: (self.header.length as usize - core::mem::size_of::<AcpiHeader>()) / 4,
            ptr_start: &self.pointers as *const () as *const u32,
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug)]
pub enum AcpiTableKind<'a> {
    Fadt(&'a Fadt),
    Ssdt,
    Madt(&'a Madt),
    Unknown(&'a AcpiHeader),
}

impl<'a> AcpiTableKind<'a> {
    /// Tries to parse an ACPI header.
    pub unsafe fn try_parse(header: &AcpiHeader) -> Result<AcpiTableKind<'a>, ()> {
        match &header.signature {
            b"FACP" => {
                let addr = header as *const AcpiHeader as *const ();
                let fadt = Fadt::from_addr(addr);
                Ok(AcpiTableKind::Fadt(fadt))
            }
            b"SSDT" => Ok(AcpiTableKind::Ssdt),
            b"APIC" => {
                let addr = header as *const AcpiHeader as *const ();
                let madt = Madt::from_addr(addr);
                Ok(AcpiTableKind::Madt(madt))
            }
            _ => Err(()),
        }
    }
}

/// Rsdt Iterator.
pub struct AcpiIterator<'a> {
    /// The number of acpi table entries.
    entries: usize,

    /// The start location of the table entries.
    ptr_start: *const u32,

    /// Iterator index.
    index: usize,

    _phantom: PhantomData<&'a ()>,
}

impl<'a> Iterator for AcpiIterator<'a> {
    type Item = AcpiTableKind<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.entries - 1 {
            return None;
        }

        let ptr = unsafe { self.ptr_start.add(self.index).read_unaligned() as *mut u8 };
        let header = unsafe { &*(ptr as *const AcpiHeader) };
        let table = unsafe { AcpiTableKind::try_parse(header) };
        self.index += 1;

        if let Ok(kind) = table {
            return Some(kind);
        }

        return Some(AcpiTableKind::Unknown(header));
    }
}
