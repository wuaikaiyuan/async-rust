use std::time::Duration;
use tokio::time::timeout;

// cancel safety
// 可以确保 Future 被取消时，能够正确处理正在运行的任务或资源。
// 如果一个任务正常运行而被取消的时候，应该继续运行完成后退出。

async fn slow_task() -> &'static str {
    tokio::time::sleep(Duration::from_secs(10)).await;
    "slow_task"
}

#[tokio::main]
async fn main() {
    let result = timeout(Duration::from_secs(3), slow_task()).await;
    match result {
        Ok(value) => println!("task finished: {}", value),
        Err(_) => println!("task timeout"),
    }
}
