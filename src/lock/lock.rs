use std::{sync::Arc, thread, time::Instant};

pub trait Lock<T> {
    fn new(data: T) -> Self;
    fn lock(&self) -> &mut T;
    fn unlock(&self);
}

pub fn test_lock<T>(lock_name: &str)
where
    T: Lock<usize> + Send + Sync + 'static,
{
    let n = 10;
    let step = 1000000;
    let lock = Arc::new(T::new(0));
    let start = Instant::now();
    let threads: Vec<_> = (0..n)
        .map(|_| {
            let lock = Arc::clone(&lock);
            thread::spawn(move || {
                for _ in 0..step {
                    let data_ref = lock.lock();
                    *data_ref = *data_ref + 1;
                    lock.unlock();
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
    assert_eq!(n * step, *lock.lock());
    lock.unlock();
    let duration = start.elapsed();
    println!("{} Time Elapsed: {:?}", lock_name, duration);
}
