use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::lock::RawLock;
use crossbeam::utils::CachePadded;

const MAX_THREADS: usize = 20; //If the number of threads is bigger than MAX_THREADS, then mutual exclusion is violated
pub struct ArrayLock {
    flag: UnsafeCell<[CachePadded<bool>; MAX_THREADS]>, //Padding cache line
    tail: AtomicUsize,
}
impl Default for ArrayLock {
    fn default() -> Self {
        ArrayLock {
            flag: {
                let mut flag = [CachePadded::new(false); MAX_THREADS];
                flag[0] = CachePadded::new(true);
                UnsafeCell::new(flag)
            },
            tail: AtomicUsize::new(0),
        }
    }
}
unsafe impl Send for ArrayLock {}
unsafe impl Sync for ArrayLock {}
impl RawLock for ArrayLock {
    type Token = usize;
    fn lock(&self) -> usize {
        let idx = self.tail.fetch_add(1, Ordering::Relaxed) % (MAX_THREADS);
        unsafe {
            while !(*self.flag.get())[idx].into_inner() {
                std::hint::spin_loop();
            }
        }
        idx
    }
    fn unlock(&self, slot_index: Self::Token) {
        unsafe {
            (*self.flag.get())[slot_index] = CachePadded::new(false);
            (*self.flag.get())[(slot_index + 1) % MAX_THREADS] = CachePadded::new(true);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lock::lock::test_lock;

    use super::ArrayLock;

    #[test]
    fn test_array_lock() {
        test_lock::<ArrayLock>("ArrayLock");
    }
}
