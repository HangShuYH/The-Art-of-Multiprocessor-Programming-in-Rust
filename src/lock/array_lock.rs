use std::{
    cell::{RefCell, UnsafeCell},
    sync::atomic::{AtomicUsize, Ordering},
};

use thread_local::ThreadLocal;

use super::lock::Lock;

const MAX_THREADS: usize = 20; //If the number of threads is bigger than MAX_THREADS, then mutual exclusion is violated
const CACHELINE_IN_BYTES: usize = 8; //Assume the cache line is 8 bytes, so each flag element is in a separate cache line
pub struct ArrayLock<T> {
    flag: UnsafeCell<[bool; MAX_THREADS * CACHELINE_IN_BYTES]>, //Padding cache line
    tail: AtomicUsize,
    slot_index: ThreadLocal<RefCell<usize>>,
    data: UnsafeCell<T>,
}
impl<T> Lock<T> for ArrayLock<T> {
    fn new(data: T) -> Self {
        ArrayLock {
            flag: {
                let mut flag = [false; MAX_THREADS * CACHELINE_IN_BYTES];
                flag[0] = true;
                UnsafeCell::new(flag)
            },
            tail: AtomicUsize::new(0),
            slot_index: ThreadLocal::new(),
            data: UnsafeCell::new(data),
        }
    }
    fn lock(&self) -> &mut T {
        let idx = self.tail.fetch_add(CACHELINE_IN_BYTES, Ordering::Relaxed)
            % (MAX_THREADS * CACHELINE_IN_BYTES);
        let slot_index = self.slot_index.get_or(|| RefCell::new(0));
        *slot_index.borrow_mut() = idx;
        unsafe {
            while !(*self.flag.get())[idx] {
                std::hint::spin_loop();
            }
            &mut *self.data.get()
        }
    }
    fn unlock(&self) {
        let slot_index = *self.slot_index.get().unwrap().borrow();
        unsafe {
            (*self.flag.get())[slot_index] = false;
            (*self.flag.get())
                [(slot_index + CACHELINE_IN_BYTES) % (MAX_THREADS * CACHELINE_IN_BYTES)] = true;
        }
    }
}

unsafe impl<T> Sync for ArrayLock<T> {}

#[cfg(test)]
mod tests {
    use crate::lock::lock::test_lock;

    use super::ArrayLock;

    #[test]
    fn test_array_lock() {
        test_lock::<ArrayLock<usize>>("ArrayLock");
    }
}
