use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

use crossbeam::utils::Backoff;

use super::lock::Lock;
pub struct BackoffLock<T> {
    flag: AtomicBool,
    data: UnsafeCell<T>,
}
impl<T> Lock<T> for BackoffLock<T> {
    fn new(data: T) -> Self {
        BackoffLock {
            flag: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
    fn lock(&self) -> &mut T {
        let backoff = Backoff::new();
        loop {
            while self.flag.load(Ordering::Relaxed) {}
            if !self.flag.fetch_or(true, Ordering::Acquire) {
                break;
            } else {
                backoff.snooze();
            }
        }
        unsafe { &mut *self.data.get() }
    }
    fn unlock(&self) {
        self.flag.store(false, Ordering::Release);
    }
}

unsafe impl<T: Send> Sync for BackoffLock<T> {}
#[cfg(test)]
mod tests {
    use crate::lock::lock::test_lock;

    use super::BackoffLock;

    #[test]
    pub fn test_backoff_lock() {
        test_lock::<BackoffLock<usize>>("BackoffLock");
    }
}
