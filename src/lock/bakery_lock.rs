use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

pub struct BakeryLock {
    flags: Arc<Vec<AtomicBool>>,
    labels: Arc<Vec<AtomicUsize>>,
    n: usize,
}
impl BakeryLock {
    pub fn new(n: usize) -> BakeryLock {
        BakeryLock {
            flags: Arc::new((0..n).map(|_| AtomicBool::new(false)).collect()),
            labels: Arc::new((0..n).map(|_| AtomicUsize::new(0)).collect()),
            n,
        }
    }
    pub fn lock(&self, id: usize) {
        self.flags[id].store(true, Ordering::SeqCst);
        let max = self
            .labels
            .iter()
            .map(|item| item.load(Ordering::SeqCst))
            .max()
            .unwrap()
            + 1;
        self.labels[id].store(max, Ordering::SeqCst);
        for i in 0..self.n {
            while i != id
                && self.flags[i].load(Ordering::SeqCst)
                && (self.labels[i].load(Ordering::SeqCst) < max
                    || self.labels[i].load(Ordering::SeqCst) == max && id < i)
            {}
        }
    }
    pub fn unlock(&self, id: usize) {
        self.flags[id].store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};

    use crate::lock::bakery_lock::BakeryLock;

    #[test]
    fn test_bakery_lock() {
        const N: usize = 5;
        const STEP: usize = 1000000;
        static mut VALUE: usize = 0;
        let bakery_lock = Arc::new(BakeryLock::new(N));
        let threads: Vec<_> = (0..N)
            .map(|i| {
                let bakery_lock = Arc::clone(&bakery_lock);
                thread::spawn(move || {
                    for _ in 0..STEP {
                        bakery_lock.lock(i);
                        unsafe {
                            VALUE = VALUE + 1;
                        }
                        bakery_lock.unlock(i);
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        unsafe {
            assert_eq!(VALUE, N * STEP);
        }
    }
}
