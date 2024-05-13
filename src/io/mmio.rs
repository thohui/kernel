use core::ptr::{read_volatile, write_volatile};

// Read a value from a memory-mapped I/O address
pub unsafe fn read<T>(addr: *const T, offset: usize) -> T {
    read_volatile(addr.add(offset))
}

// Write a value to a memory-mapped I/O address
pub unsafe fn write<T>(addr: *mut T, offset: usize, value: T) {
    write_volatile(addr.add(offset), value);
}
