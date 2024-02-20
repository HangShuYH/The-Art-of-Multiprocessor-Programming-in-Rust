use std::sync::{Condvar, Mutex};

struct LockedQueue<T> {
    items: Mutex<Vec<T>>,
    not_full: Condvar,
    not_empty: Condvar,
}
impl<T> LockedQueue<T> {
    pub fn new(capacity: usize) -> LockedQueue<T> {
        LockedQueue {
            items: Mutex::new(Vec::with_capacity(capacity)),
            not_full: Condvar::new(),
            not_empty: Condvar::new(),
        }
    }
    pub fn enqueue(&self, item: T) {
        let mut items = self.items.lock().unwrap();
        while (*items).capacity() == (*items).len() {
            items = self.not_full.wait(items).unwrap();
        }
        (*items).push(item);
        self.not_empty.notify_one();
    }
    pub fn dequeue(&self) -> T {
        let mut items = self.items.lock().unwrap();
        loop {
            if let Some(item) = (*items).pop() {
                self.not_full.notify_one();
                break item;
            } else {
                items = self.not_empty.wait(items).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};

    use super::LockedQueue;

    #[test]
    fn test_locked_queue() {
        let count = 100000;
        let queue: LockedQueue<usize> = LockedQueue::new(1000);
        let queue = Arc::new(queue);
        let producer_threads: Vec<_> = (0..5)
            .map(|_| {
                let queue = Arc::clone(&queue);
                thread::spawn(move || {
                    for _ in 0..count {
                        queue.enqueue(0);
                    }
                })
            })
            .collect();
        let consumer_threads: Vec<_> = (0..5)
            .map(|_| {
                let queue = Arc::clone(&queue);
                thread::spawn(move || {
                    for _ in 0..count {
                        queue.dequeue();
                    }
                })
            })
            .collect();
        for producer in producer_threads {
            producer.join().unwrap();
        }
        for consumer in consumer_threads {
            consumer.join().unwrap();
        }
    }
}
