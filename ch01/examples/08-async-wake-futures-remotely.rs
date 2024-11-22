use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::sync::{Arc, Mutex};
use std::future::Future;
use tokio::sync::mpsc;
use tokio::task;

// 使用 Rust 远程调用接口，在我们从操作系统获得信号，表明我们一直监听的端口收到数据之前，我们需要不断 poll 网络 Future 是没有意义的。
// 解决：通过外部引用 Future 的 waker，在需要的时候 wake Future 从而避免轮询 Future。

struct MyFuture {
    state: Arc<Mutex<MyFutureState>>,
}

struct MyFutureState {
    data: Option<Vec<u8>>,
    waker: Option<Waker>,
}

impl MyFuture {
    fn new() -> (Self, Arc<Mutex<MyFutureState>>) {
        let state = Arc::new(Mutex::new(MyFutureState { data: None, waker: None }));

        (
            MyFuture { state: state.clone() },
            state
        )
    }
}

impl Future for MyFuture {
    type Output = String;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("polling the future");
        let mut state = self.state.lock().unwrap();
        if state.data.is_some() {
            let data = state.data.take().unwrap();
            Poll::Ready(String::from_utf8(data).unwrap())
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

// 此示例中可知，pool 在初始和 wake 后共发生两次 poll，具体如下：
// 第一次是初始化，此时 state.data 为 None，waker 为 当前线程的 waker，返回 Poll::Pending，此时线程进入阻塞状态，等待被唤醒；
// 第二次是 wake，此时 state.data 为 Some(data)，因此返回 Poll::Ready(String::from_utf8(data).unwrap())，此时 my_future 的状态为 Ready，返回结果。
#[tokio::main]
async fn main(){
    let (my_future, state) = MyFuture::new();

    let (tx, mut rx) = mpsc::channel::<()>(1);
    let task_handle = task::spawn(async {
        // 第一次初始化
        my_future.await
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    println!("spawn trigger task");

    let trigger_task = task::spawn(async move {
        rx.recv().await;

        let mut state = state.lock().unwrap();
        state.data = Some(b"Hello from outside".to_vec());
        
        loop {
            if let Some(waker) = state.waker.take() {
                // 第二次 wake
                waker.wake();
                break;
            }
        }
    });

    tx.send(()).await.unwrap();

    let outome = task_handle.await.unwrap();
    println!("task completed with outcome: {:?}", outome);
    trigger_task.await.unwrap();
}