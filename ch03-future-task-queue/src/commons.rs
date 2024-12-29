use log::{error, info};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use std::future::Future;
use std::sync::LazyLock;
use async_task::{Runnable, Task};
use flume::{Sender, Receiver};

pub static HIGH_CHANNEL: LazyLock<(Sender<Runnable>, Receiver<Runnable>)> = LazyLock::new(|| flume::unbounded::<Runnable>());
pub static LOW_CHANNEL: LazyLock<(Sender<Runnable>, Receiver<Runnable>)> = LazyLock::new(|| flume::unbounded::<Runnable>());

pub async fn async_fn() {
    std::thread::sleep(Duration::from_secs(1));
    println!("async fn");
}

pub struct AsyncSleep {
    start_time: Instant,
    duration: Duration
}

impl AsyncSleep {
    fn new(duration: Duration) -> Self {
        AsyncSleep {
            start_time: Instant::now(),
            duration
        }
    }
}

impl Future for AsyncSleep {
    type Output = bool;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.start_time.elapsed() >= self.duration {
            Poll::Ready(true)
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FutureType {
    High, Low
}


macro_rules! join_future {
    ($($future:expr), *) => {
        {
            let mut result = Vec::new();
            $(
                result.push(futures_lite::future::block_on($future));
            )*
            result
        }
    };
}

// 直接运行Task，返回结果集合
macro_rules! try_join {
    ($($future:expr), *) => {
        {
            let mut result = Vec::new();
            $(
                let res = std::panic::catch_unwind(|| future::block_on($future));
                result.push(res);
            )*
            result
        }
    };
}

pub struct Runtime {
    pub high_num: usize,
    pub low_num: usize,
}

impl Runtime {
    pub fn new() -> Self {
        let core_num = std::thread::available_parallelism().unwrap().get();

        Self {
            high_num: core_num - 2,
            low_num: 1
        }
    }

    pub fn with_high_num(mut self, num: usize) -> Self {
        self.high_num = num;
        self
    }

    pub fn with_low_num(mut self, num: usize) -> Self {
        self.low_num = num;
        self
    }
}

// 后台 Future
#[derive(Debug, Clone, Copy)]
pub struct BackgroundProcess;

impl Future for BackgroundProcess {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        std::thread::sleep(Duration::from_secs(1));
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}
