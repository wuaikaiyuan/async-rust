use std::fs::Metadata;
use std::path::PathBuf;
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncReadExt;
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

// 使用异步监听文件变动

// 读文件
async fn read_file(filename: &str) -> Result<String, std::io::Error> {
    let mut file = AsyncFile::open(filename).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    Ok(contents)
}

// 文件监听
async fn watch_file_changes(tx: watch::Sender<bool>) {
    let path = PathBuf::from("data.txt");
    let mut last_modified = None;
    loop {
        if let Ok(metadata) = path.metadata() {
            let modified = metadata.modified().unwrap();

            if last_modified != Some(modified) {
                last_modified = Some(modified);
                let _ = tx.send(true);
            }
        }
        // 标准库中的sleep不会将task发送给tokio的executor
        // tokio 运行时会在当前进程中切换上下文并执行另一个线程
        // tokio 的 sleep 是非阻塞的
        sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建一个通道，用于通知文件变动
    // 单生产者多消费者，允许一个生产者向多个消费者发送消息
    let (tx, mut rx) = watch::channel(false);

    // 创建 tokio 线程
    // 将生产者 tx 传递给 watch_file_changes 函数
    tokio::spawn(watch_file_changes(tx));

    loop {
        // 等待文件变动
        if rx.changed().await.is_ok() {
            // 和`watch_file_changes`函数中`tx.send`的值一致
            println!("File changed value: {}", *rx.borrow_and_update());

            if let Ok(content) = read_file("data.txt").await {
                println!("File content: {}", content);
            }
        } else if rx.changed().await.is_err() {
            println!("tx handle closed");
            break;
        }
    }

    Ok(())
}
