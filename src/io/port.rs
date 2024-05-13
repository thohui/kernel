use core::{arch::asm, marker::PhantomData};

pub trait PortReadWrite {
    /// Writes the value to the i/o port.
    unsafe fn write_port(port: u16, value: Self);
    // /// Writes the value to the i/o port + offset.
    // unsafe fn write_offset(&mut self, offset: u16, value: S);
    /// Reads the value from the i/o port
    unsafe fn read_port(port: u16) -> Self;
    // unsafe fn read_offset(&mut self, offset: u16) -> S;
}

/// Wrapper struct around memory i/o port access.
pub struct Port<T> {
    base: u16,
    phantom: PhantomData<T>,
}

impl<T: PortReadWrite> Port<T> {
    pub fn new(base: u16) -> Self {
        Self {
            base,
            phantom: PhantomData,
        }
    }

    /// Writes a value to the port.
    pub fn write(&mut self, value: T) {
        unsafe { T::write_port(self.base, value) };
    }

    /// Writes a value to the port + offset.
    pub fn write_offset(&mut self, offset: u16, value: T) {
        unsafe { T::write_port(self.base + offset, value) };
    }

    /// Reads a value from the port.
    pub fn read(&self) -> T {
        unsafe { T::read_port(self.base) }
    }

    /// Reads a value from the port + offset.
    pub fn read_offset(&self, offset: u16) -> T {
        unsafe { T::read_port(self.base + offset) }
    }
}

impl PortReadWrite for u8 {
    unsafe fn write_port(port: u16, value: Self) {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }

    unsafe fn read_port(port: u16) -> Self {
        let value;
        asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
        value
    }
}

impl PortReadWrite for u16 {
    unsafe fn write_port(port: u16, value: Self) {
        asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
    }

    unsafe fn read_port(port: u16) -> Self {
        let value: u16;
        asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack, preserves_flags));
        value
    }
}

impl PortReadWrite for u32 {
    unsafe fn write_port(port: u16, value: u32) {
        asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack, preserves_flags));
    }

    unsafe fn read_port(port: u16) -> u32 {
        let value: u32;
        asm!("in eax, dx", out("eax") value, in("dx") port, options(nomem, nostack, preserves_flags));
        value
    }
}
