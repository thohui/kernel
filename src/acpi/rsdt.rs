use core::marker::PhantomData;

use super::{AcpiHeader, AcpiTableKind};

#[repr(C, packed)]
///https://wiki.osdev.org/RSDT
pub struct Rsdt {
    pub header: AcpiHeader,
    pointers: (),
}

impl<'a> Rsdt {
    pub unsafe fn from_addr(addr: *const u8) -> &'static Rsdt {
        &*(addr as *const Rsdt)
    }

    pub fn iter(&self) -> RsdtIterator<'a> {
        RsdtIterator {
            index: 0,
            entries: (self.header.length as usize - core::mem::size_of::<AcpiHeader>()) / 4,
            ptr_start: &self.pointers as *const () as *const u32,
            _phantom: PhantomData,
        }
    }
    pub fn raw_iter(&self) -> RsdtRawIterator {
        RsdtRawIterator {
            index: 0,
            entries: (self.header.length as usize - core::mem::size_of::<AcpiHeader>()) / 4,
            ptr_start: &self.pointers as *const () as *const u32,
            _phantom: PhantomData,
        }
    }
}

/// Rsdt iterator
pub struct RsdtIterator<'a> {
    /// The number of acpi table entries.
    entries: usize,

    /// The start location of the table entries.
    ptr_start: *const u32,

    /// Iterator index.
    index: usize,

    _phantom: PhantomData<&'a ()>,
}

impl<'a> Iterator for RsdtIterator<'a> {
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

/// An iterator iterates over the ACPI entries, returning a tuple of the header and the ptr.
pub struct RsdtRawIterator<'a> {
    /// The number of acpi table entries.
    entries: usize,

    /// The start location of the table entries.
    ptr_start: *const u32,

    /// Iterator index.
    index: usize,

    _phantom: PhantomData<&'a ()>,
}

impl<'a> Iterator for RsdtRawIterator<'a> {
    type Item = (&'a AcpiHeader, *mut u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.entries - 1 {
            return None;
        }

        unsafe {
            let ptr = self.ptr_start.add(self.index).read_unaligned() as *mut u8;
            let header = &*(ptr as *const AcpiHeader);
            self.index += 1;

            Some((header, ptr))
        }
    }
}
