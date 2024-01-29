use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

pub struct FilterLock {
    want_level: Arc<Vec<AtomicUsize>>,
    victim: Arc<Vec<AtomicUsize>>,
    n: usize,
}
impl FilterLock {
    pub fn new(n: usize) -> FilterLock {
        FilterLock {
            want_level: Arc::new((0..=n).map(|_| AtomicUsize::new(0)).collect()),
            victim: Arc::new((0..=n).map(|_| AtomicUsize::new(0)).collect()),
            n,
        }
    }
    pub fn lock(&self, id: usize) {
        for i in 1..=self.n {
            self.want_level[id].store(i, Ordering::SeqCst);
            self.victim[i].store(id, Ordering::SeqCst);
            for k in 1..=self.n {
                while k != id
                    && self.want_level[k].load(Ordering::SeqCst) >= i
                    && self.victim[i].load(Ordering::SeqCst) == id
                {}
            }
        }
    }
    pub fn unlock(&self, id: usize) {
        self.want_level[id].store(0, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread, time::Instant};

    use crate::lock::filter_lock::FilterLock;

    #[test]
    fn test_filter_lock() {
        let n = 10;
        let step = 1000000;
        static mut VALUE: usize = 0;
        let filter_lock = Arc::new(FilterLock::new(n));
        let start = Instant::now();
        let threads: Vec<_> = (0..n)
            .map(|i| {
                let filter_lock = Arc::clone(&filter_lock);
                let i = i + 1;
                thread::spawn(move || {
                    for _ in 0..step {
                        filter_lock.lock(i);
                        unsafe {
                            VALUE = VALUE + 1;
                        }
                        filter_lock.unlock(i);
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        unsafe {
            assert_eq!(VALUE, n * step);
        }
        let duration = start.elapsed();
        println!("FilterLock Time elapsed: {:?}", duration);
    }
}
