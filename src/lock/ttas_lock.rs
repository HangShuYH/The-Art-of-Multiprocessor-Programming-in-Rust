use std::sync::atomic::{AtomicBool, Ordering};

pub struct TTASLock {
    flag: AtomicBool,
}
impl TTASLock {
    pub fn new() -> Self {
        TTASLock {
            flag: AtomicBool::new(false),
        }
    }
    pub fn lock(&self) {
        loop {
            while self.flag.load(Ordering::Relaxed) {}
            if !self.flag.fetch_or(true, Ordering::Acquire) {
                break;
            }
        }
    }
    pub fn unlock(&self) {
        self.flag.store(false, Ordering::Release);
    }
}
#[cfg(test)]
mod tests {

    use std::{sync::Arc, thread, time::Instant};

    use super::TTASLock;

    #[test]
    fn test_ttas_lock() {
        let n = 10;
        let step = 1000000;
        static mut VALUE: usize = 0;
        let lock = Arc::new(TTASLock::new());
        let start = Instant::now();
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
        let duration = start.elapsed();
        println!("TTASLock Time elapsed: {:?}", duration);
    }
}
