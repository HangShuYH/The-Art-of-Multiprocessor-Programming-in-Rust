use std::{
    ptr,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

use super::lock::RawLock;

pub struct MCSNode {
    next: AtomicPtr<MCSNode>,
    locked: AtomicBool,
}
impl MCSNode {
    fn new() -> MCSNode {
        MCSNode {
            next: AtomicPtr::default(),
            locked: AtomicBool::new(false),
        }
    }
}
pub struct MCSLock {
    tail: AtomicPtr<MCSNode>,
}
impl Default for MCSLock {
    fn default() -> Self {
        MCSLock {
            tail: AtomicPtr::default(),
        }
    }
}
unsafe impl Send for MCSLock {}
unsafe impl Sync for MCSLock {}
impl RawLock for MCSLock {
    type Token = *mut MCSNode;
    fn lock(&self) -> Self::Token {
        let cur_node = Box::into_raw(Box::new(MCSNode::new()));
        let prev = self.tail.swap(cur_node, Ordering::Relaxed);
        if !prev.is_null() {
            unsafe {
                (*cur_node).locked.store(true, Ordering::Relaxed);
                (*prev).next.store(cur_node, Ordering::Relaxed);
                while (*cur_node).locked.load(Ordering::Relaxed) {}
                drop(Box::from_raw(prev));
            }
        }
        cur_node
    }
    fn unlock(&self, cur_node: Self::Token) {
        unsafe {
            if (*cur_node).next.load(Ordering::Relaxed).is_null() {
                if self
                    .tail
                    .compare_exchange(
                        cur_node,
                        ptr::null_mut(),
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    return;
                }
                while (*cur_node).next.load(Ordering::Relaxed).is_null() {}
            }
            let next = (*cur_node).next.load(Ordering::Relaxed);
            (*next).locked.store(false, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lock::lock::test_lock;

    use super::MCSLock;

    #[test]
    fn test_mcs_lock() {
        test_lock::<MCSLock>("MCSLock");
    }
}
