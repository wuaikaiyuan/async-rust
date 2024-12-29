use hyper_util::client::legacy::connect::{Connected, Connection};

use std::net::Shutdown;
use std::net::{TcpStream, ToSocketAddrs, TcpListener};
use std::pin::Pin;
use std::task::{Context, Poll};
use anyhow::{bail, Context as _, Error, Result};
use async_native_tls::TlsStream;
use smol::{io, prelude::*, Async};
use std::future::Future;
use tower_service::Service;
use smol::future;
use http_body_util::Empty;
use bytes::Bytes;
use rustom_runtime::{commons::{FutureType, Runtime}, spawn_task_macro};

use hyper::{Request, Response};
use hyper_util::client::legacy::Client;

use hyper::body::Incoming;

#[derive(Clone)]
pub struct CustomExecutor;

impl <F: Future + Send + 'static> hyper::rt::Executor<F> for CustomExecutor {
    fn execute(&self, fut: F) {
        spawn_task_macro!(async {
            println!("sending request");
            fut.await;
        }).detach();
    }
}

pub enum CustormStream {
    Plain(Async<TcpStream>),
    Tls(TlsStream<Async<TcpStream>>)
}

#[derive(Clone)]
pub struct CustomConnector;

/*
impl hyper::service::Service<hyper::Uri> for CustomConnector {

    type Response = hyper_util::rt::TokioIo<CustormStream>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    
    fn call(&self, uri: Uri) -> Self::Future {
        Box::pin(async move {
            let host = uri.host().context("host parse error")?;

            match uri.scheme_str() {
                Some("http") => {
                    let socket_addr = {
                        let host = host.to_string();
                        let port = uri.port_u16().unwrap_or(80);
                        smol::unblock(move || (host.as_str(), port)
                            .to_socket_addrs())
                            .await?
                            .next()
                            .context("cannot resolve address")?
                    };
                    let stream = Async::new(TcpStream::connect(socket_addr)?)?;
                    Ok(hyper_util::rt::TokioIo::new(CustormStream::Plain(stream)))
                },
                Some("https") => {
                    let socket_addr = {
                        let host = host.to_string();
                        let port = uri.port_u16().unwrap_or(443);
                        smol::unblock(move || (host.as_str(), port)
                            .to_socket_addrs())
                            .await?
                            .next()
                            .context("cannot resolve address")?
                    };

                    let stream = Async::new(TcpStream::connect(socket_addr)?)?;
                    let stream = async_native_tls::TlsConnector::new().connect(host, stream).await?;
                    Ok(hyper_util::rt::TokioIo::new(CustormStream::Tls(stream)))
                },
                _=> bail!("unknown scheme")
            }
        })
    }
}
*/

impl Service<hyper::Uri> for CustomConnector {
    type Response = hyper_util::rt::TokioIo<CustormStream>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // 连接器总是准备就绪
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: hyper::Uri) -> Self::Future {
        Box::pin(async move {
            let host = uri.host().context("host parse error")?;

            match uri.scheme_str() {
                Some("http") => {
                    let socket_addr = {
                        let host = host.to_string();
                        let port = uri.port_u16().unwrap_or(80);
                        smol::unblock(move || (host.as_str(), port)
                            .to_socket_addrs())
                            .await?
                            .next()
                            .context("cannot resolve address")?
                    };
                    let stream = Async::new(TcpStream::connect(socket_addr)?)?;
                    Ok(hyper_util::rt::TokioIo::new(CustormStream::Plain(stream)))
                },
                Some("https") => {
                    let socket_addr = {
                        let host = host.to_string();
                        let port = uri.port_u16().unwrap_or(443);
                        smol::unblock(move || (host.as_str(), port)
                            .to_socket_addrs())
                            .await?
                            .next()
                            .context("cannot resolve address")?
                    };

                    let stream = Async::new(TcpStream::connect(socket_addr)?)?;
                    let stream = async_native_tls::TlsConnector::new().connect(host, stream).await?;
                    Ok(hyper_util::rt::TokioIo::new(CustormStream::Tls(stream)))
                },
                _=> bail!("unknown scheme")
            }
        })
    }
}

impl tokio::io::AsyncRead for CustormStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>>  {
        match &mut *self {
            CustormStream::Plain(s) => {
                // 此处调用 Async<TcpStream> 的 poll_read 方法，尝试从 TCP 流中读取数据到缓冲区中
                Pin::new(s)
                    .poll_read(cx, buf.initialize_unfilled())
                    .map_ok(|size| {
                        // 接收读取的字节数 size，将缓冲区的读取位置向前推进 size 个字节表示示已经读取了这些字节数据
                        buf.advance(size);
                    })
            }
            CustormStream::Tls(s) => {
                // 同上
                Pin::new(s)
                    .poll_read(cx, buf.initialize_unfilled())
                    .map_ok(|size| {
                        buf.advance(size);
                    })
            }
        }
    }
}

impl tokio::io::AsyncWrite for CustormStream {
    fn poll_write(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
                buf: &[u8],
            ) -> Poll<std::io::Result<usize>> {
        match &mut *self {
            CustormStream::Plain(s) => {
                Pin::new(s).poll_write(cx, buf)
            }
            CustormStream::Tls(s) => {
                Pin::new(s).poll_write(cx, buf)
            }
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match &mut *self {
            CustormStream::Plain(s) => {
                Pin::new(s).poll_flush(cx)
            }
            CustormStream::Tls(s) => {
                Pin::new(s).poll_flush(cx)
            }
        }
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match &mut *self {
            CustormStream::Plain(s) => {
                s.get_ref().shutdown(Shutdown::Write)?;
                Poll::Ready(core::result::Result::<_, _>::Ok(()))
            }
            CustormStream::Tls(s) => {
                Pin::new(s).poll_close(cx)
            }
        }
    }
}

impl Connection for CustormStream {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

pub async fn fetch(req: Request<Empty<Bytes>>) -> Result<Response<Incoming>> {
    let client = Client::builder(CustomExecutor)
        .build::<CustomConnector, Empty<Bytes>>(CustomConnector);

    let response = client.request(req).await?;
    Ok(response)
}

pub fn start() -> Result<(), Box<dyn std::error::Error>> {
    Runtime::new().with_low_num(2).with_high_num(4).run();

    let future = async {
        
        let req = Request::get("http://www.baidu.com").body(Empty::<Bytes>::new()).unwrap();

        let response = fetch(req).await.unwrap();

        use http_body_util::BodyExt;
        // 调用 collect() 时需要使用 http_body_util 中的 BodyExt
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();

        let html = String::from_utf8(body_bytes.to_vec()).unwrap();

        println!("{}", html);
    };

    let test = spawn_task_macro!(future);
    let _outcome = future::block_on(test);

    Ok(())
}