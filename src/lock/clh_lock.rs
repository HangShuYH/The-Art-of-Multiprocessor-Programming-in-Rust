use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use super::lock::RawLock;
pub struct CLHNode {
    locked: AtomicBool,
}
impl CLHNode {
    pub fn new(locked: bool) -> Self {
        CLHNode {
            locked: AtomicBool::new(locked),
        }
    }
}
unsafe impl Send for CLHNode {}
unsafe impl Sync for CLHNode {}
pub struct CLHLock {
    tail: AtomicPtr<CLHNode>,
}
impl Default for CLHLock {
    fn default() -> Self {
        CLHLock {
            tail: AtomicPtr::new(Box::into_raw(Box::new(CLHNode::new(false)))),
        }
    }
}
impl RawLock for CLHLock {
    type Token = *const CLHNode;
    fn lock(&self) -> Self::Token {
        let cur_node = Box::into_raw(Box::new(CLHNode::new(true)));
        let prev_node = self.tail.swap(cur_node, Ordering::Relaxed);
        unsafe {
            while (*prev_node).locked.load(Ordering::Acquire) {}
            drop(Box::from_raw(prev_node));
        }
        cur_node
    }
    fn unlock(&self, cur_node: Self::Token) {
        unsafe {
            (*cur_node).locked.store(false, Ordering::Release);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lock::lock::test_lock;

    use super::CLHLock;

    #[test]
    fn test_clh_lock() {
        test_lock::<CLHLock>("CLHLock");
    }
}
