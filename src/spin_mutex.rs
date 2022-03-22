use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct SpinlockMutex<T> {
    value: UnsafeCell<T>,
    lock: AtomicBool
}

impl<T> SpinlockMutex<T> {
    pub fn new(value: T) -> Self {
        SpinlockMutex {value: UnsafeCell::from(value), lock: AtomicBool::new(false)}
    }

    pub fn spinlock(&self) -> SpinlockMutexGuard<T> {
        unsafe {
            loop {
                match self.lock.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed) {
                    Ok(false) | Err(false) => {
                        return SpinlockMutexGuard { borrowed_value: &mut *self.value.get() as &mut T, lock: &self.lock }
                    }
                    Err(true) | Ok(true) => {}
                }
                spin_loop();
            }
        }
    }
}

pub struct SpinlockMutexGuard<'a, T> {
    borrowed_value: &'a mut T,
    lock: &'a AtomicBool
}

impl<T> Drop for SpinlockMutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
    }
}

impl<T> Deref for SpinlockMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.borrowed_value
    }
}

impl<T> DerefMut for SpinlockMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.borrowed_value
    }
}