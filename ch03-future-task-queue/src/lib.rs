#[macro_use]
pub mod commons;
use commons::{async_fn, FutureType, Runtime, BackgroundProcess};

mod single_worker_queue;
use single_worker_queue::CounterFuture as SingleCounterFuture;
use futures_lite::future;
use std::time::Duration;

#[macro_use]
pub mod multi_worker_queue;

pub fn sing_task() {
    let one = SingleCounterFuture { count: 0};
    let two = SingleCounterFuture { count: 0};

    let task1 = single_worker_queue::spawn_task(one);
    let task2 = single_worker_queue::spawn_task(two);

    let task3 = single_worker_queue::spawn_task(async {
        async_fn().await;
        async_fn().await;
        async_fn().await;
        async_fn().await;
    });
    
    std::thread::sleep(Duration::from_secs(5));
    println!("before block on");

    future::block_on(task1);
    future::block_on(task2);
    future::block_on(task3);
}

pub fn multi_task() {
    let high_counter = multi_worker_queue::CounterFuture { count: 0};
    let low_counter = multi_worker_queue::CounterFuture { count: 0};

    let task1 = spawn_task_macro!(high_counter, FutureType::High);
    let task2 = spawn_task_macro!(low_counter);

    let task3 = spawn_task_macro!(async_fn());

    let task4 = spawn_task_macro!(async {
        async_fn().await;
        async_fn().await;
        async_fn().await;
        async_fn().await;
    }, FutureType::High);

    std::thread::sleep(Duration::from_secs(5));
    println!("before block on");

    // future::block_on(task1);
    // future::block_on(task2);
    // future::block_on(task3);
    // future::block_on(task4);

    // let outcome: Vec<u32> = join_future!(task1, task2);
    // let outcome_next: Vec<()> = join_future!(task3, task4);

    let cout = try_join!(task1, task2);
    let cout = try_join!(task3, task4);
}

pub fn multi_task_runtime() {
    Runtime::new().run();
    // Runtime::new().with_high_num(4).with_low_num(1).run();
    // detach: 让 Task 在后台运行
    spawn_task_macro!(BackgroundProcess{}).detach();
}