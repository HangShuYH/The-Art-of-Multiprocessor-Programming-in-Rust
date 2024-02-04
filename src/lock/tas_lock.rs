use std::sync::atomic::{AtomicBool, Ordering};

use super::lock::RawLock;
pub struct TASLock {
    flag: AtomicBool,
}
impl Default for TASLock {
    fn default() -> Self {
        TASLock {
            flag: AtomicBool::new(false),
        }
    }
}
unsafe impl Send for TASLock {}
unsafe impl Sync for TASLock {}
impl RawLock for TASLock {
    fn lock(&self) {
        while self.flag.fetch_or(true, Ordering::Acquire) {
            std::hint::spin_loop();
        }
    }
    fn unlock(&self) {
        self.flag.store(false, Ordering::Release);
    }
}
pub struct TASLockGuard {}
#[cfg(test)]
pub mod tests {

    use crate::lock::lock::test_lock;

    use super::TASLock;

    #[test]
    fn test_tas_lock() {
        test_lock::<TASLock>("TASLock");
    }
}
