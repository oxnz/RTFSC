use std::collections::HashMap;

use hpack::{Decoder, Encoder};
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    task::JoinSet,
};

use crate::http::{
    Header, Request, RequestBuilder, Response,
    v2::{
        FrameType, flags,
        transport::{Frame, Transport},
    },
};

#[derive(Debug, Default)]
pub struct Server {
    workers: JoinSet<()>,
}

impl Server {
    pub async fn serve<A: ToSocketAddrs>(&mut self, addr: A) -> std::io::Result<()> {
        let socket = TcpListener::bind(addr).await?;
        let mut workers = JoinSet::new();
        loop {
            match socket.accept().await {
                Ok((stream, _remote_addr)) => {
                    workers.spawn(async move {
                        if let Err(e) = Self::handle_client(stream.into()).await {
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

    pub async fn handle_client(mut client: Client) -> std::io::Result<()> {
        tracing::debug!("client: {client:?}");
        client.exchange_preface().await?;
        tracing::debug!("preface exchanged");
        loop {
            match client.read_request().await {
                Ok(request) => {
                    let response = Self::handle_request(request).await?;
                    client.send_response(response).await?;
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

    pub async fn handle_request(request: Request) -> std::io::Result<Response> {
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

#[derive(Debug)]
pub struct Client {
    transport: Transport,
    streams: HashMap<u32, RequestBuilder>,
}

impl From<TcpStream> for Client {
    fn from(value: TcpStream) -> Self {
        Self {
            transport: value.into(),
            streams: HashMap::default(),
        }
    }
}

impl Client {
    pub async fn exchange_preface(&mut self) -> std::io::Result<()> {
        self.transport.exchange_preface().await
    }

    pub async fn read_request(&mut self) -> std::io::Result<Request> {
        tracing::info!("read request");
        loop {
            let frame = self.transport.read_frame().await?;
            tracing::info!("read frame: {frame:?}");
            match FrameType::from(frame.r#type) {
                FrameType::Data => {
                    let request_builder = self
                        .streams
                        .entry(frame.stream_id)
                        .or_insert(Default::default());
                    request_builder.extend_body(frame.payload.as_slice());
                    if 0 != frame.flags & flags::END_STREAM {
                        return std::mem::take(request_builder).build();
                    }
                }
                FrameType::Headers => {
                    let request_builder = self
                        .streams
                        .entry(frame.stream_id)
                        .or_insert(Default::default());
                    let mut decoder = Decoder::new();
                    for (k, v) in decoder.decode(&frame.payload).unwrap() {
                        let key = str::from_utf8(&k).unwrap();
                        let value = str::from_utf8(&v).unwrap();
                        match key {
                            ":method" => {
                                request_builder.set_method(value.parse().unwrap());
                            }
                            ":scheme" => {
                                request_builder.set_scheme(value.into());
                            }
                            ":authority" => {
                                request_builder.set_authority(value.to_string());
                            }
                            ":path" => {
                                request_builder.set_path(value.to_string());
                            }
                            _ => {
                                request_builder.add_header(key, value);
                            }
                        }
                    }
                    if 0 != frame.flags & flags::END_STREAM {
                        return std::mem::take(request_builder).build();
                    }
                }
                FrameType::Settings => {
                    tracing::info!("ignore settings");
                }
                FrameType::WindowUpdate => {
                    tracing::info!("window update: {frame:?}");
                }
                _ => {
                    tracing::error!("unknown frame: {frame:?}");
                }
            }
        }
        todo!()
    }

    pub async fn send_response(&mut self, response: Response) -> std::io::Result<()> {
        tracing::info!("send reponse: {response:?}");
        let mut encoder = Encoder::new();
        let payload = encoder.encode([(b":status".as_slice(), b"200".as_slice())]);
        let header_frame = Frame {
            r#type: 0x01,
            flags: flags::END_HEADERS,
            stream_id: 1,
            payload,
        };
        self.transport.send_frame(&header_frame).await?;
        let data_frame = Frame {
            r#type: 0x00,
            flags: flags::END_STREAM,
            stream_id: 1,
            payload: response.body.unwrap_or_default(),
        };
        self.transport.send_frame(&data_frame).await?;
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
