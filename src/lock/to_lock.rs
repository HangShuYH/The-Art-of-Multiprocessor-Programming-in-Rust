use std::{
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
    time::{Duration, Instant},
};

use super::lock::{RawLock, RawTryLock};
pub struct ToNode {
    pred: AtomicPtr<ToNode>,
    locked: AtomicBool,
}
impl ToNode {
    pub fn new() -> ToNode {
        ToNode {
            pred: AtomicPtr::default(),
            locked: AtomicBool::new(true),
        }
    }
}
pub struct ToLock {
    tail: AtomicPtr<ToNode>,
}
impl Default for ToLock {
    fn default() -> Self {
        ToLock {
            tail: AtomicPtr::default(),
        }
    }
}
unsafe impl Send for ToLock {}
unsafe impl Sync for ToLock {}
impl RawLock for ToLock {
    type Token = *const ToNode;
    fn lock(&self) -> Self::Token {
        let cur_node = Box::into_raw(Box::new(ToNode::new()));
        let mut pred_node = self.tail.swap(cur_node, Ordering::Relaxed);
        if !pred_node.is_null() {
            unsafe {
                loop {
                    if !(*pred_node).locked.load(Ordering::Relaxed) {
                        drop(Box::from_raw(pred_node));
                        return cur_node;
                    }
                    let pred_pred_node = (*pred_node).pred.load(Ordering::Relaxed);
                    if !pred_pred_node.is_null() {
                        pred_node = pred_pred_node;
                    }
                }
            }
        }
        cur_node
    }
    fn unlock(&self, cur_node: Self::Token) {
        unsafe {
            (*cur_node).locked.store(false, Ordering::Relaxed);
        }
    }
}

impl RawTryLock for ToLock {
    fn try_lock(&self, duration: Duration) -> Result<Self::Token, ()> {
        let cur_node = Box::into_raw(Box::new(ToNode::new()));
        let mut pred_node = self.tail.swap(cur_node, Ordering::Relaxed);
        if !pred_node.is_null() {
            unsafe {
                let start = Instant::now();
                loop {
                    if !(*pred_node).locked.load(Ordering::Relaxed) {
                        drop(Box::from_raw(pred_node));
                        return Ok(cur_node);
                    }
                    let pred_pred_node = (*pred_node).pred.load(Ordering::Relaxed);
                    if !pred_pred_node.is_null() {
                        pred_node = pred_pred_node;
                    }
                    if Instant::now() - start >= duration {
                        break;
                    }
                }
                // Where to drop Timeout Node?
                (*cur_node).pred.store(pred_node, Ordering::Relaxed);
                return Err(());
            }
        }
        Ok(cur_node)
    }
}

#[cfg(test)]
mod tests {

    use crate::lock::lock::{test_lock, test_try_lock};

    use super::ToLock;

    #[test]
    fn test_to_lock() {
        test_lock::<ToLock>("ToLock");
    }
    #[test]
    fn test_to_try_lock() {
        test_try_lock::<ToLock>("ToTryLock");
    }
}
