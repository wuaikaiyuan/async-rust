use std::thread;
use std::time::Duration;
use std::time::Instant;
use tokio::time::sleep;

// tokio 将每个async 函数都是一个 Future，每个线程中有多个 task
// tokio 的 sleep 可阻塞当前 task 而切换上下文到其他 task 继续执行，await 也则会阻塞当前 task
// 标准库中的 sleep 则是直接阻塞当前线程，不会切换上下文
// tokio::join! 宏不创建任务，其只会允许在任务中同时执行多个 Future

async fn pre_coffee_mug() {
    println!("Pouring milk ...");
    thread::sleep(Duration::from_secs(3));
    println!("Milk poured.");
    println!("Putting instant coffee ...");
    thread::sleep(Duration::from_secs(3));
    println!("Instant coffee pub.");
}

async fn make_coffee() {
    println!("boling kettle...");
    sleep(Duration::from_secs(10)).await;
    println!("kettle boiled.");
    println!("Pouring boiled water ...");
    thread::sleep(Duration::from_secs(3));
    println!("boiled water poured.");
}

async fn make_toast() {
    println!("putting bread in toaster ...");
    sleep(Duration::from_secs(10)).await;
    println!("bread toasted.");
    println!("buttering toasted bread...");
    thread::sleep(Duration::from_secs(5));
    println!("toasted bread buttered");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    let coffee_mug_step = pre_coffee_mug();
    let coffee_step = make_coffee();
    let toast_step = make_toast();

    tokio::join!(coffee_mug_step, coffee_step, toast_step);

    println!("elapsed: {:?}", start.elapsed().as_secs());

    Ok(())
}
