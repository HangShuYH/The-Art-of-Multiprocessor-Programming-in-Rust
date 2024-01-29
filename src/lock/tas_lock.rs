use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

use super::lock::Lock;

pub struct TASLock<T> {
    flag: AtomicBool,
    data: UnsafeCell<T>,
}
unsafe impl<T: Send> Sync for TASLock<T> {}
impl<T> Lock<T> for TASLock<T> {
    fn new(data: T) -> Self {
        TASLock {
            flag: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
    fn lock(&self) -> &mut T {
        while self.flag.fetch_or(true, Ordering::Acquire) {
            std::hint::spin_loop();
        }
        unsafe { &mut *self.data.get() }
    }
    fn unlock(&self) {
        self.flag.store(false, Ordering::Release);
    }
}
#[cfg(test)]
pub mod tests {

    use crate::lock::lock::test_lock;

    use super::TASLock;

    #[test]
    fn test_tas_lock() {
        test_lock::<TASLock<usize>>("TASLock");
    }
}
