use std::sync::atomic::{AtomicBool, Ordering};

use super::lock::Lock;
pub struct TASLock {
    flag: AtomicBool,
}
impl Lock for TASLock {
    fn new() -> TASLock {
        TASLock {
            flag: AtomicBool::new(false),
        }
    }
    fn lock(&self) {
        while self.flag.fetch_or(true, Ordering::Acquire) {}
    }
    fn unlock(&self) {
        self.flag.store(false, Ordering::Release);
    }
}
#[cfg(test)]
mod tests {

    use crate::lock::lock::{self, Lock};

    use super::TASLock;

    #[test]
    fn test_tas_lock() {
        lock::test_lock(TASLock::new(), "TASLock");
    }
}
