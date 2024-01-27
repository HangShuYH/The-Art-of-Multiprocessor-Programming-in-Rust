use std::{sync::{Mutex, Arc}, thread, time::Duration};
const N:usize = 5;
fn main() {
    let mut chopsticks = Vec::with_capacity(N);
    for _ in 0..N {
        chopsticks.push(Mutex::new(0));
    }
    let chopsticks = Arc::new(chopsticks);
    let threads:Vec<_> = (0..N).map(|i| {
        let chopsticks = Arc::clone(&chopsticks);
        thread::spawn(move || {
            loop {
                let idx1;
                let idx2;
                if i % 2 == 0 {
                    idx1 = i;
                    idx2 = (i + 1) % N;
                } else {
                    idx1 = i - 1;
                    idx2 = i;
                }
                let ch1 = chopsticks[idx1].lock().unwrap();
                println!("Thread {} take {}", i, idx1);
                thread::sleep(Duration::from_millis(50));
                let ch2 = chopsticks[idx2].lock().unwrap();
                println!("Thread {} take {}", i, idx2);
                thread::sleep(Duration::from_millis(500));
                drop(ch1);
                drop(ch2);
                thread::sleep(Duration::from_millis(500));
            }
        })
    }).collect();
    for thread in threads {
        thread.join().unwrap();
    }
}