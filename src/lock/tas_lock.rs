use std::sync::atomic::{AtomicBool, Ordering};
pub struct TASLock {
    flag: AtomicBool,
}
impl TASLock {
    pub fn new() -> TASLock {
        TASLock {
            flag: AtomicBool::new(false),
        }
    }
    pub fn lock(&self) {
        while self.flag.fetch_or(true, Ordering::SeqCst) {}
    }
    pub fn unlock(&self) {
        self.flag.store(false, Ordering::SeqCst);
    }
}
#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};

    use super::TASLock;

    #[test]
    fn test_tas_lock() {
        let n = 10;
        let step = 1000000;
        static mut VALUE: usize = 0;
        let lock = Arc::new(TASLock::new());
        let threads: Vec<_> = (0..n)
            .map(|_| {
                let lock = Arc::clone(&lock);
                thread::spawn(move || {
                    for _ in 0..step {
                        lock.lock();
                        unsafe {
                            VALUE = VALUE + 1;
                        }
                        lock.unlock()
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        unsafe { assert_eq!(VALUE, n * step) };
    }
}
