use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

/// Provides safe, cross-thread access to `T`
pub struct SpinLock<T> {
    lock: AtomicBool,
    value: UnsafeCell<T>,
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a AtomicBool,
    value: *mut T,
}

unsafe impl<T> Sync for SpinLock<T> {}
unsafe impl<T> Send for SpinLock<T> {}

impl<T> SpinLock<T> {
    /// Initializes a SpinLock.
    pub fn new(value: T) -> Self {
        SpinLock {
            lock: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    /// Locks a spinlock.
    pub fn lock(&self) -> SpinLockGuard<T> {
        while self.lock.swap(true, Ordering::Acquire) {
            core::hint::spin_loop()
        }

        SpinLockGuard {
            lock: &self.lock,
            value: self.value.get(),
        }
    }
}

impl<'a, T> Deref for SpinLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &mut *self.value }
    }
}

impl<'a, T> DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.value }
    }
}

impl<'a, T> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}
