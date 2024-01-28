use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
pub struct PetersonLock {
    flags: [AtomicBool; 2],
    victim: AtomicUsize,
}
impl PetersonLock {
    pub fn new() -> PetersonLock {
        PetersonLock {
            flags: [AtomicBool::new(false), AtomicBool::new(false)],
            victim: AtomicUsize::new(0),
        }
    }
    pub fn lock(&self, number: usize) {
        assert!(number == 0 || number == 1);
        self.flags[number].store(true, Ordering::SeqCst);
        let other = 1 - number;
        self.victim.store(number, Ordering::SeqCst);
        while self.flags[other].load(Ordering::SeqCst)
            && self.victim.load(Ordering::SeqCst) == number
        {}
    }
    pub fn unlock(&self, number: usize) {
        self.flags[number].store(false, Ordering::SeqCst);
    }
}
#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};

    use super::PetersonLock;

    #[test]
    fn test_peterson() {
        let peterson_lock = Arc::new(PetersonLock::new());
        static mut VALUE: usize = 0;
        const N: usize = 1000000;
        let threads: Vec<_> = (0..=1)
            .map(|i| {
                let peterson_lock = Arc::clone(&peterson_lock);
                thread::spawn(move || {
                    for _ in 0..N {
                        peterson_lock.lock(i);
                        unsafe {
                            VALUE = VALUE + 1;
                        }
                        peterson_lock.unlock(i);
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        unsafe { assert_eq!(VALUE, 2 * N) };
    }
}
