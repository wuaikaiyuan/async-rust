// 使用标准库中的 Mutex 是不想在 poll 中使用异步
use std::sync::Arc;
use tokio::time::Duration;
use tokio::sync::Mutex;

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

async fn count(count: u32, data: Arc<tokio::sync::Mutex<SharedCounter>>, count_type: CounterType) -> u32 {
    for _ in 0..count {
        let mut data = data.lock().await;
        match count_type {
            CounterType::Increment => data.increment(),
            CounterType::Decrement => data.decrement(),
        }

        std::mem::drop(data);
        std::thread::sleep(Duration::from_secs(1));
    }

    return count;
}

#[tokio::main]
async fn main() {
    let shared_data = Arc::new(tokio::sync::Mutex::new(SharedCounter { count: 0 }));
    let shared_two = shared_data.clone();

    let handle_one = tokio::task::spawn(async move {
        count(0, shared_data, CounterType::Increment).await
    });
    let handle_two = tokio::task::spawn(async move {
        count(0, shared_two, CounterType::Decrement).await
    });
    
    let _ = tokio::join!(handle_one, handle_two);
}