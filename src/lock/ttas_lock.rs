use std::sync::atomic::{AtomicBool, Ordering};

use super::lock::RawLock;

pub struct TTASLock {
    flag: AtomicBool,
}
impl Default for TTASLock {
    fn default() -> Self {
        TTASLock {
            flag: AtomicBool::new(false),
        }
    }
}
unsafe impl Send for TTASLock {}
unsafe impl Sync for TTASLock {}
impl RawLock for TTASLock {
    fn lock(&self) {
        loop {
            while self.flag.load(Ordering::Relaxed) {
                std::hint::spin_loop();
            }
            if !self.flag.fetch_or(true, Ordering::Acquire) {
                break;
            }
        }
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
        test_lock::<TTASLock>("TTASLock");
    }
}
