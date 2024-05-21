use core::arch::x86_64::_rdtsc;

// TODO: actually calculate this.
/// CPU frequency in hz.
pub const CPU_FREQUENCY: u64 = 3_000_000_000;

/// Sleeps for the amount of provided cycles.
#[inline]
pub unsafe fn sleep_cycles(cycles: u64) {
    let start = _rdtsc();
    while (_rdtsc() - start) < cycles {
        core::hint::spin_loop()
    }
}

/// Sleeps for the amount of provided ms.
#[inline]
pub unsafe fn sleep_ms(ms: u64) {
    let cycles = (ms * CPU_FREQUENCY) / 1000;
    sleep_cycles(cycles)
}
