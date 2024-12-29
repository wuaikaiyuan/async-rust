use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll as MioPoll, Token};
use smol::future;
use std::io::{Read, Write};
use std::time::Duration;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use rustom_runtime::{commons::{FutureType, Runtime}, spawn_task_macro};


const SERVER: Token = Token(0);
const CLIENT: Token = Token(1);


struct ServerFuture {
    server: TcpListener,
    poll: MioPoll,
}

impl Future for ServerFuture {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut events = Events::with_capacity(1);

        let _ = self.poll.poll(&mut events, Some(Duration::from_millis(200))).unwrap();

        for event in events.iter() {
            if event.token() == CLIENT {
                return Poll::Ready("Hello from mio!".to_string());
            } else if event.token() == SERVER && event.is_readable(){
                let (mut stream, _) = self.server.accept().unwrap();
                let mut buffer = [0u8; 1024];// 固定大小的缓冲区
                let mut received_data = Vec::new();// 动态增长的向量

                /*
                循环会持续读取，直到：
                1.读取到流的末尾（返回0）
                2.发生错误
                3.所有数据都被读取完

                举个例子：
                假设要接收一个 4000 字节的消息，使用 1024 字节的缓冲区，循环过程：
                - 第一次读取：1024 字节 → 存入 received_data
                - 第二次读取：1024 字节 → 追加到 received_data
                - 第三次读取：1024 字节 → 追加到 received_data
                - 第四次读取：928 字节 → 追加到 received_data
                - 第五次读取：返回 0，表示读取完成 → 退出循环
                
                此方式能确保完整接收所有数据，不会发生数据丢失。在网络编程中称为"缓冲读取"（buffered reading）。
                 */
                loop {
                    // read data from the socket
                    match stream.read(&mut buffer) {
                        Ok(n) if n > 0 => {
                            // 只取实际读取的字节数(n)，追加到received_data
                            received_data.extend_from_slice(&buffer[..n]);
                        },
                        Ok(_) => break,// 读取完毕（返回0）时退出
                        Err(err) => {// 发生错误时退出
                            eprintln!("reading from stream error: {}", err);
                            break;
                        }
                    }
                }

                if !received_data.is_empty() {
                    let rev_str = String::from_utf8_lossy(&received_data);
                    return Poll::Ready(rev_str.to_string());
                }

                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        }

        cx.waker().wake_by_ref();
        return Poll::Pending;
    }
}


pub fn start() -> Result<(), Box<dyn std::error::Error>> {
    Runtime::new().with_low_num(2).with_high_num(4).run();

    let addr = "127.0.0.1:13265".parse()?;
    let mut server = TcpListener::bind(addr)?;
    let mut stream = TcpStream::connect(server.local_addr()?)?;

    let poll: MioPoll = MioPoll::new()?;
    poll.registry()
        .register(&mut server, SERVER, Interest::READABLE | Interest::WRITABLE)?;

    let server_worker = ServerFuture {server, poll};

    let test = spawn_task_macro!(server_worker);

    let mut client_poll: MioPoll = MioPoll::new()?;
    client_poll.registry().register(&mut stream, CLIENT, Interest::READABLE | Interest::WRITABLE)?;

    let mut events = Events::with_capacity(128);
    let _ = client_poll.poll(&mut events, None)?;
    for event in events.iter() {
        if event.token() == CLIENT && event.is_writable() {
            let message = "that's so dingo!\n";
            let _ = stream.write_all(message.as_bytes());
        }
    }

    let outcome = future::block_on(test);
    println!("outcome: {}", outcome);

    Ok(())
}