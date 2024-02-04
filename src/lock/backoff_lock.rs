use std::sync::atomic::{AtomicBool, Ordering};

use crossbeam::utils::Backoff;

use super::lock::RawLock;
pub struct BackoffLock {
    flag: AtomicBool,
}
impl Default for BackoffLock {
    fn default() -> Self {
        BackoffLock {
            flag: AtomicBool::new(false),
        }
    }
}
unsafe impl Send for BackoffLock {}
unsafe impl Sync for BackoffLock {}
impl RawLock for BackoffLock {
    fn lock(&self) {
        let backoff = Backoff::new();
        loop {
            while self.flag.load(Ordering::Relaxed) {}
            if !self.flag.fetch_or(true, Ordering::Acquire) {
                break;
            } else {
                backoff.snooze();
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

    use super::BackoffLock;

    #[test]
    pub fn test_backoff_lock() {
        test_lock::<BackoffLock>("BackoffLock");
    }
}
