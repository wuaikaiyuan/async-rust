use log::{error, info};
// #[macro_use]
// mod crate::commons;
use crate::commons::{FutureType, HIGH_CHANNEL, LOW_CHANNEL, Runtime};
use std::{future::Future, panic::catch_unwind, thread};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use async_task::{Runnable, Task};
use std::sync::LazyLock;

///! 一个 QUEUE -> 多个线程处理
///! 多个 QUEUE


// 队列：static 修饰确保其生命周期和程序一样长
// LazyLock 只会被初始化一次
static HIGH_QUEUE: LazyLock<flume::Sender<Runnable>> = LazyLock::new(|| {
    // let (tx, rx) = flume::unbounded::<Runnable>();

    // // 增加在队列中工作的线程数：创建 3 个线程
    // for _ in 0..2 {
    //     let rx = rx.clone();
    //     thread::spawn(move || {
    //         while let Ok(runnable) = rx.recv() {
    //             println!("runnable accpted");
    //             // 此处使用 catch_unwind 捕获 panic 是因为不知道传递给异步运行时的代码质量
    //             // 它将捕获代码抛出的任何错误，可根据结果返回 Ok(x) 或 Err(e)
    //             let _ = catch_unwind(|| runnable.run());
    //         }
    //     });
    // }
    
    // tx

    let high_num = std::env::var("HIGH_NUM").unwrap().parse::<usize>().unwrap();
    for _ in 0..high_num  {
        let high_receiver = HIGH_CHANNEL.1.clone();
        let low_receiver = LOW_CHANNEL.1.clone();

        thread::spawn(move || {
            loop {
                match high_receiver.try_recv() {
                    Ok(runnable) => {
                        let _ = catch_unwind(|| runnable.run());
                    },
                    Err(_) => {
                        match low_receiver.try_recv() {
                            Ok(runnable) => {
                                let _ = catch_unwind(|| runnable.run());
                            },
                            Err(_) => {
                                //TODO：第十章会对此进行优化，使用更响应机制的 thread parking 和 condition variables
                                std::thread::sleep(Duration::from_millis(100));
                            }
                        }
                    }
                }
                
            }
        });
    }

    HIGH_CHANNEL.0.clone()
});

static LOW_QUEUE: LazyLock<flume::Sender<Runnable>> = LazyLock::new(|| {
    // let (tx, rx) = flume::unbounded::<Runnable>();

    // // 增加在队列中工作的线程数：创建 3 个线程
    // for _ in 0..1 {
    //     let rx = rx.clone();
    //     thread::spawn(move || {
    //         while let Ok(runnable) = rx.recv() {
    //             println!("runnable accpted");
    //             // 此处使用 catch_unwind 捕获 panic 是因为不知道传递给异步运行时的代码质量
    //             // 它将捕获代码抛出的任何错误，可根据结果返回 Ok(x) 或 Err(e)
    //             let _ = catch_unwind(|| runnable.run());
    //         }
    //     });
    // }
    
    // tx

    for _ in 0..1  {   
        // let high_receiver = LOW_CHANNEL.1.clone();
        let low_receiver = LOW_CHANNEL.1.clone();
        thread::spawn(move || {
            match low_receiver.try_recv() {
                Ok(runnable) => { 
                    let _ = catch_unwind(|| runnable.run()); 
                },
                Err(_) => {
                    // match high_receiver.try_recv() {
                    //     Ok(runnable) => { 
                    //         let _ = catch_unwind(|| runnable.run()); 
                    //     },
                    //     Err(_) => {
                            //TODO：第十章会对此进行优化，使用更响应机制的 thread parking 和 condition variables
                            std::thread::sleep(Duration::from_millis(100));
                    //     }
                    // }
                },
            }
        });
    }
    LOW_CHANNEL.0.clone()
});

pub trait FutureOrderLabel: Future {
    fn get_order(&self) -> FutureType;
}

pub struct CounterFuture {
    pub count: u32,
    // pub order: FutureType,
}

impl Future for CounterFuture {
    type Output = u32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.count += 1;
        std::thread::sleep(Duration::from_secs(1));
        println!("CounterFuture poll count :{}", self.count);
        if self.count < 3 {
            println!("pending ...");
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(self.count)
        }
    }
}

// impl FutureOrderLabel for CounterFuture {
//     fn get_order(&self) -> FutureType {
//         self.order
//     }
// }


// future -> task -> queue
pub fn spawn_task<F, T>(future: F, order: FutureType) -> Task<T> 
    // 'static 保证此函数的生命周期和程序一样长
    where F: Future<Output = T> + Send + 'static,
    T: Send + 'static
{
    // 创建闭包，用于将 future 转换为 runnable
    // let schedule_high = |runnable| HIGH_QUEUE.send(runnable).unwrap();
    // let schedule_low = |runnable| LOW_QUEUE.send(runnable).unwrap();

    // let schedule = match order {
    //     FutureType::High => schedule_high,
    //     FutureType::Low => schedule_low
    // };
    // ---------- 优化如下 ---------
    
    let queue = match order {
        FutureType::High => &HIGH_QUEUE,
        FutureType::Low => &LOW_QUEUE,
    };

    let schedule = |runnable| queue.send(runnable).map_err(|e| {
        error!("failed to send task: {:?}", e);
        e
    }).unwrap();

    // -----------------------------------

    // runnable 和 task 拥有同一个指向 Fufure 的指针
    let (runnable, task) = async_task::spawn(future, schedule);
    
    runnable.schedule();

    info!("HIGH_QUEUE count: {}", HIGH_QUEUE.len());
    info!("LOW_QUEUE count: {}", LOW_QUEUE.len());

    task
}

#[macro_export]
macro_rules! spawn_task_macro {
    ($future:expr, $order: expr) => {
        // 如何调用当前crate中的函数spawn_task
        $crate::multi_worker_queue::spawn_task($future, $order)
    };
    ($future:expr) => {
        spawn_task_macro!($future, FutureType::Low)
    };
}

impl Runtime {
    
    pub fn run(&self) {
        std::env::set_var("HIGH_NUM", self.high_num.to_string());
        std::env::set_var("LOW_NUM", self.low_num.to_string());

        println!("high_num: {}", self.high_num);

        let high = spawn_task_macro!(async {}, FutureType::High);
        let low = spawn_task_macro!(async {}, FutureType::Low);

        join_future!(high, low);
    }
}