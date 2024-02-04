use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::Arc,
    thread,
    time::Instant,
};
pub trait RawLock: Default + Send + Sync {
    fn lock(&self);
    fn unlock(&self);
}
pub struct Lock<L: RawLock, T> {
    raw_lock: L,
    data: UnsafeCell<T>,
}
impl<L: RawLock, T> Lock<L, T> {
    pub fn new(data: T) -> Lock<L, T> {
        Lock {
            raw_lock: L::default(),
            data: UnsafeCell::new(data),
        }
    }
    pub fn lock(&self) -> LockGuard<L, T> {
        self.raw_lock.lock();
        LockGuard { lock: self }
    }
    pub fn unlock(guard: LockGuard<L, T>) {
        drop(guard);
    }
}
unsafe impl<L: RawLock, T: Send> Send for Lock<L, T> {}
unsafe impl<L: RawLock, T: Sync> Sync for Lock<L, T> {}
pub struct LockGuard<'a, L: RawLock, T> {
    lock: &'a Lock<L, T>,
}
impl<'a, L: RawLock, T> Deref for LockGuard<'a, L, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}
impl<'a, L: RawLock, T> DerefMut for LockGuard<'a, L, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}
impl<'a, L: RawLock, T> Drop for LockGuard<'a, L, T> {
    fn drop(&mut self) {
        self.lock.raw_lock.unlock();
    }
}
pub fn test_lock<T: RawLock + 'static>(lock_name: &str) {
    let n = 10;
    let step = 1000000;
    let lock = Arc::new(Lock::<T, usize>::new(0));
    let start = Instant::now();
    let threads: Vec<_> = (0..n)
        .map(|_| {
            let lock = Arc::clone(&lock);
            thread::spawn(move || {
                for _ in 0..step {
                    let mut data = lock.lock();
                    *data = *data + 1;
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
    assert_eq!(n * step, *lock.lock());
    let duration = start.elapsed();
    println!("{} Time Elapsed: {:?}", lock_name, duration);
}
