use std::{collections::HashMap, str::FromStr};

use hpack::{Decoder, Encoder};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    task::JoinSet,
};

use crate::http::{
    Header, Method, Protocol, Request, Response,
    v2::{PREFACE, flags},
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
        let mut preface = [0u8; PREFACE.len()];
        client.stream.read_exact(&mut preface).await.unwrap();
        assert_eq!(preface, PREFACE);
        let client_settings = client.read_frame().await?;
        assert_eq!(client_settings.r#type, 0x04);
        let server_settings = Frame {
            r#type: 0x04,
            flags: 0,
            stream_id: 0,
            payload: vec![],
        };
        client.send_frame(&server_settings).await?;
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
            crate::http::Protocol::Http("2.0".to_string()),
            crate::http::StatusCode::Ok,
            None,
            vec![Header::ContentLength(content.len())],
            Some(content),
        ))
    }
}

#[derive(Debug)]
pub struct Client {
    stream: TcpStream,
    streams: HashMap<u32, Request>,
}

impl From<TcpStream> for Client {
    fn from(value: TcpStream) -> Self {
        Self {
            stream: value,
            streams: HashMap::default(),
        }
    }
}

#[derive(Debug, Default)]
struct RequestBuilder {
    method: Option<Method>,
    scheme: Option<Scheme>,
    authority: Option<String>,
    path: Option<String>,
    headers: Vec<Header>,
    body: Option<Vec<u8>>,
}

#[derive(Debug)]
pub enum Scheme {
    Http,
    Https,
}

impl FromStr for Scheme {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "http" => Ok(Self::Http),
            "https" => Ok(Self::Https),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid scheme",
            )),
        }
    }
}

impl RequestBuilder {
    pub fn add_header<K: Into<String>, V: Into<String>>(&mut self, name: K, value: V) {
        self.headers.push(Header::Custom {
            name: name.into(),
            value: value.into(),
        });
    }

    fn set_method(&mut self, value: Method) {
        self.method = Some(value)
    }

    fn set_path(&mut self, path: String) {
        self.path = Some(path)
    }

    pub fn build(self) -> std::io::Result<Request> {
        Ok(Request::new(
            self.method.unwrap(),
            self.path.unwrap(),
            Protocol::Http("2.0".to_string()),
            self.headers,
            self.body,
        ))
    }

    fn set_scheme(&mut self, scheme: Scheme) {
        self.scheme = Some(scheme);
    }

    fn set_authority(&mut self, authority: String) {
        self.authority = Some(authority)
    }
}

impl Client {
    pub async fn read_request(&mut self) -> std::io::Result<Request> {
        tracing::info!("read request");
        let mut request_builder = RequestBuilder::default();
        loop {
            let frame = self.read_frame().await?;
            tracing::info!("read frame: {frame:?}, request_builder: {request_builder:?}");
            match frame.r#type {
                0x01 => {
                    let mut decoder = Decoder::new();
                    for (k, v) in decoder.decode(&frame.payload).unwrap() {
                        let key = str::from_utf8(&k).unwrap();
                        let value = str::from_utf8(&v).unwrap();
                        match key {
                            ":method" => {
                                request_builder.set_method(value.parse().unwrap());
                            }
                            ":scheme" => {
                                request_builder.set_scheme(value.parse().unwrap());
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
                    return request_builder.build();
                }
                0x04 => {
                    tracing::info!("ignore settings");
                }
                0x08 => {
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
        self.send_frame(&header_frame).await?;
        let data_frame = Frame {
            r#type: 0x00,
            flags: flags::END_STREAM,
            stream_id: 1,
            payload: response.body.unwrap_or_default(),
        };
        self.send_frame(&data_frame).await?;
        Ok(())
    }

    pub async fn read_frame(&mut self) -> std::io::Result<Frame> {
        let mut header = [0u8; 9];
        self.stream.read_exact(&mut header).await?;
        let len = ((header[0] as u32) << 16) | ((header[1] as u32) << 8) | (header[2] as u32);
        let frame_type = header[3];
        let flags = header[4];
        let stream_id = ((header[5] as u32 & 0x7F) << 24)
            | ((header[6] as u32) << 16)
            | ((header[7] as u32) << 8)
            | (header[8] as u32);
        let mut payload = vec![0u8; len as usize];
        self.stream.read_exact(&mut payload).await?;
        Ok(Frame {
            r#type: frame_type,
            flags,
            stream_id,
            payload,
        })
    }

    pub async fn send_frame(&mut self, frame: &Frame) -> std::io::Result<()> {
        let len: u32 = frame.payload.len() as u32;
        self.stream.write_all(&len.to_be_bytes()[1..]).await?;
        self.stream.write_all(&[frame.r#type]).await?; // type
        self.stream.write_all(&[frame.flags]).await?; // flags
        self.stream
            .write_all(&frame.stream_id.to_be_bytes())
            .await?; // stream_id
        self.stream.write_all(&frame.payload).await
    }
}

#[derive(Debug)]
pub struct Frame {
    r#type: u8,
    flags: u8,
    stream_id: u32,
    payload: Vec<u8>,
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
