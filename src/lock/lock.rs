use std::{sync::Arc, thread, time::Instant};

pub trait Lock {
    fn new() -> Self;
    fn lock(&self);
    fn unlock(&self);
}

pub fn test_lock<T>(lock: T, lock_name: &str)
where
    T: Lock + Send + Sync + 'static,
{
    let n = 10;
    let step = 1000000;
    static mut VALUE: usize = 0;
    let lock = Arc::new(lock);
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
    println!("{} Time elapsed: {:?}", lock_name, duration);
}
