use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

use rand::Rng;

use super::lock::Lock;
struct Backoff {
    min: u64,
    max: u64,
}
impl Backoff {
    pub fn new(min: u64, max: u64) -> Backoff {
        Backoff { min, max }
    }
    pub fn backoff(&mut self) {
        let mut rng = rand::thread_rng();
        let random = rng.gen_range(self.min..self.max);
        thread::sleep(Duration::from_micros(random));
        self.min = std::cmp::min(self.min * 2, self.max - 1);
    }
}
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
        let mut backoff = Backoff::new(10, 1000);
        loop {
            while self.flag.load(Ordering::Relaxed) {}
            if !self.flag.fetch_or(true, Ordering::Acquire) {
                break;
            } else {
                backoff.backoff();
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
