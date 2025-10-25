use std::{collections::HashMap, sync::Arc};

use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    task::JoinSet,
};

use crate::http::{Header, Request, RequestBuilder, Response, v2::transport::Transport};

#[derive(Debug, Default)]
pub struct Router {}

impl Router {
    pub async fn handle_request(&self, request: Request) -> std::io::Result<Response> {
        tracing::info!("handle request: {request:?}");
        let content = b"it works!".to_vec();
        Ok(Response::new(
            crate::http::Version::Http("2.0".to_string()),
            crate::http::StatusCode::Ok,
            None,
            vec![Header::ContentLength(content.len())],
            Some(content),
        ))
    }
}

#[derive(Debug, Default)]
pub struct Server {
    router: Arc<Router>,
    workers: JoinSet<()>,
}

impl Server {
    pub async fn serve<A: ToSocketAddrs>(&mut self, addr: A) -> std::io::Result<()> {
        let socket = TcpListener::bind(addr).await?;
        let mut workers = JoinSet::new();
        loop {
            match socket.accept().await {
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
            if let Some(Err(e)) = workers.try_join_next() {
                tracing::error!("join error: {e:?}");
            }
        }
        Ok(())
    }

    pub async fn handle_client(mut client: Client, router: Arc<Router>) -> std::io::Result<()> {
        tracing::debug!("client: {client:?}");
        client.exchange_preface().await?;
        tracing::debug!("preface exchanged");
        client.process(router).await
    }
}

#[derive(Debug)]
pub struct Client {
    transport: Transport,
    streams: HashMap<u32, RequestBuilder>,
    tasks: JoinSet<()>,
}

impl From<TcpStream> for Client {
    fn from(value: TcpStream) -> Self {
        Self {
            transport: value.into(),
            streams: HashMap::default(),
            tasks: JoinSet::new(),
        }
    }
}

impl Client {
    pub async fn exchange_preface(&mut self) -> std::io::Result<()> {
        self.transport.exchange_preface().await
    }

    pub async fn process(&mut self, router: Arc<Router>) -> std::io::Result<()> {
        loop {
            match self.transport.read_request().await {
                Ok(request) => {
                    let response = router.handle_request(request).await?;
                    self.transport.send_response(response).await?;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    tracing::info!("client EOF");
                    break;
                }
                Err(e) => {
                    tracing::error!("{e:?}");
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
            .with_max_level(tracing::Level::INFO)
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
}
