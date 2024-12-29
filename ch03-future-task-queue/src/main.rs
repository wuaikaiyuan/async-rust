// use std::{future::Future, panic::catch_unwind, thread};
// use std::pin::Pin;
// use std::task::{Context, Poll};
// use std::time::{Duration, Instant};
// use std::sync::LazyLock;

// use async_task::{Runnable, Task};

// // 一个 Queue -> 多个线程处理
// // 多个 Queue


// // 队列：static 修饰确保其生命周期和程序一样长
// // LazyLock 只会被初始化一次
// static QUEUE: LazyLock<flume::Sender<Runnable>> = LazyLock::new(|| {
//     let (tx, rx) = flume::unbounded::<Runnable>();

//     // 增加在队列中工作的线程数：创建 3 个线程
//     for _ in 0..3 {
//         thread::spawn(move || {
//             while let Ok(runnable) = rx.clone().recv() {
//                 println!("runnable accpted");
//                 // 此处使用 catch_unwind 捕获 panic 是因为不知道传递给异步运行时的代码质量
//                 // 它将捕获代码抛出的任何错误，可根据结果返回 Ok(x) 或 Err(e)
//                 let _ = catch_unwind(|| runnable.run());
//             }
//         });
//     }
    
//     tx
// });


// // future -> task -> queue
// fn spawn_task<F, T>(future: F) -> Task<T> 
//     // 'static 保证此函数的生命周期和程序一样长
//     where F: Future<Output = T> + Send + 'static,
//     T: Send + 'static
// {
//     // 创建闭包，用于将 future 转换为 runnable
//     let schedule = |runnable| QUEUE.send(runnable).unwrap();
//     // runnable 和 task 拥有同一个指向 Fufure 的指针
//     let (runnable, task) = async_task::spawn(future, schedule);

//     runnable.schedule();
//     println!("QUEUE count: {}", QUEUE.len());
//     task
// }

// struct CounterFuture {
//     count: u32,
// }

// impl Future for CounterFuture {
//     type Output = u32;

//     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         self.count += 1;
//         std::thread::sleep(Duration::from_secs(1));
//         println!("CounterFuture poll count :{}", self.count);
//         if self.count < 3 {
//             println!("pending ...");
//             cx.waker().wake_by_ref();
//             Poll::Pending
//         } else {
//             Poll::Ready(self.count)
//         }
//     }
// }

// async fn async_fn() {
//     std::thread::sleep(Duration::from_secs(1));
//     println!("async fn");
// }

// struct AsyncSleep {
//     start_time: Instant,
//     duration: Duration
// }

// impl AsyncSleep {
//     fn new(duration: Duration) -> Self {
//         AsyncSleep {
//             start_time: Instant::now(),
//             duration
//         }
//     }
// }

// impl Future for AsyncSleep {
//     type Output = bool;
//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         if self.start_time.elapsed() >= self.duration {
//             Poll::Ready(true)
//         } else {
//             cx.waker().wake_by_ref();
//             Poll::Pending
//         }
//     }
// }

// fn main() {
//     let one = CounterFuture { count: 0};
//     let two = CounterFuture { count: 0};

//     let task1 = spawn_task(one);
//     let task2 = spawn_task(two);

//     let task3 = spawn_task(async {
//         async_fn().await;
//         async_fn().await;
//         async_fn().await;
//         async_fn().await;
//     });

//     std::thread::sleep(Duration::from_secs(5));
//     println!("before block on");

//     future::block_on(task1);
//     future::block_on(task2);
//     future::block_on(task3);
// }

use ch03_future_task_queue::{multi_task, sing_task, multi_task_runtime};

fn main() {
    // sing_task();
    // multi_task();
    multi_task_runtime();
}