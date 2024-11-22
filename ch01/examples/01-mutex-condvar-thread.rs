use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

// 多线程实例
// 线程间通信：互斥锁和条件变量
fn main() {
    let shared_data = Arc::new((Mutex::new(false), Condvar::new()));
    let shared_data_clone = Arc::clone(&shared_data);
    let STOP = Arc::new(AtomicBool::new(false));
    let STOP_CLONE = Arc::clone(&STOP);

    let _background_thread = thread::spawn(move || {
        let (lock, cvar) = &*shared_data_clone;
        let mut receive_value = lock.lock().unwrap();
        while !STOP.load(Relaxed) {
            receive_value = cvar.wait(receive_value).unwrap();
            println!("Received value: {}", *receive_value);
        }
        println!("background thread finished");
    });

    let updater_thread = thread::spawn(move || {
        let (lock, cvar) = &*shared_data;
        let values = [false, true, false, true];

        for i in 0..4 {
            let update_value = values[i as usize];
            println!("updating value to {} ...", update_value);
            *lock.lock().unwrap() = update_value;
            cvar.notify_one();
            thread::sleep(Duration::from_millis(1000));
        }
        STOP_CLONE.store(true, Relaxed);
        println!("updater thread finished");
        cvar.notify_one();
    });
    updater_thread.join().unwrap();
}
