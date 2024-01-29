use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

use super::lock::Lock;

pub struct TTASLock<T> {
    flag: AtomicBool,
    data: UnsafeCell<T>,
}
unsafe impl<T: Send> Sync for TTASLock<T> {}
impl<T> Lock<T> for TTASLock<T> {
    fn new(data: T) -> Self {
        TTASLock {
            flag: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
    fn lock(&self) -> &mut T {
        loop {
            while self.flag.load(Ordering::Relaxed) {}
            if !self.flag.fetch_or(true, Ordering::Acquire) {
                break;
            }
        }
        unsafe { &mut *self.data.get() }
    }
    fn unlock(&self) {
        self.flag.store(false, Ordering::Release);
    }
}
#[cfg(test)]
mod tests {

    use crate::lock::lock::test_lock;

    use super::TTASLock;

    #[test]
    fn test_ttas_lock() {
        test_lock::<TTASLock<usize>>("TTASLock");
    }
}
