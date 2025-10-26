use std::{sync::Arc, time::Duration};

use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    task::JoinSet,
};

use crate::http::{Header, Request, Response, ResponseBuilder, v2::connection::Connection};

#[derive(Debug, Default)]
pub struct Router {}

impl Router {
    pub async fn handle_request(&self, request: Request) -> std::io::Result<Response> {
        tracing::info!("handle request: {request:?}");
        let content = b"it works!".to_vec();
        tokio::time::sleep(Duration::from_secs(2)).await;
        ResponseBuilder::default()
            .version(crate::http::Version::Http2)
            .status(crate::http::Status::OK)
            .headers(vec![Header::new(
                "content-length",
                content.len().to_string(),
            )])
            .body(content)
            .build()
    }
}

#[derive(Debug, Default)]
pub struct Server {
    router: Arc<Router>,
}

impl Server {
    pub async fn serve<A: ToSocketAddrs>(&mut self, addr: A) -> std::io::Result<()> {
        let socket = TcpListener::bind(addr).await?;
        tracing::info!("serving at: {:?}", socket);
        let mut workers = JoinSet::new();
        loop {
            tokio::select! {
                result = socket.accept() => {
                    match result {
                        Ok((stream, _remote_addr)) => {
                            let router = Arc::clone(&self.router);
                            workers.spawn(async move {
                                if let Err(e) = Self::handle_client(stream.into(), router).await {
                                    tracing::error!("{e:?}");
                                }
                            });
                        }
                        Err(e) => tracing::error!("accept: {e:?}"),
                    }
                }
                Some(Err(e)) = workers.join_next() => {
                    tracing::error!("join error: {e:?}");
                }
            }
        }
        Ok(())
    }

    pub async fn handle_client(mut client: Client, router: Arc<Router>) -> std::io::Result<()> {
        tracing::debug!("client: {client:?}");
        client.exchange_preface().await?;
        client.process(router).await
    }
}

#[derive(Debug)]
pub struct Client {
    connection: Connection,
    tasks: JoinSet<std::io::Result<(u32, Response)>>,
}

impl From<TcpStream> for Client {
    fn from(value: TcpStream) -> Self {
        Self {
            connection: value.into(),
            tasks: JoinSet::new(),
        }
    }
}

impl Client {
    pub async fn exchange_preface(&mut self) -> std::io::Result<()> {
        self.connection.exchange_preface().await
    }

    pub async fn process(&mut self, router: Arc<Router>) -> std::io::Result<()> {
        loop {
            tokio::select! {
                Some(result) = self.tasks.join_next() => {
                    match result {
                        Ok(result) => {
                            match result {
                                Ok((stream_id, response)) => {
                                    if let Err(e) = self.connection.send_response(stream_id, response).await {
                                        tracing::error!("send response: {e:?}");
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("{e:?}");
                                }
                            }
                        },
                        Err(e) => {
                            tracing::error!("join: {e:?}");
                        }
                    }
                }
                result = self.connection.read_request() => {
                    match result {
                        Ok((stream_id, request)) => {
                            let router = Arc::clone(&router);
                            self.tasks.spawn(async move {
                                router.handle_request(request).await.map(|x| (stream_id, x))
                            });
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                            tracing::info!("client EOF");
                            break;
                        }
                        Err(e) => {
                            tracing::error!("read request: {e:?}");
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_serve() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
        let mut server = Server::default();
        let addr = "127.0.0.1:8000";
        server.serve(addr).await.unwrap();
    }

    #[test]
    fn test_get() {
        let output = std::process::Command::new("curl")
            .args([
                "--http2-prior-knowledge",
                "-v",
                "--silent",
                "127.0.0.1:8000",
            ])
            .output()
            .unwrap();
        let stdout = output.stdout;
        let stderr = output.stderr;
        unsafe {
            println!("stdout:\n{}", str::from_utf8_unchecked(&stdout));
            eprintln!("stderr:\n{}", str::from_utf8_unchecked(&stderr));
        }
    }

    #[test]
    fn test_multiplex() {
        let output = std::process::Command::new("curl")
            .args([
                "--http2-prior-knowledge",
                "--parallel",
                "-v",
                "--silent",
                "127.0.0.1:8000/1",
                "127.0.0.1:8000/2",
                "127.0.0.1:8000/3",
                "127.0.0.1:8000/4",
            ])
            .output()
            .unwrap();
        let stdout = output.stdout;
        let stderr = output.stderr;
        unsafe {
            println!("stdout:\n{}", str::from_utf8_unchecked(&stdout));
            eprintln!("stderr:\n{}", str::from_utf8_unchecked(&stderr));
        }
    }
}
