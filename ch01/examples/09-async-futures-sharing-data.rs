// 使用标准库中的 Mutex 是不想在 poll 中使用异步
use std::sync::{Arc, Mutex};
use tokio::time::Duration;
use core::task::Poll;
use std::task::Context;
use std::pin::Pin;
use std::future::Future;

#[derive(Debug)]
enum CounterType {
    Increment,
    Decrement,
}

struct SharedCounter {
    count: i32,
}

impl SharedCounter {
    fn increment(&mut self) {
        self.count += 1;
    }

    fn decrement(&mut self) {
        self.count -= 1;
    }
}

struct CounterFuture {
    counter_type: CounterType,
    data_reference: Arc<Mutex<SharedCounter>>,
    count: u32
}

impl CounterFuture {
    fn new(counter_type: CounterType, data_reference: Arc<Mutex<SharedCounter>>, count: u32) -> Self {
        CounterFuture {
            counter_type,
            data_reference,
            count
        }
    }
}

impl Future for CounterFuture {
    type Output = u32;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        std::thread::sleep(Duration::from_secs(2));
        // 使用标准库中的Mutex会阻塞当前线程，而tokio的Mutex是异步的，而poll是同步方法，
        // 那么获取锁的时候使用try_lock，不管有没获取到锁立即返回，没获取到poll返回Pending
        let mut guard = match self.data_reference.try_lock() {
            Ok(guard) => guard,
            Err(error) => {
                println!("error for {:?}: {}", self.counter_type, error);
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        };
        let value = &mut *guard;

        match self.counter_type {
            CounterType::Increment => {
                value.increment();
                println!("increment: {}", value.count);
            },
            CounterType::Decrement => {
                value.decrement();
                println!("decrement: {}", value.count);
            }
        }

        std::mem::drop(guard);
        self.count += 1;

        if self.count < 3 {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        } else {
            Poll::Ready(self.count)
        }
    }
}

#[tokio::main]
async fn main() {
    let shared_data = Arc::new(Mutex::new(SharedCounter { count: 0 }));

    let counter_one = CounterFuture::new(CounterType::Increment, shared_data.clone(), 0);
    let counter_two = CounterFuture::new(CounterType::Decrement, shared_data.clone(), 0);

    let handle_one = tokio::task::spawn(async move { counter_one.await });
    let handle_two = tokio::task::spawn(async move { counter_two.await });

    let _ = tokio::join!(handle_one, handle_two);
}