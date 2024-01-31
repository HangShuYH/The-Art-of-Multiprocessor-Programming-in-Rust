use std::{
    cell::{RefCell, UnsafeCell},
    sync::atomic::{AtomicUsize, Ordering},
};

use super::lock::Lock;
use crossbeam::utils::CachePadded;
use thread_local::ThreadLocal;

const MAX_THREADS: usize = 20; //If the number of threads is bigger than MAX_THREADS, then mutual exclusion is violated
pub struct ArrayLock<T> {
    flag: UnsafeCell<[CachePadded<bool>; MAX_THREADS]>, //Padding cache line
    tail: AtomicUsize,
    slot_index: ThreadLocal<RefCell<usize>>,
    data: UnsafeCell<T>,
}
impl<T> Lock<T> for ArrayLock<T> {
    fn new(data: T) -> Self {
        ArrayLock {
            flag: {
                let mut flag = [CachePadded::new(false); MAX_THREADS];
                flag[0] = CachePadded::new(true);
                UnsafeCell::new(flag)
            },
            tail: AtomicUsize::new(0),
            slot_index: ThreadLocal::new(),
            data: UnsafeCell::new(data),
        }
    }
    fn lock(&self) -> &mut T {
        let idx = self.tail.fetch_add(1, Ordering::Relaxed) % (MAX_THREADS);
        let slot_index = self.slot_index.get_or(|| RefCell::new(0));
        *slot_index.borrow_mut() = idx;
        unsafe {
            while !(*self.flag.get())[idx].into_inner() {
                std::hint::spin_loop();
            }
            &mut *self.data.get()
        }
    }
    fn unlock(&self) {
        let slot_index = *self.slot_index.get().unwrap().borrow();
        unsafe {
            (*self.flag.get())[slot_index] = CachePadded::new(false);
            (*self.flag.get())[(slot_index + 1) % MAX_THREADS] = CachePadded::new(true);
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
