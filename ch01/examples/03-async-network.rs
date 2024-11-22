use reqwest::Error;
use serde::Deserialize;
use serde_json;
use std::time::Duration;
use std::time::Instant;
use tokio::time::sleep;

#[derive(Deserialize, Debug)]
struct Response {
    url: String,
    args: serde_json::Value,
}

async fn fetch_data(seconds: u64) -> Result<Response, Error> {
    let req_url = format!("https://httpbin.org/delay/{}", seconds);
    let delay_response = reqwest::get(&req_url).await?.json().await?;
    Ok(delay_response)
}

async fn calculate_last_login() {
    sleep(Duration::from_secs(1)).await;
    println!("logged in 2 days ago");
}

// tokio::join 中虽然将 fetch_data 放到了 calculate_last_login 前面，但是这两个函数都是异步执行的，
// 查看结果可知 calculate_last_login 先输出，间接证明了 fetch_data 是非阻塞的特性
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let data = fetch_data(3);
    let time_since = calculate_last_login();

    let (posts, _time_since) = tokio::join!(data, time_since);
    let duration = start_time.elapsed();

    println!("Fetched {:?}", posts);
    println!("Time token: {:?}", duration);

    Ok(())
}
